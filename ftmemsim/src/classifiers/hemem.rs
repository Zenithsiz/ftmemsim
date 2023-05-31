//! Hemem classifier

// Modules
pub mod memories;
pub mod page_table;
pub mod statistics;

// Exports
pub use self::{
	memories::{Memories, Memory},
	page_table::{Page, PagePtr, PageTable},
	statistics::Statistics,
};

// Imports
use {
	self::memories::MemIdx,
	crate::{pin_trace, sim},
	anyhow::Context,
	itertools::Itertools,
	std::fmt,
};

/// Hemem classifier
#[derive(Debug)]
pub struct HeMem {
	/// Config
	config: Config,

	/// Memories
	memories: Memories,

	/// Page table
	page_table: PageTable,

	/// Statistics
	statistics: Statistics,
}

impl HeMem {
	/// Creates a hemem classifier
	pub fn new(config: Config, memories: Vec<Memory>) -> Self {
		Self {
			config,
			memories: Memories::new(memories),
			page_table: PageTable::new(),
			statistics: Statistics::new(),
		}
	}

	/// Maps a page to the first available memory and returns it.
	///
	/// # Errors
	/// Returns an error if unable to insert
	///
	/// # Panics
	/// Panics if the page is already mapped.
	pub fn map_page(&mut self, page_ptr: PagePtr) -> Result<MemIdx, anyhow::Error> {
		if self.page_table.contains(page_ptr) {
			panic!("Page is already mapped: {page_ptr:?}");
		}

		for (mem_idx, mem) in self.memories.iter_mut() {
			// Try to reserve a page on this memory
			match mem.reserve_page() {
				// If we got it, add the page to the page table
				Ok(()) => {
					let page = Page::new(page_ptr, mem_idx);
					self.page_table.insert(page).expect("Unable to insert unmapped page");
					return Ok(mem_idx);
				},

				// If we didn't manage to, go to the next page
				Err(err) => {
					tracing::trace!(?page_ptr, ?mem_idx, ?err, "Unable to reserve page on memory");
					continue;
				},
			}
		}

		// If we got here, all memories were full
		anyhow::bail!("All memories were full");
	}

	/// Cools a memory by (at most) `count` pages.
	///
	/// Returns the number of pages cooled.
	///
	/// # Panics
	/// Panics if `mem_idx` is an invalid memory index
	pub fn cool_memory(&mut self, cur_time: u64, mem_idx: MemIdx, count: usize) -> usize {
		let mut cooled_pages = 0;
		for page_ptr in self.page_table.coldest_pages(mem_idx, count) {
			if self.cool_page(cur_time, page_ptr).is_ok() {
				cooled_pages += 1;
			}
		}

		cooled_pages
	}

	/// Migrates a page, possibly cooling the destination if full.
	///
	/// # Errors
	/// Returns an error if unable to migrate the page to `dst_mem_idx`.
	///
	/// # Panics
	/// Panics if `page_ptr` isn't a mapped page.
	/// Panics if `dst_mem_idx` is an invalid memory index.
	pub fn migrate_page(&mut self, cur_time: u64, page_ptr: PagePtr, dst_mem_idx: MemIdx) -> Result<(), anyhow::Error> {
		let page = self.page_table.get_mut(page_ptr).expect("Page wasn't in page table");
		let src_mem_idx = page.mem_idx();

		match self.memories.migrate_page(src_mem_idx, dst_mem_idx) {
			// If we managed to, move the page's memory
			Ok(()) => {
				page.move_mem(dst_mem_idx);

				self.statistics
					.register_page_migration(page_ptr, statistics::PageMigration {
						time:    cur_time,
						mem_idx: dst_mem_idx,
					});
			},

			// Else try to cool the destination memory first, then try again
			Err(err) => {
				tracing::trace!(
					?src_mem_idx,
					?dst_mem_idx,
					?err,
					"Unable to migrate page, cooling destination"
				);

				// TODO: Cool for more than just 1 page at a time?
				let pages_cooled = self.cool_memory(cur_time, dst_mem_idx, 1);
				match pages_cooled > 0 {
					// If we cooled at least 1 page, migrate it
					true => {
						self.memories
							.migrate_page(src_mem_idx, dst_mem_idx)
							.expect("Just freed some pages when cooling");
						self.page_table
							.get_mut(page_ptr)
							.expect("Page wasn't in page table")
							.move_mem(dst_mem_idx);
						self.statistics
							.register_page_migration(page_ptr, statistics::PageMigration {
								time:    cur_time,
								mem_idx: dst_mem_idx,
							});
					},

					// Else we can't move it
					false => anyhow::bail!("Cooler memory is full, even after cooling it"),
				}
			},
		}

		Ok(())
	}

