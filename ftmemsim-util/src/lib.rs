//! Utilities

// Features
#![feature(decl_macro)]

// Modules
pub mod duration;
pub mod logger;

// Exports
pub use duration::FemtoDuration;

// Imports
use std::{cell::RefCell, fmt, io};

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
