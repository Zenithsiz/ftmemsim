//! Random read-write test binary

// Imports
use {
	rand::seq::SliceRandom,
	std::{hint, ptr},
};

// TODO: Make these runtime constants?
const PASSES: usize = 2;
const WRITES_PER_PASS: usize = 100;
const READS_PER_PASS: usize = 150;

fn main() {
	let mut v = vec![0u8; 128 * PAGE_SIZE];

	// Note: We `step_by` the page size because we only care about initializing a single page.
	// TODO: Ensure we're not accidentally measuring creation of `v_{writes, reads}`. We unfortunately
	//       cannot (easily) re-use the memory on each pass.
	for _ in 0..PASSES {
		let mut v_writes = v.iter_mut().step_by(PAGE_SIZE).collect::<Vec<_>>();
		v_writes.shuffle(&mut rand::thread_rng());
		for x in v_writes.drain(..) {
			for _ in 0..WRITES_PER_PASS {
				// SAFETY: Target is valid for writes.
				// Note: We simply want to avoid the write being elided
				unsafe {
					ptr::write_volatile(x, hint::black_box(0));
				}
			}
		}

		let mut v_reads = v.iter().step_by(PAGE_SIZE).collect::<Vec<_>>();
		v_reads.shuffle(&mut rand::thread_rng());
		for x in v_reads.drain(..) {
			for _ in 0..READS_PER_PASS {
				// SAFETY: Target is valid for reads.
				// Note: We simply want to avoid the write being elided
				unsafe {
					hint::black_box(ptr::read_volatile(x));
				}
			}
		}
	}
}

const PAGE_SIZE: usize = 4096;
