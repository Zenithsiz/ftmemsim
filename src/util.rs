//! Utilities

// Modules
mod duration;

// Exports
pub use duration::FemtoDuration;

// Imports
use std::io;

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
