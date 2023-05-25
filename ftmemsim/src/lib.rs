//! Filipe's Tiered Memory Simulator (`ftmemsim`)

// Features
#![feature(decl_macro, lint_reasons, get_many_mut, seek_stream_len)]

// Modules
pub mod classifiers;
pub mod pin_trace;
pub mod sim;

// Exports
pub use self::{
	pin_trace::PinTraceReader,
	sim::{Classifier, Simulator},
};
