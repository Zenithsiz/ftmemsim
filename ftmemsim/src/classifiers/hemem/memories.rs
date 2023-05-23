//! Memories

// Imports
use crate::util::FemtoDuration;

/// Memories.
///
/// Maintains an array of memories, ordered from fastest to slowest.
#[derive(Clone, Debug)]
pub struct Memories {
	/// All memories
	memories: Vec<Memory>,
}

impl Memories {
	/// Creates all the memories from an iterator of memories.
	///
	/// Memories are expected to be ordered from fastest to slowest.
	pub fn new(memories: impl IntoIterator<Item = Memory>) -> Self {
		Self {
			memories: memories.into_iter().collect(),
		}
	}

	/// Returns an iterator over all memories from fastest to slowest
	pub fn iter_mut(&mut self) -> impl Iterator<Item = (MemIdx, &mut Memory)> {
		self.memories
			.iter_mut()
			.enumerate()
			.map(|(idx, mem)| (MemIdx(idx), mem))
	}

	/// Migrates a page from `src` to `dst`
	///
	/// Returns `Err` if the source memory is empty or the destination memory is full.
	///
	/// # Panics
	/// Panics if either `src` or `dst` are invalid memory indexes
	pub fn migrate_page(&mut self, src: MemIdx, dst: MemIdx) -> Result<(), anyhow::Error> {
		// Get the memories
		let [src, dst] = match self.memories.get_many_mut([src.0, dst.0]) {
			Ok(mems) => mems,
			Err(_) => match src == dst {
				true => return Ok(()),
				_ => panic!("Source or destination memory indexes were invalid"),
			},
		};

		// Ensure they're not empty/full
		anyhow::ensure!(!src.is_empty(), "Source memory was empty");
		anyhow::ensure!(!dst.is_full(), "Destination memory was full");

		// Then move them
		dst.reserve_page().expect("Unable to reserve after checking non-full");
		src.release_page().expect("Unable to release after checking non-empty");

		Ok(())
	}

	/// Returns the faster memory after `mem_idx`
	pub fn faster_memory(&self, mem_idx: MemIdx) -> Option<MemIdx> {
		match mem_idx.0 {
			0 => None,
			_ => Some(MemIdx(mem_idx.0 - 1)),
		}
	}

	/// Returns the slower memory after `mem_idx`
	pub fn slower_memory(&self, mem_idx: MemIdx) -> Option<MemIdx> {
		match mem_idx.0 + 1 >= self.memories.len() {
			true => None,
			false => Some(MemIdx(mem_idx.0 + 1)),
		}
	}
}

/// Memory index
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct MemIdx(usize);

/// Memory
#[derive(Clone, Debug)]
pub struct Memory {
	/// Name
	name: String,

	// Page size/capacity
	page_len:      usize,
	page_capacity: usize,

	// Latencies
	_latencies: AccessLatencies,
}

impl Memory {
	/// Creates a new memory
	pub fn new(name: impl Into<String>, page_capacity: usize, latencies: AccessLatencies) -> Self {
		Self {
			name: name.into(),
			page_len: 0,
			page_capacity,
			_latencies: latencies,
		}
	}

	/// Attempts to release a page on this memory
	pub fn release_page(&mut self) -> Result<(), anyhow::Error> {
		// Ensure we're not empty
		anyhow::ensure!(!self.is_empty(), "Memory is empty");

		// Then release the page
		self.page_len -= 1;

		Ok(())
	}

	/// Attempts to reserve a page on this memory
	pub fn reserve_page(&mut self) -> Result<(), anyhow::Error> {
		// Ensure we're not full
		anyhow::ensure!(!self.is_full(), "Memory is full");

		// Then reserve the page
		self.page_len += 1;

		Ok(())
	}

	/// Returns if this memory is empty
	pub fn is_empty(&self) -> bool {
		self.page_len == 0
	}

	/// Returns if this memory is full
	pub fn is_full(&self) -> bool {
		self.page_len >= self.page_capacity
	}

	/// Returns this memory's name
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Returns the number of resident pages in this memory
	pub fn page_len(&self) -> usize {
		self.page_len
	}

	/// Returns the page capacity of this memory
	pub fn page_capacity(&self) -> usize {
		self.page_capacity
	}
}

/// Access latencies
#[derive(Clone, Copy, Debug)]
pub struct AccessLatencies {
	/// Read latency
	pub read: FemtoDuration,

	/// Write latency
	pub write: FemtoDuration,

	/// Fault latency
	pub fault: FemtoDuration,
}
