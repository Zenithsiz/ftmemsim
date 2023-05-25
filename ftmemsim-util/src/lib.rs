//! Utilities

// Features
#![feature(decl_macro)]

// Modules
pub mod duration;
pub mod logger;

// Exports
pub use duration::FemtoDuration;

// Imports
use std::{cell::RefCell, collections::BTreeMap, fmt, io};

/// Extension trait for `R: io::Read` types to read a byte array
#[extend::ext(name = ReadByteArray)]
pub impl<R: io::Read> R {
	/// Reads a byte array `[u8; N]` from this reader.
	///
	/// Returns `Err` if unable to read exactly `N` bytes.
	fn read_byte_array<const N: usize>(&mut self) -> Result<[u8; N], io::Error> {
		let mut array = [0u8; N];
		self.read_exact(&mut array)?;
		Ok(array)
	}
}

/// [`fmt::Display`] helper to display using a `FnMut(&mut fmt::Formatter)`
pub struct DisplayWrapper<F: FnMut(&mut fmt::Formatter) -> fmt::Result>(RefCell<F>);

impl<F: FnMut(&mut fmt::Formatter) -> fmt::Result> DisplayWrapper<F> {
	/// Creates a new display wrapper
	#[must_use]
	pub const fn new(func: F) -> Self {
		Self(RefCell::new(func))
	}
}


impl<F: FnMut(&mut fmt::Formatter) -> fmt::Result> fmt::Display for DisplayWrapper<F> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// Note: `f` cannot be re-entrant, so this cannot fail
		self.0.borrow_mut()(f)
	}
}


/// Page accesses
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PageAccesses {
	pub accesses: Vec<PageAccess>,
}

/// Page access
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PageAccess {
	pub page_ptr:       u64,
	pub time:           u64,
	pub mem_idx:        usize,
	pub faulted:        bool,
	pub kind:           PageAccessKind,
	pub prev_temp:      usize,
	pub cur_temp:       usize,
	pub caused_cooling: bool,
}

/// Page access kind
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum PageAccessKind {
	Read,
	Write,
}

/// Page locations
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PageLocations {
	// Note: We use a `BTreeMap` to ensure the order of the locations
	//       is always the same, as well as to sort it by page.
	// TODO: Just use `HashMap` here and instead just sort the data when
	//       creating the graphs?
	pub locations: BTreeMap<u64, Vec<PageLocation>>,
}

/// Page location over time
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PageLocation {
	pub mem_idx: usize,
	pub time:    u64,
}
