//! Duration

// Imports
use std::{fmt, writeln};

/// Duration with femto-second precision
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct FemtoDuration {
	/// Whole seconds
	secs: u64,

	/// femto seconds (0..FEMTOS_PER_SEC)
	femto_secs: u64,
}

impl FemtoDuration {
	/// Number of femto-seconds per nano-second
	pub const FEMTOS_PER_NANO: u64 = 1_000_000;
	/// Number of nano-seconds per second
	pub const NANOS_PER_SEC: u64 = 1_000_000_000;
	/// Number of femto-seconds per second
	pub const _FEMTOS_PER_SEC: u64 = 1_000_000_000_000_000;

	/// Creates a new duration from floating-point nanoseconds
	// TODO: Deal with rounding better?
	pub fn from_nanos_f64(nanos: f64) -> Self {
		let secs = (nanos / Self::NANOS_PER_SEC as f64) as u64;
		let femto_secs = ((nanos % Self::NANOS_PER_SEC as f64) * Self::FEMTOS_PER_NANO as f64) as u64;

		Self { secs, femto_secs }
	}
}


impl fmt::Display for FemtoDuration {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let secs = self.secs % 60;
		let mins = self.secs / 60 % 60;
		let hours = self.secs / 60 / 60;

		let femtos = self.femto_secs % 1000;
		let picos = self.femto_secs / 1000 % 1000;
		let nanos = self.femto_secs / 1000 / 1000 % 1000;
		let micros = self.femto_secs / 1000 / 1000 / 1000 % 1000;
		let millis = self.femto_secs / 1000 / 1000 / 1000 / 1000 % 1000;

		match (hours, mins, secs, millis, micros, nanos, picos, femtos) {
			// If we have no hours, mins or secs, format in the smallest unit
			(0, 0, 0, 0, 0, 0, 0, 0) => (),
			(0, 0, 0, 0, 0, 0, 0, _) => writeln!(f, "{femtos}fs")?,
			(0, 0, 0, 0, 0, 0, ..) => writeln!(f, "{picos}.{femtos}ps")?,
			(0, 0, 0, 0, 0, ..) => writeln!(f, "{nanos}.{picos}{femtos}ns")?,
			(0, 0, 0, 0, ..) => writeln!(f, "{micros}.{nanos}{picos}{femtos}µs")?,
			(0, 0, 0, ..) => writeln!(f, "{millis}.{micros}{nanos}{picos}{femtos}ms")?,

			// Else format it as the decimal part
			(0, 0, ..) => writeln!(f, "{secs}.{millis}{micros}{nanos}{picos}{femtos}s")?,
			(0, ..) => writeln!(f, "{mins}m{secs}.{millis}{micros}{nanos}{picos}{femtos}s")?,
			(..) => writeln!(f, "{hours}h{mins}m{secs}.{millis}{micros}{nanos}{picos}{femtos}s")?,
		}

		Ok(())
	}
}
