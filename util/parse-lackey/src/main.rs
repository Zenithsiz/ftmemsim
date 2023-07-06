//! Parses `valgrind`'s `lackey` tool output from stdin,
//! converting it to a pin trace.

// Features
#![feature(if_let_guard)]

// Imports
use {
	anyhow::Context,
	ftmemsim::PinTraceWriter,
	std::{
		fs,
		io::{BufRead, BufWriter},
		time::SystemTime,
	},
};

fn main() -> Result<(), anyhow::Error> {
	// Create the writer
	// TODO: Allow customizing the output trace file.
	let file = fs::File::create("output.trace").context("Unable to create output file")?;
	let file = BufWriter::new(file);
	let mut pin_writer = PinTraceWriter::new(file).context("Unable to create pin trace writer")?;

	let start_time = SystemTime::now();

	// Start reading the output
	let mut stdin = std::io::stdin().lock();
	let mut line = String::new();
	while let Ok(1..) = {
		line.clear();
		stdin.read_line(&mut line)
	} {
		// Pop the newline
		line.pop();
		if line.ends_with('\r') {
			line.pop();
		}

		// Get the kind of record
		let (kind, addr)= match &line {
			line if let Some(rest) = line.strip_prefix("I ") => (Kind::Inst, rest),
			line if let Some(rest) = line.strip_prefix("L ") => (Kind::Read, rest),
			line if let Some(rest) = line.strip_prefix("S ") => (Kind::Write, rest),
			line if let Some(rest) = line.strip_prefix("M ") => (Kind::Modify, rest),

			// Else ignore line
			_ => continue,
		};

		// Parse the address
		let addr = u64::from_str_radix(addr, 16).context("Unable to parse address")?;

		// Then get the time
		// TODO: Improve this, performance-wise?
		// Note: It's fine to truncate nanos to `u64`, since that's
		//       still 584.55453 years before wrapping.
		let time = std::time::SystemTime::now()
			.duration_since(start_time)
			.expect("System time was non-monotonic")
			.as_nanos() as u64;

		// And write the record
		let record = ftmemsim::pin_trace::Record {
			time,
			addr,
			kind: match kind {
				// TODO: Should we ignore *all* instructions? Technically
				//       the user can `mmap` a exec-able region that will be
				//       watched by hemem, so it might be worth it to not ignore some?
				Kind::Inst => continue,
				Kind::Read => ftmemsim::pin_trace::RecordAccessKind::Read,
				// TODO: What to do with `modify`s? Maybe emit both read+write?
				Kind::Write | Kind::Modify => ftmemsim::pin_trace::RecordAccessKind::Write,
			},
		};
		pin_writer.write(&record).context("Unable to write record")?;
	}

	// Finally finish writing the pin trace
	pin_writer.finish().context("Unable to finish writing pin writer")?;

	Ok(())
}

/// Record kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Kind {
	Inst,
	Read,
	Write,
	Modify,
}