	/// Cools a page.
	///
	/// # Errors
	/// Returns an error if unable to cool the page.
	///
	/// # Panics
	/// Panics if `page_ptr` isn't a mapped page.
	pub fn cool_page(&mut self, cur_time: u64, page_ptr: PagePtr) -> Result<(), anyhow::Error> {
		let page = self.page_table.get_mut(page_ptr).expect("Page wasn't in page table");

		// Get the new memory index in the slower memory
		let dst_mem_idx = match self.memories.slower_memory(page.mem_idx()) {
			Some(mem_idx) => mem_idx,
			None => anyhow::bail!("Page is already in the slowest memory"),
		};

		// Then try to migrate it
		self.migrate_page(cur_time, page_ptr, dst_mem_idx)
			.context("Unable to migrate page to slower memory")
	}

	/// Warms a page.
	///
	/// # Errors
	/// Returns an error if unable to warm the page.
	///
	/// # Panics
	/// Panics if `page_ptr` isn't a mapped page.
	pub fn warm_page(&mut self, cur_time: u64, page_ptr: PagePtr) -> Result<(), anyhow::Error> {
		let page = self.page_table.get_mut(page_ptr).expect("Page wasn't in page table");

		// Get the new memory index in the faster memory
		let src_mem_idx = page.mem_idx();
		let dst_mem_idx = match self.memories.faster_memory(src_mem_idx) {
			Some(mem_idx) => mem_idx,
			None => anyhow::bail!("Page is already in the hottest memory"),
		};

		// Then try to migrate it
		self.migrate_page(cur_time, page_ptr, dst_mem_idx)
			.context("Unable to migrate page to faster memory")
	}

	/// Returns the statistics
	pub fn statistics(&self) -> &Statistics {
		&self.statistics
	}
}

