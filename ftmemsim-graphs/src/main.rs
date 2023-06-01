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
	gnuplot::{AutoOption, AxesCommon, DashType, FillRegionType, PlotOption},
	gzp::par::decompress::ParDecompress,
	itertools::Itertools,
	palette::{LinSrgb, Mix},
	std::{
		cmp,
		collections::{BTreeMap, HashMap, VecDeque},
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
		args::SubCmd::PageMigrations { input_file, output } => {
			// Parse the input file
			let data = self::read_data(&input_file)?;

			// Then index the page pointers.
			// Note: We do this because the page pointers are very far away, value-wise, which
			//       causes them to display far away in the graph. Since the actual values of the
			//       pages don't matter to us, we just index them by order of appearance.
			let page_ptr_idxs = data
				.hemem
				.page_migrations
				.migrations
				.iter()
				.enumerate()
				.map(|(idx, (page_ptr, _))| (page_ptr, idx))
				.collect::<std::collections::HashMap<_, _>>();

			// Then calculate the min/max time so we can normalize it to 0..1.
			// Note: We do this because the time values themselves don't matter, only
			//       the relative time.
			// TODO: Better defaults when empty?
			let (min_time, max_time) = data
				.hemem
				.page_migrations
				.migrations
				.iter()
				.flat_map(|(_, page_migrations)| page_migrations.iter().map(|page_migration| page_migration.time))
				.minmax()
				.into_option()
				.unwrap_or((0, 1));

			// And calculate the points to display
			struct Point {
				x: f64,
				y: usize,
			}

			let mut points_alloc = vec![];
			let mut points_migration_to_faster = vec![];
			let mut points_migration_to_slower = vec![];
			for (page_ptr, page_migrations) in &data.hemem.page_migrations.migrations {
				for page_migration in page_migrations {
					let points = match page_migration.prev_mem_idx {
						// TODO: Care about how much it was moved once we have more than 2 memories?
						// Note: Like the hemem simulator, here we assume that a lower number means faster.
						Some(prev_mem_idx) => match page_migration.cur_mem_idx.cmp(&prev_mem_idx) {
							cmp::Ordering::Less => &mut points_migration_to_faster,
							cmp::Ordering::Equal => unreachable!("Memory was migration to the same memory"),
							cmp::Ordering::Greater => &mut points_migration_to_slower,
						},
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
			fg.axes2d()
				.points(points_alloc.iter().map(|p| p.x), points_alloc.iter().map(|p| p.y), &[
					PlotOption::Caption("Page migrations (Allocation)"),
					PlotOption::Color("black"),
					PlotOption::PointSymbol('O'),
					PlotOption::PointSize(0.2),
				])
				.points(
					points_migration_to_faster.iter().map(|p| p.x),
					points_migration_to_faster.iter().map(|p| p.y),
					&[
						PlotOption::Caption("Page migrations (Migrations to faster)"),
						PlotOption::Color("green"),
						PlotOption::PointSymbol('O'),
						PlotOption::PointSize(0.2),
					],
				)
				.points(
					points_migration_to_slower.iter().map(|p| p.x),
					points_migration_to_slower.iter().map(|p| p.y),
					&[
						PlotOption::Caption("Page migrations (Migrations to slower)"),
						PlotOption::Color("red"),
						PlotOption::PointSymbol('O'),
						PlotOption::PointSize(0.2),
					],
				)
				.set_x_label("Time (normalized)", &[])
				.set_y_label("Page (indexed)", &[])
				.set_x_range(AutoOption::Fix(0.0), AutoOption::Fix(1.0));

			self::save_plot(&output.file, &mut fg, output.width, output.height).context("Unable to save plot")?;
		},

		// TODO: This is no longer a histogram, rename it?
		args::SubCmd::PageMigrationsHist { input_file, output } => {
			// Parse and build the data
			let data = self::read_data(&input_file)?;
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

			self::save_plot(&output.file, &mut fg, output.width, output.height).context("Unable to save plot")?;
		},

		// TODO: This is no longer a histogram, rename it?
		args::SubCmd::PageMigrationsHistMultiple { input_files, output } => {
			// Create the figure
			// Note: We do this before parsing the data since we parse
			//       the files in parallel (with a limit, to not load too
			//       many at once)
			let mut fg = gnuplot::Figure::new();
			let fg_axes2d = fg.axes2d();

			// Then process all input files
			// TODO: Process them in parallel with `rayon`? Issue is that we need to add each
			//       plot in order so that the legend stays consistent.
			for (data_idx, input_file) in input_files.iter().enumerate() {
				// Parse and build the data
				let data = self::read_data(input_file).with_context(|| format!("Unable to read {input_file:?}"))?;
				let data = self::page_migrations_hist_data(&data);

				// Then render the lines
				let progress = data_idx as f64 / (input_files.len() as f64 - 1.0);

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

			self::save_plot(&output.file, &mut fg, output.width, output.height).context("Unable to save plot")?;
		},

		args::SubCmd::PageTemperature { input_file, output } => {
			// Parse the input file
			let data = self::read_data(&input_file)?;

			let (min_time, max_time) = data
				.hemem
				.page_accesses
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
			let points = data
				.hemem
				.page_accesses
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
					PlotOption::Caption("Page migrations (Avg)"),
					PlotOption::Color("black"),
					PlotOption::LineStyle(DashType::Solid),
					PlotOption::LineWidth(1.0),
				])
				.fill_between(
					points.iter().map(|p| p.x),
					points.iter().map(|p| p.y_avg - p.y_err),
					points.iter().map(|p| p.y_avg + p.y_err),
					&[
						PlotOption::Caption("Page migrations (Error)"),
						PlotOption::Color("green"),
						PlotOption::FillAlpha(0.3),
						PlotOption::FillRegion(FillRegionType::Below),
					],
				)
				.set_x_label("Time (normalized)", &[])
				.set_y_label("Temperature", &[])
				.set_x_range(AutoOption::Fix(0.0), AutoOption::Fix(1.0))
				.set_y_range(AutoOption::Fix(0.0), AutoOption::Fix(max_y));

			self::save_plot(&output.file, &mut fg, output.width, output.height).context("Unable to save plot")?;
		},

		args::SubCmd::PageTemperatureDensity {
			input_file,
			output,
			temp_exponent,
			temp_read_weight,
			temp_write_weight,
		} => {
			// Parse the input file
			let data = self::read_data(&input_file)?;

			let (min_time, max_time) = data
				.hemem
				.page_accesses
				.accesses
				.iter()
				.map(|page_access| page_access.time)
				.minmax()
				.into_option()
				.unwrap_or((0, 1));

			// Then index the page pointers.
			// Note: We do this because the page pointers are very far away, value-wise, which
			//       causes them to display far away in the graph. Since the actual values of the
			//       pages don't matter to us, we just index them by order of appearance.
			let page_ptr_idxs = data
				.hemem
				.page_migrations
				.migrations
				.iter()
				.enumerate()
				.map(|(idx, (page_ptr, _))| (page_ptr, idx))
				.collect::<std::collections::BTreeMap<_, _>>();


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
					let mut temps = page_accesses
						.map(|page_access| {
							let cur_temp = cur_temps.entry(page_ptr).or_insert(0.0);
							match page_access.kind {
								ftmemsim::data::PageAccessKind::Read => *cur_temp += temp_read_weight,
								ftmemsim::data::PageAccessKind::Write => *cur_temp += temp_write_weight,
							};

							(page_access.time, *cur_temp)
						})
						.sorted_by_key(|&(time, _)| time)
						.collect::<VecDeque<_>>();

					let last_temp = temps.back().map_or(0.0, |&(_, temp)| temp);
					temps.push_front((min_time, 0.0));
					temps.push_back((max_time, last_temp));

					(page_ptr, Vec::from(temps))
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

				for &[(prev_time2, prev_temp), (cur_time, cur_temp)] in temps.array_windows::<2>() {
					let prev_time = (prev_time2 - min_time) as f64 / (max_time - min_time) as f64;
					let cur_time = (cur_time - min_time) as f64 / (max_time - min_time) as f64;

					let progress = (prev_temp + cur_temp) / (2.0 * max_temp);
					let progress = progress.powf(temp_exponent);

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

			self::save_plot(&output.file, &mut fg, output.width, output.height).context("Unable to save plot")?;
		},
	}

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
