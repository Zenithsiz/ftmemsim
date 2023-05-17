//! Page table

// Imports
use {
	super::memories::MemIdx,
	itertools::Itertools,
	std::collections::{btree_map, BTreeMap},
};

/// Page table
#[derive(Debug)]
pub struct PageTable {
	/// All pages, by their address
	// TODO: `HashMap` with custom hash? We don't use the order
	pages: BTreeMap<PagePtr, Page>,

	/// Current cooling clock tick
	cooling_clock_tick: usize,
}

impl PageTable {
	/// Creates an empty page table
	pub fn new() -> Self {
		Self {
			pages:              BTreeMap::new(),
			cooling_clock_tick: 0,
		}
	}

	/// Returns if a page exists in this page table
	pub fn contains(&self, page_ptr: PagePtr) -> bool {
		self.pages.contains_key(&page_ptr)
	}

	/// Returns a page from this page table.
	pub fn get_mut(&mut self, page_ptr: PagePtr) -> Option<&mut Page> {
		// Try to get the page
		let page = self.pages.get_mut(&page_ptr)?;

		// Then cool it before returning
		page.cool_accesses(self.cooling_clock_tick);
		Some(page)
	}

	/// Inserts a new page into this page table.
	///
	/// # Errors
	/// Returns an error if the page already exists
	pub fn insert(&mut self, mut page: Page) -> Result<(), anyhow::Error> {
		match self.pages.entry(page.ptr) {
			btree_map::Entry::Vacant(entry) => {
				// Note: We cool it before inserting to ensure that the page is up to date.
				page.cool_accesses(self.cooling_clock_tick);
				entry.insert(page);

				Ok(())
			},
			btree_map::Entry::Occupied(_) => anyhow::bail!("Page already existed: {page:?}"),
		}
	}

	/// Cools all pages
	pub fn cool_all_pages(&mut self) {
		// Note: Instead of increasing all pages at once, we simply increase
		//       our cooling clock and then, when accessing a page, we update
		//       the pages's clock tick to match ours.
		self.cooling_clock_tick += 1;
	}

	/// Returns the `count` coldest pages in memory `mem_idx`
	// TODO: Optimize this function? Runs in `O(N)` with all memories
	pub fn coldest_pages(&mut self, mem_idx: MemIdx, count: usize) -> Vec<PagePtr> {
		self.pages
			.iter_mut()
			.update(|(_, page)| page.cool_accesses(self.cooling_clock_tick))
			.filter(|(_, page)| page.mem_idx == mem_idx)
			.sorted_by_key(|(_, page)| page.temperature())
			.map(|(&page_ptr, _)| page_ptr)
			.take(count)
			.collect()
	}
}

/// Page
#[derive(Clone, Copy, Debug)]
pub struct Page {
	/// Pointer
	ptr: PagePtr,

	/// Memory index
	mem_idx: MemIdx,

	// Read/Write accesses (adjusted)
	adjusted_read_accesses:  usize,
	adjusted_write_accesses: usize,

	// Current cooling clock tick
	cur_cooling_clock_tick: usize,
}

impl Page {
	/// Creates a new page
	pub fn new(ptr: PagePtr, mem_idx: MemIdx) -> Self {
		Self {
			ptr,
			mem_idx,
			adjusted_read_accesses: 0,
			adjusted_write_accesses: 0,
			cur_cooling_clock_tick: 0,
		}
	}

	/// Returns the memory index of this page
	pub fn mem_idx(&self) -> MemIdx {
		self.mem_idx
	}

	/// Moves this page to `mem_idx`
	pub fn move_mem(&mut self, mem_idx: MemIdx) {
		self.mem_idx = mem_idx;
	}

	/// Registers a read access
	pub fn register_read_access(&mut self) {
		self.adjusted_read_accesses += 1;
	}

	/// Registers a write access
	pub fn register_write_access(&mut self) {
		self.adjusted_write_accesses += 1;
	}

	/// Returns if this page is hot
	pub fn is_hot(&self, read_hot_threshold: usize, write_hot_threshold: usize) -> bool {
		self.adjusted_read_accesses >= read_hot_threshold || self.adjusted_write_accesses >= write_hot_threshold
	}

	/// Returns this page's temperature
	pub fn temperature(&self) -> usize {
		// TODO: Tune this definition?
		self.adjusted_read_accesses * 1 + self.adjusted_write_accesses * 2
	}

	/// Returns if either read or write accesses are over a threshold
	pub fn over_threshold(&self, threshold: usize) -> bool {
		self.adjusted_read_accesses >= threshold || self.adjusted_write_accesses >= threshold
	}

	/// Cools this page's accesses to match the global cooling clock
	fn cool_accesses(&mut self, global_access_cooling_clock_tick: usize) {
		assert!(self.cur_cooling_clock_tick <= global_access_cooling_clock_tick);

		let offset = (global_access_cooling_clock_tick - self.cur_cooling_clock_tick).min(usize::BITS as usize - 1);
		self.adjusted_read_accesses >>= offset;
		self.adjusted_write_accesses >>= offset;
		self.cur_cooling_clock_tick = global_access_cooling_clock_tick;
	}
}

/// Page pointer.
///
/// Guaranteed to be page-aligned
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct PagePtr(u64);

impl std::fmt::Debug for PagePtr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("PagePtr")
			.field(&format_args!("{:#010x}", self.0))
			.finish()
	}
}

impl PagePtr {
	/// Page mask
	pub const PAGE_MASK: u64 = (1 << 12 - 1);

	/// Creates a page pointer from a `u64`.
	///
	/// Will truncate any bits below the page mask.
	pub fn new(page: u64) -> Self {
		Self(page & !Self::PAGE_MASK)
	}

	/// Returns the page pointer as a u64
	pub fn _to_u64(self) -> u64 {
		self.0
	}
}