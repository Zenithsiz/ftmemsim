//! Creates graphs from `ftmemsim`'s output

// Features
#![feature(lint_reasons, array_windows, float_minimum_maximum)]

// Modules
mod args;

// Imports
use {
	anyhow::Context,
	args::Args,
	clap::Parser,
	ftmemsim_util::logger,
	gnuplot::{AutoOption, AxesCommon, PlotOption},
	gzp::par::decompress::ParDecompress,
	itertools::Itertools,
	palette::{LinSrgb, Mix},
	std::{collections::BTreeMap, fs, path::Path},
};

fn main() -> Result<(), anyhow::Error> {
	// Get arguments
	let args = Args::parse();
	logger::pre_init::debug(format!("Args: {args:?}"));

	// Initialize logging
	logger::init(args.log_file.as_deref(), args.log_file_append);

	// Then check the sub-command
	match args.sub_cmd {
		args::SubCmd::PageMigrations(cmd_args) => self::draw_page_migrations(&cmd_args)?,
		args::SubCmd::PageMigrationsHist(cmd_args) => self::draw_page_migrations_hist(cmd_args)?,
		args::SubCmd::PageMigrationsHistMultiple(cmd_args) => self::draw_page_migrations_hist_multiple(cmd_args)?,
		args::SubCmd::PageLocation(cmd_args) => self::draw_page_location(cmd_args)?,
		args::SubCmd::PageTemperature(cmd_args) => self::draw_page_temperature(cmd_args)?,
		args::SubCmd::PageTemperatureAvg(cmd_args) => self::draw_page_temperature_avg(cmd_args)?,
		args::SubCmd::MemoryOccupancy(cmd_args) => self::draw_memory_occupancy(cmd_args)?,
	}

	Ok(())
}

/// Draws the page migrations plot
fn draw_page_migrations(cmd_args: &args::PageMigrations) -> Result<(), anyhow::Error> {
	// Parse the config and input file
	let config = self::read_config(&cmd_args.config_file)
		.with_context(|| format!("Unable to read config file: {:?}", cmd_args.config_file))?;
	let data = self::read_data(&cmd_args.input_file)
		.with_context(|| format!("Unable to read data file: {:?}", cmd_args.input_file))?;

	// Then index the page pointers.
	let page_ptr_idxs = self::page_ptr_idxs(&data);

	// And calculate the points to display
	struct Point {
		x: f64,
		y: usize,
	}

	// Note: We use `BTreeMap` here to ensure a consistent order across runs (for creating gifs)
	let mut points_alloc = vec![];
	let mut points_migrations_all = BTreeMap::<(usize, usize), Vec<Point>>::new();
	for (page_ptr, page_migrations) in &data.hemem.page_migrations.migrations {
		for page_migration in page_migrations {
			// Get the points to add the point to.
			// Note: If we didn't have a previous memory index, we use the allocations bucket, else
			//       we grab corresponding to the `(prev, cur)` migration pair
			let points = match page_migration.prev_mem_idx {
				Some(prev_mem_idx) => points_migrations_all
					.entry((prev_mem_idx, page_migration.cur_mem_idx))
					.or_default(),
				None => &mut points_alloc,
			};

			points.push(Point {
				x: self::date_normalized(page_migration.time, &data),
				y: *page_ptr_idxs.get(page_ptr).expect("Page ptr had no index"),
			});
		}
	}

	// Finally create and save the plot
	let mut fg = gnuplot::Figure::new();
	let fg_axes2d = fg
		.axes2d()
		.points(points_alloc.iter().map(|p| p.x), points_alloc.iter().map(|p| p.y), &[
			PlotOption::Caption("Page allocations"),
			PlotOption::Color("blue"),
			PlotOption::PointSymbol('O'),
			PlotOption::PointSize(1.5 * cmd_args.point_size),
		]);

	for (migration_idx, (&(prev_mem_idx, cur_mem_idx), points_migrations)) in points_migrations_all.iter().enumerate() {
		// Calculate the color for these migrations
		// Note: We use the red to dictate the current memory and green for the previous,
		//       this is to a greener color indicates a positive migration, while a redder
		//       color a negative migration
		let max_mem_idx = config.hemem.memories.len();
		let color = LinSrgb::new(
			cur_mem_idx as f64 / max_mem_idx as f64,
			prev_mem_idx as f64 / max_mem_idx as f64,
			0.0,
		);
		let color = format!("#{:x}", color.into_format::<u8>());

		// Then get the memories (for then ames)
		let prev_mem = config
			.hemem
			.memories
			.get(prev_mem_idx)
			.expect("Config had less memories than input file");
		let cur_mem = config
			.hemem
			.memories
			.get(cur_mem_idx)
			.expect("Config had less memories than input file");

		// Note: Since we're drawing back-to-front, we need the first points
		//       to be larger than the last.
		//       We also never hit 0 here due to `migration_idx` < `points_migrations_all.len()`.
		let point_size_progress = 1.0 - migration_idx as f64 / points_migrations_all.len() as f64;
		let point_size = point_size_progress * cmd_args.point_size;

		fg_axes2d.points(
			points_migrations.iter().map(|p| p.x),
			points_migrations.iter().map(|p| p.y),
			&[
				PlotOption::Caption(&format!("Page migrations ({} to {})", prev_mem.name, cur_mem.name)),
				PlotOption::Color(&color),
				PlotOption::PointSymbol('O'),
				PlotOption::PointSize(point_size),
			],
		);
	}

	fg_axes2d
		.set_x_label("Time (normalized)", &[])
		.set_y_label("Page (indexed)", &[])
		.set_x_range(AutoOption::Fix(0.0), AutoOption::Fix(1.0))
		.set_y_range(AutoOption::Fix(0.0), AutoOption::Fix(page_ptr_idxs.len() as f64));

	// Then output the plot
	self::handle_output(&cmd_args.output, &mut fg).context("Unable to handle output")?;

	Ok(())
}

