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

	/// Input
	pub input_file: PathBuf,

	/// Sub-command
	#[command(subcommand)]
	pub sub_cmd: SubCmd,
}

/// Sub-command
#[expect(clippy::enum_variant_names)] // It's a coincidence, we'll add more varied names
#[derive(Debug, clap::Subcommand)]
pub enum SubCmd {
	/// Creates a graph for page locations
	///
	/// Uses the `page_locations.yaml` data
	#[clap(name = "page-locations")]
	PageLocations {
		/// Output
		#[clap(flatten)]
		output: ArgsOutputFile,
	},

	/// Creates a histogram of page migrations
	///
	/// Uses the `page_locations.yaml` data
	#[clap(name = "page-migrations")]
	PageMigrations {
		/// Output
		#[clap(flatten)]
		output: ArgsOutputFile,
	},

	/// Page temperature
	///
	/// Uses the `page_accesses.yaml` data
	#[clap(name = "page-temperature")]
	PageTemperature {
		/// Output
		#[clap(flatten)]
		output: ArgsOutputFile,
	},
}

/// Output file
#[derive(Debug, clap::Args)]
pub struct ArgsOutputFile {
	/// Output file
	#[clap(short = 'o', long = "output")]
	pub file: PathBuf,

	/// Output file width
	#[clap(long = "output-width", default_value_t = 640)]
	pub width: u32,

	/// Output file height
	#[clap(long = "output-height", default_value_t = 480)]
	pub height: u32,
}
