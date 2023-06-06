//! Simulator

// Imports
use {
	crate::{pin_trace, pin_trace::PinTraceReader},
	anyhow::Context,
	std::{
		fmt,
		io,
		ops::Range,
		time::{Duration, Instant},
	},
};

/// Simulator
#[derive(Debug)]
pub struct Simulator {
	/// Trace skip
	///
	/// Dictates how many records are skipped for each trace.
	/// A value of 0 implies that the classifier receives all records as traces,
	/// while a value of 1 implies it receives every other record as a trace.
	trace_skip: usize,

	/// Debug output period
	///
	/// Interval in which to output debug output for the classifier
	debug_output_period: Duration,
}

impl Simulator {
	/// Creates a new simulator
	pub fn new(trace_skip: usize, debug_output_period: Duration) -> Self {
		Self {
			trace_skip,
			debug_output_period,
		}
	}

	/// Runs the simulator on all traces from `pin_trace_reader` with classifier `classifier`
	pub fn run<C: Classifier>(
		&mut self,
		pin_trace_reader: &mut PinTraceReader<impl io::Read + io::Seek>,
		classifier: &mut C,
	) -> Result<RunOutput, anyhow::Error> {
		// Note: We start in the past so that we output right away at the start
		let mut last_debug_time = Instant::now() - self.debug_output_period;

		// Create the record iterator
		let total_records = pin_trace_reader.records_remaining();
		let record_it = std::iter::from_fn(|| pin_trace_reader.read_next().transpose());

		// Go through all records
		let mut first_time = None;
		let mut last_time = None;
		for (record_idx, record_res) in record_it.enumerate().step_by(self.trace_skip + 1) {
			let record = record_res.context("Unable to read next record")?;

			// Update the first and last time.
			// TODO: We're assuming all records are ordered by time, check when this *doesn't* happen
			first_time.get_or_insert(record.time);
			last_time = Some(record.time);

			// Handle each trace
			let trace = Trace { record };
			classifier
				.handle_trace(trace)
				.context("Unable to handle trace with classifier")?;

			// Then show debug output, if it's been long enough
			let cur_time = Instant::now();
			if cur_time.duration_since(last_debug_time) >= self.debug_output_period {
				let records_processed_percentage = 100.0 * (record_idx as f64 / total_records as f64);
				tracing::info!(
					"[{records_processed_percentage:.2}%] Debug: {}",
					ftmemsim_util::DisplayWrapper::new(|f| classifier.fmt_debug(f))
				);
				last_debug_time = cur_time
			}
		}

		Ok(RunOutput {
			time_span: first_time.zip(last_time).map(|(first, last)| first..(last + 1)),
		})
	}
}

/// Output for [`Simulator::run`]
#[derive(Clone, Debug)]
pub struct RunOutput {
	/// Time span
	pub time_span: Option<Range<u64>>,
}

/// Classifier
pub trait Classifier {
	/// Handles a trace
	fn handle_trace(&mut self, trace: Trace) -> Result<(), anyhow::Error>;

	/// Formats debug output to `f`.
	fn fmt_debug(&mut self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error>;
}

/// Trace
#[derive(Clone, Copy, Debug)]
pub struct Trace {
	/// Record that originated this trace
	pub record: pin_trace::Record,
}