fn draw_page_migrations_hist(cmd_args: args::PageMigrationsHist) -> Result<(), anyhow::Error> {
	// Parse and build the data
	let data = self::read_data(&cmd_args.input_file)?;
	let data = self::page_migrations_hist_data(&data);

	// Finally create and save the plot
	let mut fg = gnuplot::Figure::new();
	fg.axes2d()
		.lines(0..data.len(), &data, &[
			PlotOption::Caption("Migration count"),
			PlotOption::Color("black"),
			PlotOption::LineWidth(cmd_args.line_width),
		])
		.set_x_log(Some(10.0))
		.set_y_log(Some(10.0))
		.set_x_range(AutoOption::Fix(1.0), AutoOption::Fix(data.len() as f64))
		.set_y_range(AutoOption::Fix(1.0), AutoOption::Auto)
		.set_x_label("Migrations (flattened)", &[])
		.set_y_label("Migrations", &[]);

	// Then output the plot
	self::handle_output(&cmd_args.output, &mut fg).context("Unable to handle output")?;

	Ok(())
}

fn draw_page_migrations_hist_multiple(cmd_args: args::PageMigrationsHistMultiple) -> Result<(), anyhow::Error> {
	// Create the figure
	// Note: We do this before parsing the data since we parse
	//       the files in parallel (with a limit, to not load too
	//       many at once)
	let mut fg = gnuplot::Figure::new();
	let fg_axes2d = fg.axes2d();

	// Then process all input files
	// TODO: Process them in parallel with `rayon`? Issue is that we need to add each
	//       plot in order so that the legend stays consistent.
	for (data_idx, input_file) in cmd_args.input_files.iter().enumerate() {
		// Parse and build the data
		let data = self::read_data(input_file).with_context(|| format!("Unable to read {input_file:?}"))?;
		let data = self::page_migrations_hist_data(&data);

		// Then render the lines
		let progress = data_idx as f64 / (cmd_args.input_files.len() as f64 - 1.0);

		let color = LinSrgb::new(1.0, 0.0, 0.0).mix(LinSrgb::new(0.0, 1.0, 0.0), progress);
		let color = format!("#{:x}", color.into_format::<u8>());

		fg_axes2d.lines(0..data.len(), &data, &[
			PlotOption::Caption(&format!("Migration count ({})", input_file.display())),
			PlotOption::Color(&color),
			PlotOption::LineWidth(cmd_args.line_width),
		]);
	}

	// Finally finish building the graph
	fg_axes2d
		.set_x_log(Some(10.0))
		.set_y_log(Some(10.0))
		.set_x_range(AutoOption::Fix(1.0), AutoOption::Auto)
		.set_y_range(AutoOption::Fix(1.0), AutoOption::Auto)
		.set_x_label("Migrations (flattened)", &[])
		.set_y_label("Migrations", &[]);

	// Then output the plot
	self::handle_output(&cmd_args.output, &mut fg).context("Unable to handle output")?;

	Ok(())
}

