//! Simple read-write test binary

// Imports
use std::{hint, ptr};

// TODO: Make these runtime constants?
const PASSES: usize = 2;
const WRITES_PER_PASS: usize = 100;
const READS_PER_PASS: usize = 150;

fn main() {
	let mut v = vec![0u8; 128 * PAGE_SIZE];

	// Note: We `step_by` the page size because we only care about initializing a single page.
	for _ in 0..PASSES {
		for x in v.iter_mut().step_by(PAGE_SIZE) {
			for _ in 0..WRITES_PER_PASS {
				// SAFETY: Target is valid for writes.
				// Note: We simply want to avoid the write being elided
				unsafe {
					ptr::write_volatile(x, hint::black_box(0));
				}
			}
		}

		for x in v.iter().step_by(PAGE_SIZE) {
			for _ in 0..READS_PER_PASS {
				// SAFETY: Target is valid for writes.
				// Note: We simply want to avoid the write being elided
				unsafe {
					hint::black_box(ptr::read_volatile(x));
				}
			}
		}
	}
}

const PAGE_SIZE: usize = 4096;
