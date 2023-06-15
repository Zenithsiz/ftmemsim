//! Single page read-write test binary

// Imports
use std::{hint, ptr};

// TODO: Make these runtime constants?
const PASSES: usize = 10;
const WRITES_PER_PASS: usize = 10000;
const READS_PER_PASS: usize = 15000;

fn main() {
	// Note: We over-allocate to ensure the allocation goes through `mmap`.
	let mut v = vec![0u8; 128 * PAGE_SIZE];

	// Note: We `step_by` the page size because we only care about initializing a single page.
	for _ in 0..PASSES {
		for _ in 0..WRITES_PER_PASS {
			// SAFETY: Target is valid for writes.
			// Note: We simply want to avoid the write being elided
			unsafe {
				ptr::write_volatile(v.as_mut_ptr(), hint::black_box(0));
			}
		}

		for _ in 0..READS_PER_PASS {
			// SAFETY: Target is valid for writes.
			// Note: We simply want to avoid the write being elided
			unsafe {
				hint::black_box(ptr::read_volatile(v.as_ptr()));
			}
		}
	}
}

const PAGE_SIZE: usize = 4096;