/// Draws the page location graph
fn draw_page_location(cmd_args: args::PageLocation) -> Result<(), anyhow::Error> {
	// Parse the config and input file
	let config = self::read_config(&cmd_args.config_file)
		.with_context(|| format!("Unable to read config file: {:?}", cmd_args.config_file))?;
	let data = self::read_data(&cmd_args.input_file)
		.with_context(|| format!("Unable to read data file: {:?}", cmd_args.input_file))?;

	// Then index the page pointers.
	let page_ptr_idxs = self::page_ptr_idxs(&data);

	// Get all the points
	struct Point {
		x: f64,
		y: usize,
	}
	let all_points = data
		.hemem
		.page_accesses
		.accesses
		.iter()
		.map(|page_access| (page_access.mem_idx, page_access))
		.into_group_map()
		.into_iter()
		.map(|(mem_idx, page_accesses)| {
			let points = page_accesses
				.into_iter()
				.map(|page_access| Point {
					x: self::date_normalized(page_access.time, &data),
					y: *page_ptr_idxs.get(&page_access.page_ptr).expect("Page ptr had no index"),
				})
				.collect::<Vec<_>>();
			(mem_idx, points)
		})
		.collect::<BTreeMap<_, _>>();

	// Finally create and save the plot
	let mut fg = gnuplot::Figure::new();
	let axes_2d = fg.axes2d();

	for (&mem_idx, points) in all_points.iter().rev() {
		let mem = config
			.hemem
			.memories
			.get(mem_idx)
			.context("Config had less memories than input file")?;

		let color_progress = match all_points.len() {
			1 => 0.0,
			_ => mem_idx as f64 / (all_points.len() as f64 - 1.0),
		};
		let color = LinSrgb::new(0.0, 1.0, 0.0).mix(LinSrgb::new(1.0, 0.0, 0.0), color_progress);
		let color = format!("#{:x}", color.into_format::<u8>());

		// Note: Since we're drawing back-to-front, we need the first points
		//       to be larger than the last.
		//       We also never hit 0 here due to `mem_idx` < `all_points.len()`.
		let point_size_progress = 1.0 - (all_points.len() as f64 - mem_idx as f64) / (1.0 + all_points.len() as f64);
		let point_size = point_size_progress * cmd_args.point_size;

		axes_2d.points(points.iter().map(|p| p.x), points.iter().map(|p| p.y), &[
			PlotOption::Caption(&format!("Page location {mem_idx:?} ({})", mem.name)),
			PlotOption::Color(&color),
			PlotOption::PointSymbol('O'),
			PlotOption::PointSize(point_size),
		]);
	}

	axes_2d
		.set_x_label("Time (normalized)", &[])
		.set_y_label("Page (indexed)", &[])
		.set_x_range(AutoOption::Fix(0.0), AutoOption::Fix(1.0))
		.set_y_range(AutoOption::Fix(0.0), AutoOption::Fix(page_ptr_idxs.len() as f64));

	// Then output the plot
	self::handle_output(&cmd_args.output, &mut fg).context("Unable to handle output")?;

	Ok(())
}

