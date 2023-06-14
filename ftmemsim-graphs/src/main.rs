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
	gnuplot::{AutoOption, AxesCommon, FillRegionType, PlotOption},
	gzp::par::decompress::ParDecompress,
	itertools::Itertools,
	palette::{LinSrgb, Mix},
	std::{
		collections::{BTreeMap, HashMap, VecDeque},
		fs,
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
		args::SubCmd::PageMigrations(cmd_args) => self::draw_page_migrations(&cmd_args)?,
		args::SubCmd::PageMigrationsHist(cmd_args) => self::draw_page_migrations_hist(cmd_args)?,
		args::SubCmd::PageMigrationsHistMultiple(cmd_args) => self::draw_page_migrations_hist_multiple(cmd_args)?,
		args::SubCmd::PageLocation(cmd_args) => self::draw_page_location(cmd_args)?,
		args::SubCmd::PageTemperatureDensity(cmd_args) => self::draw_page_temperature_density(cmd_args)?,
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

	// TODO: 0, 1 defaults are weird: We don't actually use them
	let min_time = data.time_span.as_ref().map_or(0, |range| range.start);
	let max_time = data.time_span.as_ref().map_or(1, |range| range.end - 1);

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
				x: (page_migration.time - min_time) as f64 / (max_time - min_time) as f64,
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
			PlotOption::PointSize(2.0 * cmd_args.point_size),
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
			.context("Config had less memories than input file")?;
		let cur_mem = config
			.hemem
			.memories
			.get(cur_mem_idx)
			.context("Config had less memories than input file")?;

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
		])
		.set_x_range(AutoOption::Fix(0.0), AutoOption::Fix(data.len() as f64))
		.set_y_range(AutoOption::Fix(0.0), AutoOption::Auto)
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
		]);
	}

	// Finally finish building the graph
	fg_axes2d
		.set_x_range(AutoOption::Fix(0.0), AutoOption::Auto)
		.set_y_range(AutoOption::Fix(0.0), AutoOption::Auto)
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

	// TODO: 0, 1 defaults are weird: We don't actually use them
	let min_time = data.time_span.as_ref().map_or(0, |range| range.start);
	let max_time = data.time_span.as_ref().map_or(1, |range| range.end - 1);

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
					x: (page_access.time - min_time) as f64 / (max_time - min_time) as f64,
					y: *page_ptr_idxs.get(&page_access.page_ptr).expect("Page ptr had no index"),
				})
				.collect::<Vec<_>>();
			(mem_idx, points)
		})
		.collect::<BTreeMap<_, _>>();

	// Finally create and save the plot
	let mut fg = gnuplot::Figure::new();
	let axes_2d = fg.axes2d();

	for (&mem_idx, points) in &all_points {
		let mem = config
			.hemem
			.memories
			.get(mem_idx)
			.context("Config had less memories than input file")?;

		let color_progress = mem_idx as f64 / (all_points.len() as f64 - 1.0);
		let color = LinSrgb::new(0.0, 1.0, 0.0).mix(LinSrgb::new(1.0, 0.0, 0.0), color_progress);
		let color = format!("#{:x}", color.into_format::<u8>());

		// Note: Since we're drawing back-to-front, we need the first points
		//       to be larger than the last.
		//       We also never hit 0 here due to `mem_idx` < `all_points.len()`.
		let point_size_progress = 1.0 - mem_idx as f64 / all_points.len() as f64;
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

fn draw_page_temperature_density(cmd_args: args::PageTemperatureDensity) -> Result<(), anyhow::Error> {
	// Parse the input file
	let data = self::read_data(&cmd_args.input_file)?;

	// TODO: 0, 1 defaults are weird: We don't actually use them
	let min_time = data.time_span.as_ref().map_or(0, |range| range.start);
	let max_time = data.time_span.as_ref().map_or(1, |range| range.end - 1);

	// Then index the page pointers.
	let page_ptr_idxs = self::page_ptr_idxs(&data);

	// Get all the points
	let mut cur_temps = HashMap::<u64, f64>::new();
	let points = data
		.hemem
		.page_accesses
		.accesses
		.iter()
		.group_by(|page_access| page_access.page_ptr)
		.into_iter()
		.map(|(page_ptr, page_accesses)| {
			// Note: The page accesses will already be sorted by time
			let mut temps = page_accesses
				.map(|page_access| {
					let cur_temp = cur_temps.entry(page_ptr).or_insert(0.0);
					match page_access.kind {
						ftmemsim::data::PageAccessKind::Read => *cur_temp += cmd_args.temp_read_weight,
						ftmemsim::data::PageAccessKind::Write => *cur_temp += cmd_args.temp_write_weight,
					};

					let time = (page_access.time - min_time) as f64 / (max_time - min_time) as f64;
					(time, *cur_temp)
				})
				.collect::<VecDeque<_>>();

			// Add a temperature at the start and end to ensure that all rectangles are drawn from both ends
			let last_temp = temps.back().map_or(0.0, |&(_, temp)| temp);
			temps.push_front((0.0, 0.0));
			temps.push_back((1.0, last_temp));

			(page_ptr, temps)
		})
		.collect::<BTreeMap<_, _>>();

	let max_temp = points
		.values()
		.flat_map(|temps| temps.iter().map(|&(_, temp)| temp))
		.max_by(f64::total_cmp)
		.unwrap_or(0.0);

	// Finally create and save the plot
	let mut fg = gnuplot::Figure::new();
	let fg_axes2d: &mut gnuplot::Axes2D = fg.axes2d();
	for (&page_ptr, temps) in &points {
		let page_ptr_idx = *page_ptr_idxs.get(&page_ptr).expect("Page ptr had no index");

		for ((prev_time, prev_temp), (cur_time, cur_temp)) in temps.iter().copied().tuple_windows() {
			let progress = (prev_temp + cur_temp) / (2.0 * max_temp);
			let progress = progress.powf(cmd_args.temp_exponent);

			let color = LinSrgb::new(1.0, 0.0, 0.0).mix(LinSrgb::new(0.0, 1.0, 0.0), progress);
			let color = format!("#{:x}", color.into_format::<u8>());

			fg_axes2d.fill_between([prev_time, cur_time], [page_ptr_idx; 2], [page_ptr_idx + 1; 2], &[
				PlotOption::Color(&color),
				PlotOption::FillAlpha(1.0),
				PlotOption::FillRegion(FillRegionType::Below),
			]);
		}
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
