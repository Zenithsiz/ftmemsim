//! `pin` traces parsing.

// Imports
use {
	anyhow::Context,
	byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt},
	ftmemsim_util::ReadByteArray,
	std::io,
};

/// Pin trace reader
#[derive(Clone, Debug)]
pub struct PinTraceReader<R> {
	/// Header
	_header: Header,

	/// Records remaining
	records_remaining: u64,

	/// Reader
	reader: R,
}

impl<R: io::Read + io::Seek> PinTraceReader<R> {
	/// Parses a pin trace from a reader
	pub fn from_reader(mut reader: R) -> Result<Self, anyhow::Error> {
		// Read the magic
		let magic = reader.read_byte_array().context("Unable to read magic")?;
		anyhow::ensure!(magic == MAGIC, "Found wrong magic {magic:?}, expected {MAGIC:?}",);

		// Read the header
		let header = Header::from_reader(&mut reader).context("Unable to read header")?;
		tracing::trace!(?header, "Parsed header");

		// Get the total number of records
		// TODO: Not have this hack here?
		let total_records = {
			let magic_size = MAGIC.len() as u64;
			let header_size = Header::BYTE_SIZE as u64;
			let record_size = Record::BYTE_SIZE as u64;


			let total_actual_size = reader.stream_len().context("Unable to get stream length")?;
			let total_expected_size = magic_size + header_size + header.records * record_size;
			if total_actual_size != total_expected_size {
				tracing::warn!(
					"Pin trace size differs from expected. Found {total_actual_size}, expected {total_expected_size}"
				);
			}

			(total_actual_size - magic_size - header_size) / record_size
		};

		Ok(Self {
			_header: header,
			records_remaining: total_records,
			reader,
		})
	}

	/// Reads the next record
	pub fn read_next(&mut self) -> Result<Option<Record>, anyhow::Error> {
		// If we're done, return `None`
		if self.records_remaining == 0 {
			return Ok(None);
		}

		// Else parse the next record and reduce the remaining records
		let record = Record::from_reader(&mut self.reader).context("Unable to read record")?;
		self.records_remaining -= 1;

		Ok(Some(record))
	}

	/// Returns the remaining records
	pub fn records_remaining(&self) -> u64 {
		self.records_remaining
	}
}

/// Pin trace writer
#[derive(Clone, Debug)]
pub struct PinTraceWriter<W> {
	/// Records written
	records_written: u64,

	/// Writer
	writer: W,
}

impl<W: io::Write + io::Seek> PinTraceWriter<W> {
	/// Creates a new writer
	pub fn new(mut writer: W) -> Result<Self, anyhow::Error> {
		// Write the magic
		// Note: We rewind to ensure we write at the start, because we then
		//       later come back to write the header
		writer.rewind().context("Unable to rewind to start")?;
		writer.write(&MAGIC).context("Unable to write header")?;

		// Skip over the header
		writer
			.seek(io::SeekFrom::Current(Header::BYTE_SIZE as i64))
			.context("Unable to seek past header")?;

		Ok(Self {
			writer,
			records_written: 0,
		})
	}

	/// Writes a record
	pub fn write(&mut self, record: &Record) -> Result<(), anyhow::Error> {
		record.to_writer(&mut self.writer).context("Unable to write record")?;

		self.records_written += 1;
		Ok(())
	}

	/// Finishes writing
	// TODO: Accept the `rate` / `{load, store}_{misses, accesses}`?
	pub fn finish(mut self) -> Result<W, anyhow::Error> {
		// Rewind the writer and write the header
		self.writer
			.seek(io::SeekFrom::Start(MAGIC.len() as u64))
			.context("Unable to seek to header")?;

		let header = Header {
			records:        self.records_written,
			rate:           0,
			load_misses:    0,
			load_accesses:  0,
			store_misses:   0,
			store_accesses: 0,
		};
		header.to_writer(&mut self.writer).context("Unable to write header")?;

		Ok(self.writer)
	}
}

/// Magic
pub const MAGIC: [u8; 8] = *b"PINT v0\0";

/// Header
#[derive(Clone, Copy, Debug)]
pub struct Header {
	/// Total records
	records: u64,

	/// Rate
	rate: u64,

