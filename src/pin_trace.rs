//! `pin` traces parsing.

// Imports
use {
	crate::util::ReadByteArray,
	anyhow::Context,
	byteorder::{LittleEndian, ReadBytesExt},
	std::io,
};

/// Pin trace
#[derive(Clone, Debug)]
pub struct PinTrace {
	/// Header
	pub header: Header,

	/// All records
	pub records: Vec<Record>,
}

impl PinTrace {
	/// Magic
	pub const MAGIC: [u8; 8] = *b"PINT v0\0";

	/// Parses a pin trace from a reader
	pub fn from_reader<R: io::Read + io::Seek>(reader: &mut R) -> Result<Self, anyhow::Error> {
		// Read the magic
		let magic = reader.read_byte_array().context("Unable to read magic")?;
		anyhow::ensure!(
			magic == Self::MAGIC,
			"Found wrong magic {:?}, expected {:?}",
			magic,
			Self::MAGIC
		);

		// Read the header
		let header = Header::from_reader(reader).context("Unable to read header")?;
		tracing::trace!(?header, "Parsed header");

		// And align to 16-bytes
		{
			let cur_pos = reader
				.stream_position()
				.context("Unable to get current stream position")?;
			let aligned_pos = 0x10 * ((cur_pos + 0xf) / 0x10);
			reader
				.seek(io::SeekFrom::Start(aligned_pos))
				.context("Unable to align to 16-byte")?;
		}

		let mut records =
			Vec::with_capacity(usize::try_from(header.records).context("Total records didn't fit into a `usize`")?);
		for _ in 0..header.records {
			let record = Record::from_reader(reader).context("Unable to read record")?;
			records.push(record);
		}

		Ok(Self { header, records })
	}
}

/// Header
#[derive(Clone, Copy, Debug)]
#[expect(dead_code)] // We'll use them when calculating statistics later on
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
	/// Parses a header from a reader
	pub fn from_reader<R: io::Read>(reader: &mut R) -> Result<Self, anyhow::Error> {
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

		Ok(Self {
			records,
			rate,
			load_misses,
			load_accesses,
			store_misses,
			store_accesses,
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
	/// Parses a record from a reader
	pub fn from_reader<R: io::Read>(reader: &mut R) -> Result<Self, anyhow::Error> {
		let time = reader.read_u64::<LittleEndian>().context("Unable to read records")?;
		let addr_with_kind = reader.read_u64::<LittleEndian>().context("Unable to read rate")?;

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
