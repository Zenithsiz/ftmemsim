//! Utilities

// Features
#![feature(decl_macro)]

// Modules
pub mod logger;

/// Serialized page location
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PageLocations {
	pub locations: Vec<PageLocation>,
}

/// Page location over time
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PageLocation {
	pub page_ptr: u64,
	pub mem_idx:  usize,
	pub time:     u64,
}