	/// Load misses
	load_misses: u64,

	/// Load accesses
	load_accesses: u64,

	/// Store misses
	store_misses: u64,

	/// Store accesses
	store_accesses: u64,
}

impl Header {
	/// Returns the size of this header (including any padding)
	pub const BYTE_SIZE: usize = 0x38;

	/// Parses a header from a reader
	pub fn from_reader<R: io::Read + io::Seek>(reader: &mut R) -> Result<Self, anyhow::Error> {
		// Read the fields
		let records = reader.read_u64::<LittleEndian>().context("Unable to read records")?;
		let rate = reader.read_u64::<LittleEndian>().context("Unable to read rate")?;
		let load_misses = reader
			.read_u64::<LittleEndian>()
			.context("Unable to read load misses")?;
		let load_accesses = reader
			.read_u64::<LittleEndian>()
			.context("Unable to read load accesses")?;
		let store_misses = reader
			.read_u64::<LittleEndian>()
			.context("Unable to read store misses")?;
		let store_accesses = reader
			.read_u64::<LittleEndian>()
			.context("Unable to read store accesses")?;

		// Then seek over the padding
		reader
			.seek(io::SeekFrom::Current(8))
			.context("Unable to seek over padding")?;

		Ok(Self {
			records,
			rate,
			load_misses,
			load_accesses,
			store_misses,
			store_accesses,
		})
	}

	/// Writes a header to a writer
	pub fn to_writer<W: io::Write + io::Seek>(&self, writer: &mut W) -> Result<(), anyhow::Error> {
		writer
			.write_u64::<LittleEndian>(self.records)
			.context("Unable to write reads")?;
		writer
			.write_u64::<LittleEndian>(self.rate)
			.context("Unable to write rate")?;
		writer
			.write_u64::<LittleEndian>(self.load_misses)
			.context("Unable to write load misses")?;
		writer
			.write_u64::<LittleEndian>(self.load_accesses)
			.context("Unable to write load accesses")?;
		writer
			.write_u64::<LittleEndian>(self.store_misses)
			.context("Unable to write store misses")?;
		writer
			.write_u64::<LittleEndian>(self.store_accesses)
			.context("Unable to write store accesses")?;

		writer
			.seek(io::SeekFrom::Current(8))
			.context("Unable to write padding")?;

		Ok(())
	}
}

/// Record
#[derive(Clone, Copy, Debug)]
pub struct Record {
	/// Timestamp (TODO: unix?)
	pub time: u64,

	/// Address, `4KiB`-aligned
	pub addr: u64,

	/// Access kind
	pub kind: RecordAccessKind,
}

impl Record {
	/// Returns the size of this record
	pub const BYTE_SIZE: usize = 0x10;

	/// Parses a record from a reader
	pub fn from_reader<R: io::Read>(reader: &mut R) -> Result<Self, anyhow::Error> {
		let time = reader.read_u64::<LittleEndian>().context("Unable to read time")?;
		let addr_with_kind = reader
			.read_u64::<LittleEndian>()
			.context("Unable to read address + kind")?;

		let addr = addr_with_kind & !0xfff;
		let kind = match addr_with_kind & 0xfff {
			0 => RecordAccessKind::Read,
			1 => RecordAccessKind::Write,
			kind => anyhow::bail!("Unknown access kind: {kind}"),
		};

		Ok(Self { time, addr, kind })
	}

	/// Writes a record to a writer
	pub fn to_writer<W: io::Write + io::Seek>(&self, writer: &mut W) -> Result<(), anyhow::Error> {
		writer
			.write_u64::<LittleEndian>(self.time)
			.context("Unable to write reads")?;

		let kind_encoded = match self.kind {
			RecordAccessKind::Read => 0b0,
			RecordAccessKind::Write => 0b1,
		};
		let addr_with_kind = (self.addr & !0xfff) | (kind_encoded & 0xfff);

		writer
			.write_u64::<LittleEndian>(addr_with_kind)
			.context("Unable to write rate")?;

		Ok(())
	}
}

/// Record access kind
#[derive(Clone, Copy, Debug)]
pub enum RecordAccessKind {
	/// Read
	Read,

	/// Write
	Write,
}
