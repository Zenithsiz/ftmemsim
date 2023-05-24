//! Creates graphs from `ftmemsim`'s output

// Features
#![feature(lint_reasons)]

use plotlib::style::{LineJoin, LineStyle};

// Modules
mod args;

// Imports
use {
	anyhow::Context,
	args::Args,
	clap::Parser,
	ftmemsim_util::logger,
	itertools::Itertools,
	plotlib::{
		page::Page,
		repr::{Histogram, HistogramBins, Plot},
		style::{BoxStyle, PointStyle},
		view::ContinuousView,
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
		args::SubCmd::PageLocations {
			input_file,
			output_file,
			point_size,
			width,
			height,
			x_tick_marks,
			y_tick_marks,
			point_color,
		} => {
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
			let points = page_locations
				.locations
				.iter()
				.flat_map(|(page_ptr, page_locations)| {
					page_locations.iter().map(|page_location| {
						(
							(page_location.time - min_time) as f64 / (max_time - min_time) as f64,
							*page_ptr_idxs.get(page_ptr).expect("Page ptr had no index") as f64,
						)
					})
				})
				.collect::<Vec<_>>();

			// Finally build the plot and render it
			let plot = Plot::new(points).point_style(PointStyle::new().size(point_size).colour(point_color));

			let view = ContinuousView::new()
				.add(plot)
				.x_max_ticks(x_tick_marks)
				.y_max_ticks(y_tick_marks)
				.x_label("Time")
				.y_label("Page (Indexed)");

			Page::single(&view)
				.dimensions(width, height)
				.save(output_file)
				.map_err(|err| anyhow::anyhow!("Unable to save output file: {err:?}"))?;
		},

		// TODO: Allow customization for all of the parameters here?
		args::SubCmd::PageMigrations {
			input_file,
			output_file,
		} => {
			// Parse the page locations
			let page_locations = {
				let page_locations_file = std::fs::File::open(input_file).context("Unable to open input file")?;
				serde_json::from_reader::<_, ftmemsim_util::PageLocations>(page_locations_file)
					.context("Unable to parse input file")?
			};

			let max_migrations = page_locations
				.locations
				.values()
				.map(|page_locations| page_locations.len())
				.max()
				.unwrap_or(0);

			// Build the data
			let data = page_locations
				.locations
				.values()
				.map(|page_locations| page_locations.len() as f64)
				.collect::<Vec<_>>();

			// Finally build the histogram and render it
			let hist = Histogram::from_slice(&data, HistogramBins::Count(20.min(max_migrations) - 1))
				.style(&BoxStyle::new().fill("burlywood"));

			let view = ContinuousView::new()
				.add(hist)
				.x_max_ticks(20.min(max_migrations))
				.y_max_ticks(6)
				.x_label("Migrations")
				.y_label("Occurrences");

			Page::single(&view)
				.dimensions(640, 480)
				.save(output_file)
				.map_err(|err| anyhow::anyhow!("Unable to save output file: {err:?}"))?;
		},
		args::SubCmd::PageTemperature {
			input_file,
			output_file,
		} => {
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

			let mut temp_cur_average = 0.0;
			let temp_points = page_accesses
				.accesses
				.iter()
				.enumerate()
				.map(|(idx, page_access)| {
					temp_cur_average += page_access.cur_temp as f64;
					(
						(page_access.time - min_time) as f64 / (max_time - min_time) as f64,
						temp_cur_average / (idx as f64 + 1.0),
					)
				})
				.collect::<Vec<_>>();

			// Finally build the plot and render it
			let temp_plot = Plot::new(temp_points)
				.line_style(LineStyle::new().width(1.0).colour("#000000").linejoin(LineJoin::Round));

			let view = ContinuousView::new()
				.add(temp_plot)
				.x_max_ticks(6)
				.y_max_ticks(6)
				.x_label("Time")
				.y_label("Temperature");

			Page::single(&view)
				.dimensions(640, 480)
				.save(output_file)
				.map_err(|err| anyhow::anyhow!("Unable to save output file: {err:?}"))?;
		},
	}

	Ok(())
}
