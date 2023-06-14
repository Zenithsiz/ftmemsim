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
	#[clap(name = "page-migrations")]
	PageMigrations(PageMigrations),

	// TODO: This is no longer a histogram, rename it?
	#[clap(name = "page-migrations-hist")]
	PageMigrationsHist(PageMigrationsHist),

	// TODO: This is no longer a histogram, rename it?
	#[clap(name = "page-migrations-hist-multiple")]
	PageMigrationsHistMultiple(PageMigrationsHistMultiple),

	#[clap(name = "page-location")]
	PageLocation(PageLocation),

	#[clap(name = "page-temperature-density")]
	PageTemperatureDensity(PageTemperatureDensity),
}

/// Creates a graph for page migrations
#[derive(Debug, clap::Args)]
pub struct PageMigrations {
	/// Input
	pub input_file: PathBuf,

	/// Config file
	#[clap(long = "config")]
	pub config_file: PathBuf,

	/// Output
	#[clap(flatten)]
	pub output: Output,

	/// Point size
	#[clap(long = "point-size", default_value_t = 0.5)]
	pub point_size: f64,
}

/// Creates a histogram of page migrations
#[derive(Debug, clap::Args)]
pub struct PageMigrationsHist {
	/// Input
	pub input_file: PathBuf,

	/// Output
	#[clap(flatten)]
	pub output: Output,
}

/// Creates a histogram of page migrations from multiple data
#[derive(Debug, clap::Args)]
pub struct PageMigrationsHistMultiple {
	/// Input files
	pub input_files: Vec<PathBuf>,

	/// Output
	#[clap(flatten)]
	pub output: Output,
}


/// Page location
#[derive(Debug, clap::Args)]
pub struct PageLocation {
	/// Input
	pub input_file: PathBuf,

	/// Config file
	#[clap(long = "config")]
	pub config_file: PathBuf,

	/// Output
	#[clap(flatten)]
	pub output: Output,

	/// Point size
	#[clap(long = "point-size", default_value_t = 0.5)]
	pub point_size: f64,
}


/// Page temperature density
#[derive(Debug, clap::Args)]
pub struct PageTemperatureDensity {
	/// Input
	pub input_file: PathBuf,

	/// Output
	#[clap(flatten)]
	pub output: Output,

	/// Temperature exponent
	#[clap(long = "temp-exponent", default_value_t = 1.0)]
	pub temp_exponent: f64,

	/// Read weight for temperature
	#[clap(long = "temp-read-weight", default_value_t = 1.0)]
	#[clap(allow_hyphen_values = true)]
	pub temp_read_weight: f64,

	/// Write weight for temperature
	#[clap(long = "temp-write-weight", default_value_t = 2.0)]
	#[clap(allow_hyphen_values = true)]
	pub temp_write_weight: f64,
}

/// Output
#[derive(Debug, clap::Args)]
pub struct Output {
	/// Interactive mode
	#[clap(long = "interactive")]
	pub interactive: bool,

	/// Output file
	#[clap(short = 'o', long = "output", group = "output-file")]
	pub file: Option<PathBuf>,

	/// Output file width
	#[clap(long = "output-width", requires = "output-file", default_value_t = 640)]
	pub width: u32,

	/// Output file height
	#[clap(long = "output-height", requires = "output-file", default_value_t = 480)]
	pub height: u32,
}
