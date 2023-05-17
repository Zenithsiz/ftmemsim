//! Filipe's Tiered Memory Simulator (`ftmemsim`)

// Features
#![feature(decl_macro, lint_reasons, get_many_mut, seek_stream_len)]

// Modules
mod args;
mod classifiers;
mod logger;
mod pin_trace;
mod sim;
mod util;

// Imports
use {
	self::{args::Args, pin_trace::PinTrace},
	crate::{classifiers::hemem, sim::Simulator, util::FemtoDuration},
	anyhow::Context,
	clap::Parser,
	std::fs,
};

fn main() -> Result<(), anyhow::Error> {
	// Get arguments
	let args = Args::parse();
	logger::pre_init::debug(format!("Args: {args:?}"));

	// Initialize logging
	logger::init(args.log_file.as_deref(), args.log_file_append);

	// Read the trace file
	let pin_trace = {
		let mut pin_trace_file = fs::File::open(&args.trace_file).context("Unable to open trace file")?;
		PinTrace::from_reader(&mut pin_trace_file).context("Unable to parse pin trace")?
	};
	tracing::trace!(target: "ftmemsim::parse_pin_trace", ?pin_trace, "Parsed pin trace");

	// Run the simulator
	let mut sim = Simulator::new(0);
	let mut hemem = hemem::HeMem::new(
		hemem::Config {
			read_hot_threshold:       8,
			write_hot_threshold:      4,
			global_cooling_threshold: 18,
		},
		vec![
			hemem::Memory::new("ram", 100, hemem::memories::AccessLatencies {
				read:  FemtoDuration::from_nanos_f64(1.5),
				write: FemtoDuration::from_nanos_f64(1.0),
				fault: FemtoDuration::from_nanos_f64(10.0),
			}),
			hemem::Memory::new("optane", 800, hemem::memories::AccessLatencies {
				read:  FemtoDuration::from_nanos_f64(5.0),
				write: FemtoDuration::from_nanos_f64(4.0),
				fault: FemtoDuration::from_nanos_f64(50.0),
			}),
		],
	);

	sim.run(pin_trace.records.iter().copied(), &mut hemem)
		.context("Unable to run simulator")?;

	Ok(())
}
