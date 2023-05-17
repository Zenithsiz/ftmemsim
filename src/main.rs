//! Filipe's Tiered Memory Simulator (`ftmemsim`)

// Features
#![feature(decl_macro, lint_reasons)]

// Modules
mod args;
mod logger;
mod pin_trace;
mod util;

// Imports
use {
	self::{args::Args, pin_trace::PinTrace},
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


	Ok(())
}
