//! Creates graphs from `ftmemsim`'s output

// Modules
mod args;

// Imports
use {
	anyhow::Context,
	args::Args,
	clap::Parser,
	ftmemsim_util::logger,
	itertools::Itertools,
	plotlib::{page::Page, repr::Plot, style::PointStyle, view::ContinuousView},
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
				.map(|(idx, page_location)| (page_location.page_ptr, idx))
				.collect::<std::collections::HashMap<_, _>>();

			// Then calculate the min/max time so we can normalize it to 0..1.
			// Note: We do this because the time values themselves don't matter, only
			//       the relative time.
			// TODO: Better defaults when empty?
			let (min_time, max_time) = page_locations
				.locations
				.iter()
				.map(|page_location| page_location.time)
				.minmax()
				.into_option()
				.unwrap_or((0, 1));

			// And calculate the points to display
			let points = page_locations
				.locations
				.iter()
				.map(|page_location| {
					(
						(page_location.time - min_time) as f64 / (max_time - min_time) as f64,
						*page_ptr_idxs
							.get(&page_location.page_ptr)
							.expect("Page ptr had no index") as f64,
					)
				})
				.collect::<Vec<_>>();

			// Then build the plot and render it
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
	}

	Ok(())
}
