//! Simple read-write test binary

// Imports
use std::{hint, ptr};

const PAGE_SIZE: usize = 4096;

// TODO: Make these runtime constants?
const TOTAL_BYTES: usize = 16384 * PAGE_SIZE;
const PASSES: usize = 8;
const PASS_STEP: usize = PAGE_SIZE;

fn main() {
	let mut v = vec![0u8; TOTAL_BYTES];

	// Note: We `step_by` the page size because we only care about initializing a single page.
	for _ in 0..PASSES {
		for x in v.iter_mut().step_by(PASS_STEP) {
			// SAFETY: Target is valid for writes.
			// Note: We simply want to avoid the write being elided
			unsafe {
				ptr::write_volatile(x, hint::black_box(0));
			}
		}

		for x in v.iter_mut().step_by(PASS_STEP) {
			// SAFETY: Target is valid for writes.
			// Note: We simply want to avoid the write being elided
			unsafe {
				hint::black_box(ptr::read_volatile(x));
			}
		}
	}
}