fn draw_page_temperature(cmd_args: args::PageTemperature) -> Result<(), anyhow::Error> {
	// Parse the input file
	let data = self::read_data(&cmd_args.input_file)?;

	// Then index the page pointers.
	let page_ptr_idxs = self::page_ptr_idxs(&data);

	let max_temp = data
		.hemem
		.page_accesses
		.accesses
		.iter()
		.map(|page_access| page_access.cur_temp)
		.max()
		.unwrap_or(0);

	// Get all the points
	struct Point {
		x: f64,
		y: usize,
	}
	let all_points = data
		.hemem
		.page_accesses
		.accesses
		.iter()
		.map(|page_access| {
			let point = Point {
				x: self::date_normalized(page_access.time, &data),
				y: *page_ptr_idxs.get(&page_access.page_ptr).expect("Page ptr had no index"),
			};

			// Note: We shrink the temperature into a `0..=255` range so we can
			//       assign a unique color to each temperature
			let temp_idx = (page_access.cur_temp as f64) / (max_temp as f64);
			let temp_idx = (temp_idx * 255.0) as u8;

			(temp_idx, point)
		})
		.into_group_map()
		.into_iter()
		.collect::<BTreeMap<_, _>>();

	// Finally create and save the plot
	let mut fg = gnuplot::Figure::new();
	let fg_axes2d = fg.axes2d();

	for (&temp_idx, points) in all_points.iter().rev() {
		let color_progress = temp_idx as f64 / 255.0;
		let color = LinSrgb::new(1.0, 0.0, 0.0).mix(LinSrgb::new(0.0, 1.0, 0.0), color_progress);
		let color = format!("#{:x}", color.into_format::<u8>());

		let point_size_progress = (1.0 + temp_idx as f64) / 256.0;
		let point_size = point_size_progress * cmd_args.point_size;

		fg_axes2d.points(points.iter().map(|p| p.x), points.iter().map(|p| p.y), &[
			PlotOption::Color(&color),
			PlotOption::PointSymbol('O'),
			PlotOption::PointSize(point_size),
		]);
	}

	// TODO: Get the color bar to show up?
	fg_axes2d
		.set_x_label("Time (normalized)", &[])
		.set_y_label("Page (indexed)", &[])
		.set_cb_label("Temperature", &[])
		.set_x_range(AutoOption::Fix(0.0), AutoOption::Fix(1.0))
		.set_y_range(AutoOption::Fix(0.0), AutoOption::Fix(page_ptr_idxs.len() as f64))
		.set_cb_range(AutoOption::Fix(0.0), AutoOption::Fix(max_temp as f64));

	// Then output the plot
	self::handle_output(&cmd_args.output, &mut fg).context("Unable to handle output")?;

	Ok(())
}

fn draw_page_temperature_avg(cmd_args: args::PageTemperatureAvg) -> Result<(), anyhow::Error> {
	// Parse the input file
	let data = self::read_data(&cmd_args.input_file)?;

	// Then index the page pointers.
	let page_ptr_idxs = self::page_ptr_idxs(&data);

	// Get all the points
	struct Point {
		page_indexed: usize,
		temp_avg:     f64,
		temp_err:     f64,
	}
	let points = data
		.hemem
		.page_accesses
		.accesses
		.iter()
		.map(|page_access| (page_access.page_ptr, page_access))
		.into_group_map()
		.into_iter()
		.map(|(page_ptr, page_access)| {
			let temp_mean = page_access
				.iter()
				.map(|page_access| page_access.cur_temp as f64)
				.collect::<average::Variance>();

			Point {
				page_indexed: *page_ptr_idxs.get(&page_ptr).expect("Page ptr had no index"),
				temp_avg:     temp_mean.mean(),
				temp_err:     temp_mean.error(),
			}
		})
		.sorted_by_key(|p| p.page_indexed)
		.collect::<Vec<_>>();

	let max_avg_temp = points
		.iter()
		.map(|p| p.temp_avg + p.temp_err)
		.max_by(f64::total_cmp)
		.unwrap_or(0.0);

	// Finally create and save the plot
	// TODO: Replace with bars?
	let mut fg = gnuplot::Figure::new();
	fg.axes2d()
		.x_error_lines(
			points.iter().map(|p| p.temp_avg),
			points.iter().map(|p| p.page_indexed),
			points.iter().map(|p| p.temp_err),
			&[
				PlotOption::Caption("Page temperature (average)"),
				PlotOption::Color("black"),
				PlotOption::LineWidth(cmd_args.line_width),
			],
		)
		.set_x_label("Temperature", &[])
		.set_y_label("Page (indexed)", &[])
		.set_x_range(AutoOption::Fix(0.0), AutoOption::Fix(max_avg_temp))
		.set_y_range(AutoOption::Fix(0.0), AutoOption::Fix(page_ptr_idxs.len() as f64));

	// Then output the plot
	self::handle_output(&cmd_args.output, &mut fg).context("Unable to handle output")?;

	Ok(())
}

