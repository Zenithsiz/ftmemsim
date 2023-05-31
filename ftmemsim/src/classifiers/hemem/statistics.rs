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

	/// Page migrations
	page_migration: HashMap<PagePtr, Vec<PageMigration>>,
}

impl Statistics {
	/// Creates new, empty, statistics
	pub fn new() -> Self {
		Self {
			accesses:       vec![],
			page_migration: HashMap::new(),
		}
	}

	/// Registers an access on these statistics
	pub fn register_access(&mut self, access: Access) {
		self.accesses.push(access);
	}

	/// Registers migration for a page
	pub fn register_page_migration(&mut self, page_ptr: PagePtr, page_migration: PageMigration) {
		self.page_migration.entry(page_ptr).or_default().push(page_migration);
	}

	/// Returns all accesses
	pub fn accesses(&self) -> &[Access] {
		&self.accesses
	}

	/// Returns all page migrations
	pub fn page_migrations(&self) -> &HashMap<PagePtr, Vec<PageMigration>> {
		&self.page_migration
	}
}

impl Default for Statistics {
	fn default() -> Self {
		Self::new()
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

	/// Caused a global cooling?
	pub caused_cooling: bool,
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

/// Page migration
#[derive(Clone, Debug)]
pub struct PageMigration {
	/// Timestamp
	pub time: u64,

	/// Previous memory
	pub prev_mem_idx: Option<MemIdx>,

	/// Memory
	pub cur_mem_idx: MemIdx,
}
