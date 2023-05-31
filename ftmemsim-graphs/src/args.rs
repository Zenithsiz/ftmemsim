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

	/// Sub-command
	#[command(subcommand)]
	pub sub_cmd: SubCmd,
}

/// Sub-command
#[expect(clippy::enum_variant_names)] // It's a coincidence, we'll add more varied names
#[derive(Debug, clap::Subcommand)]
pub enum SubCmd {
	/// Creates a graph for page migrations
	#[clap(name = "page-migrations")]
	PageMigrations {
		/// Input
		input_file: PathBuf,

		/// Output
		#[clap(flatten)]
		output: ArgsOutputFile,
	},

	/// Creates a histogram of page migrations
	#[clap(name = "page-migrations-hist")]
	PageMigrationsHist {
		/// Input
		input_file: PathBuf,

		/// Output
		#[clap(flatten)]
		output: ArgsOutputFile,
	},

	/// Creates a histogram of page migrations from multiple data
	#[clap(name = "page-migrations-hist-multiple")]
	PageMigrationsHistMultiple {
		/// Input files
		input_files: Vec<PathBuf>,

		/// Output
		#[clap(flatten)]
		output: ArgsOutputFile,
	},

	/// Page temperature
	#[clap(name = "page-temperature")]
	PageTemperature {
		/// Input
		input_file: PathBuf,

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
