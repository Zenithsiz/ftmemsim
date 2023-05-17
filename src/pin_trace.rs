//! `pin` traces parsing.

// Imports
use {
	crate::util::ReadByteArray,
	anyhow::Context,
	byteorder::{LittleEndian, ReadBytesExt},
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
	/// Magic
	pub const MAGIC: [u8; 8] = *b"PINT v0\0";

	/// Parses a pin trace from a reader
	pub fn from_reader(mut reader: R) -> Result<Self, anyhow::Error> {
		// Read the magic
		let magic = reader.read_byte_array().context("Unable to read magic")?;
		anyhow::ensure!(
			magic == Self::MAGIC,
			"Found wrong magic {:?}, expected {:?}",
			magic,
			Self::MAGIC
		);

		// Read the header
		let header = Header::from_reader(&mut reader).context("Unable to read header")?;
		tracing::trace!(?header, "Parsed header");

		// Get the total number of records
		// TODO: Not have this hack here?
		let total_records = {
			let magic_size = Self::MAGIC.len() as u64;
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

/// Header
#[derive(Clone, Copy, Debug)]
pub struct Header {
	/// Total records
	records: u64,

	/// Rate
	_rate: u64,

	/// Load misses
	_load_misses: u64,

	/// Load accesses
	_load_accesses: u64,

	/// Store misses
	_store_misses: u64,

	/// Store accesses
	_store_accesses: u64,
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
			_rate: rate,
			_load_misses: load_misses,
			_load_accesses: load_accesses,
			_store_misses: store_misses,
			_store_accesses: store_accesses,
		})
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
	pub const BYTE_SIZE: usize = 0x16;

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
}

/// Record access kind
#[derive(Clone, Copy, Debug)]
pub enum RecordAccessKind {
	/// Read
	Read,

	/// Write
	Write,
}
