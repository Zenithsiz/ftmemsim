//! Output data

// Imports
use std::{collections::BTreeMap, ops::Range};

/// Output data
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct Data {
	pub time_span: Option<Range<u64>>,
	pub hemem:     HeMemData,
}

/// Hemem output data
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct HeMemData {
	pub page_accesses:   PageAccesses,
	pub page_migrations: PageMigrations,
}

/// Page accesses
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct PageAccesses {
	pub accesses: Vec<PageAccess>,
}

/// Page access
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct PageAccess {
	pub page_ptr:       u64,
	pub time:           u64,
	pub mem_idx:        usize,
	pub faulted:        bool,
	pub kind:           PageAccessKind,
	pub prev_temp:      usize,
	pub cur_temp:       usize,
	pub caused_cooling: bool,
}

/// Page access kind
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub enum PageAccessKind {
	Read,
	Write,
}

/// Page migrations
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct PageMigrations {
	// Note: We use a `BTreeMap` to ensure the order of the migrations
	//       is always the same, as well as to sort it by page.
	// TODO: Just use `HashMap` here and instead just sort the data when
	//       creating the graphs?
	pub migrations: BTreeMap<u64, Vec<PageMigration>>,
}

/// Page migration
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct PageMigration {
	// TODO: Switch these to `u64`s?
	pub prev_mem_idx: Option<usize>,
	pub cur_mem_idx:  usize,
	pub time:         u64,
}
