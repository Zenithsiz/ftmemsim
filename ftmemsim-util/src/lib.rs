//! Utilities

// Features
#![feature(decl_macro)]

// Modules
pub mod logger;

// Imports
use std::collections::HashMap;

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
	pub page_ptr:  u64,
	pub time:      u64,
	pub mem_idx:   usize,
	pub faulted:   bool,
	pub kind:      PageAccessKind,
	pub prev_temp: usize,
	pub cur_temp:  usize,
}

/// Page access kind
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum PageAccessKind {
	Read,
	Write,
}

/// Page locations
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PageLocations {
	pub locations: HashMap<u64, Vec<PageLocation>>,
}

/// Page location over time
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PageLocation {
	pub mem_idx: usize,
	pub time:    u64,
}