impl sim::Classifier for HeMem {
	fn handle_trace(&mut self, trace: sim::Trace) -> Result<(), anyhow::Error> {
		tracing::trace!(?trace, "Received trace");
		let page_ptr = PagePtr::new(trace.record.addr);

		// Map the page if it doesn't exist
		let page_prev_mem_idx = self.page_table.get_mut(page_ptr).map(|page| page.mem_idx());
		if !self.page_table.contains(page_ptr) {
			tracing::trace!(?page_ptr, "Mapping page");
			let page_mem_idx = self.map_page(page_ptr).context("Unable to map page")?;

			// Register an initial page migration when mapping
			self.statistics
				.register_page_migration(page_ptr, statistics::PageMigration {
					time:    trace.record.time,
					mem_idx: page_mem_idx,
				});
		};
		let page = self.page_table.get_mut(page_ptr).expect("Page wasn't in page table");
		let page_was_hot = page.is_hot(self.config.read_hot_threshold, self.config.write_hot_threshold);
		let page_prev_temperature = page.temperature();


		// Register the access on the page
		match trace.record.kind {
			pin_trace::RecordAccessKind::Read => page.register_read_access(),
			pin_trace::RecordAccessKind::Write => page.register_write_access(),
		};

		// If the page is over the threshold, cool all pages
		let caused_cooling = page.over_threshold(self.config.global_cooling_threshold);
		if caused_cooling {
			self.page_table.cool_all_pages();
		}

		// Finally check if it's still hot and adjust if necessary
		let page = self.page_table.get_mut(page_ptr).expect("Page wasn't in page table");
		let page_is_hot = page.is_hot(self.config.read_hot_threshold, self.config.write_hot_threshold);
		let page_cur_mem_idx = page.mem_idx();
		let page_cur_temperature = page.temperature();

		// If the page isn't hot and it was hot, cool it
		if !page_is_hot && page_was_hot {
			tracing::trace!(?page_ptr, "Page is no longer hot, cooling it");
			if let Err(err) = self.cool_page(trace.record.time, page_ptr) {
				tracing::trace!(?page_ptr, ?err, "Unable to cool page");
			}
		}

		// If the page was cold and is now hot, head it
		if page_is_hot && !page_was_hot {
			tracing::trace!(?page_ptr, "Page is now hot, warming it");
			if let Err(err) = self.warm_page(trace.record.time, page_ptr) {
				tracing::trace!(?page_ptr, ?err, "Unable to warm page");
			}
		}

		// Finally register the access in our statistics
		self.statistics.register_access(statistics::Access {
			time: trace.record.time,
			page_ptr,
			kind: match trace.record.kind {
				pin_trace::RecordAccessKind::Read => statistics::AccessKind::Read,
				pin_trace::RecordAccessKind::Write => statistics::AccessKind::Write,
			},
			mem: match page_prev_mem_idx {
				Some(mem_idx) => statistics::AccessMem::Resided(mem_idx),
				None => statistics::AccessMem::Mapped(page_cur_mem_idx),
			},
			prev_temperature: page_prev_temperature,
			cur_temperature: page_cur_temperature,
			caused_cooling,
		});

		Ok(())
	}

	fn fmt_debug(&mut self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		// Note: Start with a newline, since we're a multi-line output
		f.pad("\n")?;

		for (mem_idx, memory) in self.memories.iter_mut() {
			let name = memory.name();
			let len = memory.page_len();
			let capacity = memory.page_capacity();
			let occupancy_percentage = 100.0 * (len as f64 / capacity as f64);
			writeln!(
				f,
				"Memory {name} ({mem_idx:?}): {len} / {capacity} ({occupancy_percentage:.2}%)"
			)?;
		}

		{
			let total_accesses = self.statistics.accesses().len();
			writeln!(f, "Total accesses: {total_accesses}")?;

			let average_prev_temperature = self
				.statistics
				.accesses()
				.iter()
				.map(|access| access.prev_temperature as f64)
				.collect::<average::Variance>();

			let average_cur_temperature = self
				.statistics
				.accesses()
				.iter()
				.map(|access| access.cur_temperature as f64)
				.collect::<average::Variance>();

			writeln!(
				f,
				"Average temperature: {:.4} ± {:.4} (Prev), {:.4} ± {:.4} (Cur)",
				average_prev_temperature.mean(),
				average_prev_temperature.error(),
				average_cur_temperature.mean(),
				average_cur_temperature.error()
			)?;

			let average_page_migrations = self
				.statistics
				.page_migrations()
				.iter()
				.map(|(_, migrations)| migrations.len() as f64)
				.collect::<average::Variance>();
			let (min_page_migrations, max_page_migrations) = self
				.statistics
				.page_migrations()
				.iter()
				.map(|(_, migrations)| migrations.len() as f64)
				.minmax()
				.into_option()
				.unwrap_or((f64::NEG_INFINITY, f64::INFINITY));

			writeln!(
				f,
				"Average page migrations: {:.4} ± {:.4} ({min_page_migrations:.2}..{max_page_migrations:.2})",
				average_page_migrations.mean(),
				average_page_migrations.error()
			)?;
		}

		Ok(())
	}
}

/// Configuration
#[derive(Clone, Debug)]
pub struct Config {
	// R/W hotness threshold
	pub read_hot_threshold:  usize,
	pub write_hot_threshold: usize,

	/// Max threshold for global cooling
	pub global_cooling_threshold: usize,
}
