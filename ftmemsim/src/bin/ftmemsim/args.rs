//! Arguments

// Imports
use std::path::PathBuf;

/// Arguments
#[derive(Debug)]
#[derive(clap::Parser)]
pub struct Args {
	/// Log file
	///
	/// Specifies a file to perform verbose logging to.
	/// You can use `RUST_LOG_FILE` to set filtering options
	#[clap(long = "log-file")]
	pub log_file: Option<PathBuf>,

	/// Whether to append to the log file
	#[clap(long = "log-file-append")]
	pub log_file_append: bool,

	/// Trace file
	pub trace_file: PathBuf,

	/// Config file
	#[clap(long = "config")]
	pub config_file: PathBuf,

	/// Output file
	#[clap(long = "output")]
	pub output_file: Option<PathBuf>,
}
