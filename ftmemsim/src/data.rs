//! Output data

// Imports
use std::collections::BTreeMap;

/// Output data
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Data {
	pub hemem: HeMemData,
}

/// Hemem output data
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct HeMemData {
	pub page_accesses:   PageAccesses,
	pub page_migrations: PageMigrations,
}

/// Page accesses
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PageAccesses {
	pub accesses: Vec<PageAccess>,
}

/// Page access
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
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
pub enum PageAccessKind {
	Read,
	Write,
}

/// Page migrations
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
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
pub struct PageMigration {
	pub prev_mem_idx: Option<usize>,
	pub cur_mem_idx:  usize,
	pub time:         u64,
}
