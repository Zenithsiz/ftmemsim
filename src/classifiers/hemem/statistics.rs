//! Statistics

// Imports
use super::{memories::MemIdx, PagePtr};

/// Statistics
#[derive(Clone, Debug)]
pub struct Statistics {
	/// All accesses
	accesses: Vec<Access>,
}

impl Statistics {
	/// Creates new, empty, statistics
	pub fn new() -> Self {
		Self { accesses: vec![] }
	}

	/// Registers an access on these statistics
	pub fn register_access(&mut self, access: Access) {
		self.accesses.push(access);
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
