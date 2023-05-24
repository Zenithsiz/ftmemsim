//! Utilities

// Features
#![feature(decl_macro)]

// Modules
pub mod logger;

// Imports
use std::collections::HashMap;


/// Serialized page location
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