fn draw_memory_occupancy(cmd_args: args::MemoryOccupancy) -> Result<(), anyhow::Error> {
	// Parse the config and input file
	let config = self::read_config(&cmd_args.config_file)
		.with_context(|| format!("Unable to read config file: {:?}", cmd_args.config_file))?;
	let data = self::read_data(&cmd_args.input_file)
		.with_context(|| format!("Unable to read data file: {:?}", cmd_args.input_file))?;

	// Calculate all the occupancies over time
	let mut memories_occupancy = (0..config.hemem.memories.len())
		.map(|mem_idx| (mem_idx, 0_usize))
		.collect::<BTreeMap<_, _>>();

	// Note: We sort and group all migrations by time, so we can process each "time step"
	//       all at once, then only output the final occupancies. This is to avoid having
	//       occupancies go above the capacity temporarily within one time step.
	let occupancies = data
		.hemem
		.page_migrations
		.migrations
		.values()
		.flatten()
		.sorted_by_key(|migration| migration.time)
		.group_by(|migration| migration.time)
		.into_iter()
		.flat_map(|(time, migrations)| {
			// Updates the occupancy of a memory by `delta`.
			// Panics if the occupancy would be negative or above `usize::MAX`.
			let mut update_occupancy = |mem_idx, delta| {
				let occupancy = memories_occupancy
					.get_mut(&mem_idx)
					.expect("Config had less memories than input file");

				*occupancy = occupancy
					.checked_add_signed(delta)
					.expect("Memory occupancy was negative / above `usize::MAX`");

				(mem_idx, (self::date_normalized(time, &data), *occupancy))
			};

			// Process all migrations in this time step
			// Note: We select the *last* unique value by reversing, then
			//       selecting the first unique value with `.unique_by`.
			migrations
				.flat_map(|migration| {
					let prev_mem_occupancy = migration.prev_mem_idx.map(|mem_idx| update_occupancy(mem_idx, -1));
					let cur_mem_occupancy = update_occupancy(migration.cur_mem_idx, 1);

					[prev_mem_occupancy, Some(cur_mem_occupancy)]
				})
				.flatten()
				.collect::<Vec<_>>()
				.into_iter()
				.rev()
				.unique_by(|&(mem_idx, _)| mem_idx)
		})
		.into_group_map()
		.into_iter()
		.collect::<BTreeMap<_, _>>();

	// Finally create the plot
	let mut fg = gnuplot::Figure::new();
	let fg_axes2d = fg.axes2d();

	for (mem_idx, occupancies) in occupancies {
		let color_progress = 1.0 - mem_idx as f64 / (config.hemem.memories.len() as f64 - 1.0);
		let color = LinSrgb::new(1.0, 0.0, 0.0).mix(LinSrgb::new(0.0, 1.0, 0.0), color_progress);
		let color = format!("#{:x}", color.into_format::<u8>());

		let mem = config
			.hemem
			.memories
			.get(mem_idx)
			.expect("Config had less memories than input file");

		fg_axes2d.lines(
			occupancies.iter().map(|(time, _)| time),
			occupancies.iter().map(|(_, occupancy)| occupancy),
			&[
				PlotOption::Caption(&format!("Memory {mem_idx:?} ({})", mem.name)),
				PlotOption::Color(&color),
				PlotOption::LineWidth(cmd_args.line_width),
			],
		);
	}

	fg_axes2d
		.set_x_label("Time (normalized)", &[])
		.set_y_label("Page occupancy", &[])
		.set_x_range(AutoOption::Fix(0.0), AutoOption::Fix(1.0))
		.set_y_range(AutoOption::Fix(0.0), AutoOption::Auto);

	// Then output the plot
	self::handle_output(&cmd_args.output, &mut fg).context("Unable to handle output")?;

	Ok(())
}

