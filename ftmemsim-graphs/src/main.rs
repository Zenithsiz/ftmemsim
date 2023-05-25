//! Creates graphs from `ftmemsim`'s output

// Features
#![feature(lint_reasons)]

// Modules
mod args;

// Imports
use {
	anyhow::Context,
	args::Args,
	clap::Parser,
	ftmemsim_util::logger,
	gnuplot::{AutoOption, AxesCommon, DashType, FillRegionType, PlotOption},
	itertools::Itertools,
	std::{
		collections::{BTreeMap, HashMap},
		path::Path,
	},
};

fn main() -> Result<(), anyhow::Error> {
	// Get arguments
	let args = Args::parse();
	logger::pre_init::debug(format!("Args: {args:?}"));

	// Initialize logging
	logger::init(args.log_file.as_deref(), args.log_file_append);

	// Then check the sub-command
	match args.sub_cmd {
		args::SubCmd::PageLocations { input_file, output } => {
			// Parse the page locations
			let page_locations = {
				let page_locations_file = std::fs::File::open(input_file).context("Unable to open input file")?;
				serde_json::from_reader::<_, ftmemsim_util::PageLocations>(page_locations_file)
					.context("Unable to parse input file")?
			};

			// Then index the page pointers.
			// Note: We do this because the page pointers are very far away, value-wise, which
			//       causes them to display far away in the graph. Since the actual values of the
			//       pages don't matter to us, we just index them by order of appearance.
			let page_ptr_idxs = page_locations
				.locations
				.iter()
				.enumerate()
				.map(|(idx, (page_ptr, _))| (page_ptr, idx))
				.collect::<std::collections::HashMap<_, _>>();

			// Then calculate the min/max time so we can normalize it to 0..1.
			// Note: We do this because the time values themselves don't matter, only
			//       the relative time.
			// TODO: Better defaults when empty?
			let (min_time, max_time) = page_locations
				.locations
				.iter()
				.flat_map(|(_, page_locations)| page_locations.iter().map(|page_location| page_location.time))
				.minmax()
				.into_option()
				.unwrap_or((0, 1));

			// And calculate the points to display
			struct Point {
				x: f64,
				y: usize,
			}
			let points_alloc = page_locations
				.locations
				.iter()
				.flat_map(|(page_ptr, page_locations)| {
					page_locations.first().map(|page_location| Point {
						x: (page_location.time - min_time) as f64 / (max_time - min_time) as f64,
						y: *page_ptr_idxs.get(page_ptr).expect("Page ptr had no index"),
					})
				})
				.collect::<Vec<_>>();
			let points_migration = page_locations
				.locations
				.iter()
				.flat_map(|(page_ptr, page_locations)| {
					page_locations.iter().skip(1).map(|page_location| Point {
						x: (page_location.time - min_time) as f64 / (max_time - min_time) as f64,
						y: *page_ptr_idxs.get(page_ptr).expect("Page ptr had no index"),
					})
				})
				.collect::<Vec<_>>();

			// Finally create and save the plot
			let mut fg = gnuplot::Figure::new();
			fg.axes2d()
				.points(points_alloc.iter().map(|p| p.x), points_alloc.iter().map(|p| p.y), &[
					PlotOption::Caption("Page locations (Allocation)"),
					PlotOption::Color("red"),
					PlotOption::PointSymbol('O'),
					PlotOption::PointSize(0.2),
				])
				.points(
					points_migration.iter().map(|p| p.x),
					points_migration.iter().map(|p| p.y),
					&[
						PlotOption::Caption("Page locations (Migrations)"),
						PlotOption::Color("black"),
						PlotOption::PointSymbol('O'),
						PlotOption::PointSize(0.2),
					],
				)
				.set_x_label("Time (normalized)", &[])
				.set_y_label("Page (indexed)", &[])
				.set_x_range(AutoOption::Fix(0.0), AutoOption::Fix(1.0));

			self::save_plot(&output.file, &mut fg, output.width, output.height).context("Unable to save plot")?;
		},

		// TODO: Allow customization for all of the parameters here?
		args::SubCmd::PageMigrations { input_file, output } => {
			// Parse the page locations
			let page_locations = {
				let page_locations_file = std::fs::File::open(input_file).context("Unable to open input file")?;
				serde_json::from_reader::<_, ftmemsim_util::PageLocations>(page_locations_file)
					.context("Unable to parse input file")?
			};

			// Build the data
			// Note: `-1` since the initial location doesn't count as a migration
			let data = page_locations
				.locations
				.values()
				.map(|page_locations| page_locations.len() - 1)
				.counts()
				.into_iter()
				.collect::<BTreeMap<_, _>>();

			// Finally create and save the plot
			let mut fg = gnuplot::Figure::new();
			fg.axes2d()
				.boxes_set_width(data.keys(), data.values(), (0..data.len()).map(|_| 0.8), &[
					PlotOption::Caption("Migration count"),
					PlotOption::Color("black"),
				])
				.set_y_log(Some(10.0))
				.set_x_ticks(Some((AutoOption::Fix(1.0), 0)), &[], &[])
				.set_y_range(AutoOption::Fix(0.0), AutoOption::Auto)
				.set_x_label("Migrations", &[])
				.set_y_label("Count", &[]);

			self::save_plot(&output.file, &mut fg, output.width, output.height).context("Unable to save plot")?;
		},
		args::SubCmd::PageTemperature { input_file, output } => {
			// Parse the page accesses
			let page_accesses = {
				let page_accesses_file = std::fs::File::open(input_file).context("Unable to open input file")?;
				serde_json::from_reader::<_, ftmemsim_util::PageAccesses>(page_accesses_file)
					.context("Unable to parse input file")?
			};

			let (min_time, max_time) = page_accesses
				.accesses
				.iter()
				.map(|page_access| page_access.time)
				.minmax()
				.into_option()
				.unwrap_or((0, 1));

			// Get all the points
			struct Point {
				x:             f64,
				y_avg:         f64,
				y_err:         f64,
				global_cooled: bool,
			}
			let mut cur_temps = HashMap::new();
			let points = page_accesses
				.accesses
				.iter()
				.map(|page_access| {
					// Update the temperatures
					// TODO: Optimize global cooling?
					*cur_temps.entry(page_access.page_ptr).or_insert(0) = page_access.prev_temp;
					if page_access.caused_cooling {
						for temp in cur_temps.values_mut() {
							*temp /= 2;
						}
					}

					// TODO: Optimize this with a moving average?
					let average_temp = &cur_temps
						.values()
						.map(|&temp| temp as f64)
						.collect::<average::Variance>();
					Point {
						x:             (page_access.time - min_time) as f64 / (max_time - min_time) as f64,
						y_avg:         average_temp.mean(),
						y_err:         average_temp.error(),
						global_cooled: page_access.caused_cooling,
					}
				})
				.collect::<Vec<_>>();

			let max_y = points.iter().map(|p| p.y_avg).max_by(f64::total_cmp).unwrap_or(0.0);

			// Finally create and save the plot
			let mut fg = gnuplot::Figure::new();
			fg.axes2d()
				.boxes_set_width(
					points.iter().map(|p| p.x),
					points.iter().map(|p| if p.global_cooled { max_y } else { 0.0 }),
					(0..points.len()).map(|_| 0.1 / (points.len() as f64)),
					&[
						PlotOption::Caption("Global cooling"),
						PlotOption::Color("red"),
						PlotOption::LineWidth(0.1),
					],
				)
				.lines(points.iter().map(|p| p.x), points.iter().map(|p| p.y_avg), &[
					PlotOption::Caption("Page locations (Avg)"),
					PlotOption::Color("black"),
					PlotOption::LineStyle(DashType::Solid),
					PlotOption::LineWidth(1.0),
				])
				.fill_between(
					points.iter().map(|p| p.x),
					points.iter().map(|p| p.y_avg - p.y_err),
					points.iter().map(|p| p.y_avg + p.y_err),
					&[
						PlotOption::Caption("Page locations (Error)"),
						PlotOption::Color("green"),
						PlotOption::FillAlpha(0.3),
						PlotOption::FillRegion(FillRegionType::Below),
					],
				)
				//.set_y_log(Some(10.0))
				.set_x_label("Time (normalized)", &[])
				.set_y_label("Temperature", &[])
				.set_x_range(AutoOption::Fix(0.0), AutoOption::Fix(1.0))
				.set_y_range(AutoOption::Fix(0.0), AutoOption::Fix(max_y));

			self::save_plot(&output.file, &mut fg, output.width, output.height).context("Unable to save plot")?;
		},
	}

	Ok(())
}

/// Saves the plot `fg` to `output_file`, depending on it's extension
fn save_plot(output_file: &Path, fg: &mut gnuplot::Figure, width_px: u32, height_px: u32) -> Result<(), anyhow::Error> {
	match output_file
		.extension()
		.context("Output file had no extension")?
		.to_str()
		.context("Output file extension was non-utf8")?
		.to_ascii_lowercase()
		.as_str()
	{
		"png" => fg
			.save_to_png(output_file, width_px, height_px)
			.context("Unable to save output file as png")?,

		"svg" => fg
			.save_to_svg(output_file, width_px, height_px)
			.context("Unable to save output file as svg")?,

		"html" => fg
			.save_to_canvas(output_file, width_px, height_px)
			.context("Unable to save output file as html canvas")?,

		ext => anyhow::bail!("Unknown extension: {ext:?}"),
	}

	Ok(())
}
