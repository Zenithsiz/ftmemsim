//! Simulator

// Imports
use crate::pin_trace;

/// Simulator
pub struct Simulator {
	/// Trace skip
	///
	/// Dictates how many records are skipped for each trace.
	/// A value of 0 implies that the classifier receives all records as traces,
	/// while a value of 1 implies it receives every other record as a trace.
	trace_skip: usize,
}

impl Simulator {
	/// Creates a new simulator
	pub fn new(trace_skip: usize) -> Self {
		Self { trace_skip }
	}

	/// Runs the simulator on records `records` with classifier `classifier`
	pub fn run<C: Classifier>(&mut self, records: impl IntoIterator<Item = pin_trace::Record>, classifier: &mut C) {
		for record in records.into_iter().step_by(self.trace_skip + 1) {
			let trace = Trace { record };
			classifier.handle_trace(trace);
		}
	}
}


/// Classifier
pub trait Classifier {
	/// Handles a trace
	fn handle_trace(&mut self, trace: Trace);
}

/// Trace
#[derive(Clone, Copy, Debug)]
pub struct Trace {
	/// Record that originated this trace
	pub record: pin_trace::Record,
}
