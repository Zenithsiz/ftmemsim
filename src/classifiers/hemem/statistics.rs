//! Statistics

// Imports
use {
	super::{memories::MemIdx, PagePtr},
	std::collections::HashMap,
};

/// Statistics
#[derive(Clone, Debug)]
pub struct Statistics {
	/// All accesses
	accesses: Vec<Access>,

	/// Page locations
	page_locations: HashMap<PagePtr, Vec<PageLocation>>,
}

impl Statistics {
	/// Creates new, empty, statistics
	pub fn new() -> Self {
		Self {
			accesses:       vec![],
			page_locations: HashMap::new(),
		}
	}

	/// Registers an access on these statistics
	pub fn register_access(&mut self, access: Access) {
		self.accesses.push(access);
	}

	/// Registers a new location for a page
	pub fn register_page_location(&mut self, page_ptr: PagePtr, page_location: PageLocation) {
		self.page_locations.entry(page_ptr).or_default().push(page_location);
	}

	/// Returns all accesses
	pub fn accesses(&self) -> &[Access] {
		&self.accesses
	}

	/// Returns all page locations
	pub fn page_locations(&self) -> &HashMap<PagePtr, Vec<PageLocation>> {
		&self.page_locations
	}
}


/// An access to a page
#[derive(Clone, Copy, Debug)]
pub struct Access {
	/// Timestamp (TODO: unix?)
	pub time: u64,

	/// Page pointer
	pub page_ptr: PagePtr,

	/// Access kind
	pub kind: AccessKind,

	/// Access memory
	pub mem: AccessMem,

	/// Page previous temperature
	pub prev_temperature: usize,

	/// Page current temperature
	pub cur_temperature: usize,
}

/// Access kind for [`Access`]
#[derive(Clone, Copy, Debug)]
pub enum AccessKind {
	/// Read
	Read,

	/// Write
	Write,
}

/// Access memory for [`Access`]
#[derive(Clone, Copy, Debug)]
pub enum AccessMem {
	/// Page was mapped to memory
	Mapped(MemIdx),

	/// Page resided in memory
	Resided(MemIdx),
}

/// Page location
#[derive(Clone, Debug)]
pub struct PageLocation {
	/// Timestamp
	pub time: u64,

	/// Memory
	pub mem_idx: MemIdx,
}
