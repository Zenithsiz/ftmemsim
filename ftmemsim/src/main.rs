//! Filipe's Tiered Memory Simulator (`ftmemsim`)

// Features
#![feature(decl_macro, lint_reasons, get_many_mut, seek_stream_len)]

// Modules
mod args;
mod classifiers;
mod pin_trace;
mod sim;

// Imports
use {
	self::{args::Args, pin_trace::PinTraceReader},
	crate::{classifiers::hemem, sim::Simulator},
	anyhow::Context,
	clap::Parser,
	ftmemsim_util::{logger, FemtoDuration},
	std::{fs, time::Duration},
};

fn main() -> Result<(), anyhow::Error> {
	// Get arguments
	let args = Args::parse();
	logger::pre_init::debug(format!("Args: {args:?}"));

	// Initialize logging
	logger::init(args.log_file.as_deref(), args.log_file_append);

	// Read the trace file
	let mut pin_trace_file = fs::File::open(&args.trace_file).context("Unable to open trace file")?;
	let mut pin_trace_reader = PinTraceReader::from_reader(&mut pin_trace_file).context("Unable to parse pin trace")?;
	tracing::trace!(target: "ftmemsim::parse_pin_trace", ?pin_trace_reader, "Parsed pin trace");

	// Run the simulator
	let mut sim = Simulator::new(0, Duration::from_secs_f64(1.0));
	let mut hemem = hemem::HeMem::new(
		hemem::Config {
			read_hot_threshold:       8,
			write_hot_threshold:      4,
			global_cooling_threshold: 18,
		},
		vec![
			hemem::Memory::new("ram", 1000, hemem::memories::AccessLatencies {
				read:  FemtoDuration::from_nanos_f64(1.5),
				write: FemtoDuration::from_nanos_f64(1.0),
				fault: FemtoDuration::from_nanos_f64(10.0),
			}),
			hemem::Memory::new("optane", 8000, hemem::memories::AccessLatencies {
				read:  FemtoDuration::from_nanos_f64(5.0),
				write: FemtoDuration::from_nanos_f64(4.0),
				fault: FemtoDuration::from_nanos_f64(50.0),
			}),
		],
	);

	sim.run(&mut pin_trace_reader, &mut hemem)
		.context("Unable to run simulator")?;

	// TODO: Make locations configurable
	{
		let page_locations_file =
			fs::File::create("resources/data/page_locations.json").context("Unable to create page locations file")?;
		let page_locations = hemem.page_locations();
		serde_json::to_writer(page_locations_file, &page_locations)
			.context("Unable to write to page locations file")?;
	}

	{
		let page_accesses_file =
			fs::File::create("resources/data/page_accesses.json").context("Unable to create page accesses file")?;
		let page_accesses = hemem.page_accesses();
		serde_json::to_writer(page_accesses_file, &page_accesses).context("Unable to write to page accesses file")?;
	}

	Ok(())
}
