//! Filipe's Tiered Memory Simulator (`ftmemsim`)

// Features
#![feature(decl_macro, lint_reasons, get_many_mut, seek_stream_len)]

// Modules
mod args;
mod config;

// Imports
use {
	self::args::Args,
	anyhow::Context,
	clap::Parser,
	ftmemsim::{classifiers::hemem, PinTraceReader, Simulator},
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

	// Read the config file
	// TODO: Allow not passing it and use a default?
	let config = {
		let config_file = fs::File::open(&args.config_file).context("Unable to open config file")?;
		serde_json::from_reader::<_, self::config::Config>(config_file).context("Unable to parse config file")?
	};

	// Run the simulator
	let mut sim = Simulator::new(
		config.trace_skip,
		Duration::from_secs_f64(config.debug_output_period_secs),
	);
	let mut hemem = hemem::HeMem::new(
		hemem::Config {
			read_hot_threshold:       config.hemem.read_hot_threshold,
			write_hot_threshold:      config.hemem.write_hot_threshold,
			global_cooling_threshold: config.hemem.global_cooling_threshold,
		},
		config
			.hemem
			.memories
			.iter()
			.map(|mem| {
				hemem::Memory::new(&mem.name, mem.page_capacity, hemem::memories::AccessLatencies {
					read:  FemtoDuration::from_nanos_f64(mem.read_latency_ns),
					write: FemtoDuration::from_nanos_f64(mem.write_latency_ns),
					fault: FemtoDuration::from_nanos_f64(mem.fault_latency_ns),
				})
			})
			.collect(),
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
