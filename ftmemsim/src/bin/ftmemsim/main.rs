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
	ftmemsim::{
		classifiers::{hemem, hemem::memories::MemIdx},
		data,
		PinTraceReader,
		Simulator,
	},
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

	if let Some(output_path) = &args.output_file {
		let hemem_statistics = hemem.statistics();
		let data = data::Data {
			hemem: data::HeMemData {
				page_accesses:   data::PageAccesses {
					accesses: hemem_statistics
						.accesses()
						.iter()
						.map(|page_access| data::PageAccess {
							page_ptr:       page_access.page_ptr.to_u64(),
							time:           page_access.time,
							mem_idx:        match page_access.mem {
								hemem::statistics::AccessMem::Mapped(mem_idx) |
								hemem::statistics::AccessMem::Resided(mem_idx) => mem_idx.to_usize(),
							},
							faulted:        matches!(page_access.mem, hemem::statistics::AccessMem::Mapped(_)),
							kind:           match page_access.kind {
								hemem::statistics::AccessKind::Read => data::PageAccessKind::Read,
								hemem::statistics::AccessKind::Write => data::PageAccessKind::Write,
							},
							prev_temp:      page_access.prev_temperature,
							cur_temp:       page_access.cur_temperature,
							caused_cooling: page_access.caused_cooling,
						})
						.collect(),
				},
				page_migrations: data::PageMigrations {
					migrations: hemem_statistics
						.page_migrations()
						.iter()
						.map(|(page_ptr, page_migrations)| {
							let migrations = page_migrations
								.iter()
								.map(move |page_migration| data::PageMigration {
									prev_mem_idx: page_migration.prev_mem_idx.map(MemIdx::to_usize),
									cur_mem_idx:  page_migration.cur_mem_idx.to_usize(),
									time:         page_migration.time,
								})
								.collect();

							(page_ptr.to_u64(), migrations)
						})
						.collect(),
				},
			},
		};

		let output_file = fs::File::create(output_path).context("Unable to create output file")?;
		serde_json::to_writer(output_file, &data).context("Unable to write to output file")?;
	}

	Ok(())
}
