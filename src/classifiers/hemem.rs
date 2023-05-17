//! Hemem classifier

// Imports
use crate::sim;

/// Hemem classifier
pub struct HeMem {}

impl HeMem {
	/// Creates a hemem classifier
	pub fn new() -> Self {
		Self {}
	}
}

impl sim::Classifier for HeMem {
	fn handle_trace(&mut self, trace: sim::Trace) {
		tracing::trace!(?trace, "Received trace")
	}
}
