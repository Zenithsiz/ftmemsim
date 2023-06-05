//! Configuration

/// Configuration
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
	/// Trace skip
	pub trace_skip: usize,

	/// Debug output period (in seconds)
	pub debug_output_period_secs: f64,

	/// Hemem configuration
	pub hemem: HeMemConfig,
}

/// HeMem config
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct HeMemConfig {
	pub read_hot_threshold:       usize,
	pub write_hot_threshold:      usize,
	pub global_cooling_threshold: usize,
	pub memories:                 Vec<HeMemMemory>,
}

/// HeMem memory
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct HeMemMemory {
	pub name:             String,
	pub page_capacity:    usize,
	pub read_latency_ns:  f64,
	pub write_latency_ns: f64,
	pub fault_latency_ns: f64,
}
