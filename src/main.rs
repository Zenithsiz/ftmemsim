//! Filipe's Tiered Memory Simulator (`ftmemsim`)

// Features
#![feature(decl_macro)]

// Modules
mod args;
mod logger;

// Imports
use {crate::args::Args, clap::Parser};

fn main() {
	// Get arguments
	let args = Args::parse();
	logger::pre_init::debug(format!("Args: {args:?}"));

	// Initialize logging
	logger::init(args.log_file.as_deref(), args.log_file_append);
}
