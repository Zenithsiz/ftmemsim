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
#[derive(Debug, clap::Subcommand)]
pub enum SubCmd {
	/// Creates a graph for page locations
	///
	/// Uses the `page_locations.yaml` data
	#[clap(name = "page-locations")]
	PageLocations {
		/// Input
		input_file: PathBuf,

		/// Output
		#[clap(short = 'o', long = "output")]
		output_file: PathBuf,

		/// Point size
		#[clap(long = "point-size", default_value_t = 1.0)]
		point_size: f32,

		/// Width
		#[clap(long = "width", default_value_t = 640)]
		width: u32,

		/// Height
		#[clap(long = "height", default_value_t = 480)]
		height: u32,

		/// Tick marks for x axis
		#[clap(long = "x-tick-marks", default_value_t = 6)]
		x_tick_marks: usize,

		/// Tick marks for y axis
		#[clap(long = "y-tick-marks", default_value_t = 6)]
		y_tick_marks: usize,

		/// Point color
		#[clap(long = "point-color", default_value_t = { "#000000".to_owned() })]
		point_color: String,
	},

	/// Creates a histogram of page migrations
	///
	/// Uses the `page_locations.yaml` data
	#[clap(name = "page-migrations")]
	PageMigrations {
		/// Input
		input_file: PathBuf,

		/// Output
		#[clap(short = 'o', long = "output")]
		output_file: PathBuf,
	},
}