/// Calculates a date normalized
///
/// Panics if there is no time span.
fn date_normalized(time: u64, data: &ftmemsim::data::Data) -> f64 {
	let time_span = data.time_span.as_ref().expect("Data had no time span");
	let min_time = time_span.start;
	let max_time = time_span.end.checked_sub(1).expect("Max time was `0`");
	assert!(min_time < max_time, "Data duration was 0 or less");

	(time - min_time) as f64 / (max_time - min_time) as f64
}

/// Computes the data to use fr the `page-migrations-hist` graph
fn page_migrations_hist_data(data: &ftmemsim::data::Data) -> Vec<usize> {
	data.hemem
		.page_migrations
		.migrations
		.values()
		.map(|page_migrations| {
			// Note: `-1` since the initial migration doesn't count as a migration
			page_migrations.len() - 1
		})
		.counts()
		.into_iter()
		.flat_map(|(migrations_len, count)| {
			// Note: This simply repeats the `migrations_len` entry
			//       `count` times. Equivalent to `vec![migrations_len; count]`
			(0..count).map(move |_| migrations_len)
		})
		.sorted()
		.rev()
		.collect::<Vec<_>>()
}

/// Reads config from `config_file`
fn read_config(config_file: &Path) -> Result<ftmemsim::config::Config, anyhow::Error> {
	// Open the file
	let config_file = fs::File::open(config_file).context("Unable to open config file")?;

	// Then parse it
	let data =
		serde_json::from_reader::<_, ftmemsim::config::Config>(config_file).context("Unable to parse config file")?;

	Ok(data)
}

/// Reads data from `input_file`
fn read_data(input_file: &Path) -> Result<ftmemsim::data::Data, anyhow::Error> {
	// Open the file
	let data_file = std::fs::File::open(input_file).context("Unable to open input file")?;
	let mut data_file = ParDecompress::<gzp::deflate::Mgzip>::builder().from_reader(data_file);

	// Then parse it
	let data = bincode::decode_from_std_read::<ftmemsim::data::Data, _, _>(&mut data_file, bincode::config::standard())
		.context("Unable to parse input file")?;

	Ok(data)
}

/// Handles the plot output
///
/// Outputs `fg`, if `output.file` is `Some(_)`, then shows the plot if `output.interactive` is true
fn handle_output(output: &args::Output, fg: &mut gnuplot::Figure) -> Result<(), anyhow::Error> {
	// If we have an output file, output it to file
	if let Some(output_file) = &output.file {
		self::save_plot(output_file, fg, output.width, output.height).context("Unable to save plot")?
	}

	// Then show the interactive mode, if requested
	if output.interactive {
		fg.show().context("Unable to show plot")?;
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

/// Indexes all the page pointers in `data`.
///
///
/// We do this because the page pointers are very far away, value-wise, which
/// causes them to display far away in the graph. Since the actual values of the
/// pages don't matter to us, we just index, ordering by the page pointer value.
fn page_ptr_idxs(data: &ftmemsim::data::Data) -> BTreeMap<u64, usize> {
	// Note: We use `migrations` since each page is guaranteed to have at least 1 migration,
	//       the allocation.
	data.hemem
		.page_migrations
		.migrations
		.iter()
		.enumerate()
		.map(|(idx, (&page_ptr, _))| (page_ptr, idx))
		.collect::<BTreeMap<_, _>>()
}
