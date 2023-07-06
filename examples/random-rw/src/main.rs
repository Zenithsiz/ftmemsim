//! Random read-write test binary

// Imports
use {
	rand::Rng,
	std::{hint, ptr},
};

const PAGE_SIZE: usize = 4096;

// TODO: Make these runtime constants?
const TOTAL_BYTES: usize = 4096 * PAGE_SIZE;
const PASSES: usize = 2;
const PASS_BYTES: usize = TOTAL_BYTES / PAGE_SIZE;

fn main() {
	let mut v = vec![0u8; TOTAL_BYTES];

	let mut thread_rng = rand::thread_rng();
	for _ in 0..PASSES {
		for idx in std::iter::from_fn(|| Some(thread_rng.gen_range(0..TOTAL_BYTES))).take(PASS_BYTES) {
			let x = &mut v[idx];

			// SAFETY: Target is valid for writes.
			// Note: We simply want to avoid the write being elided
			unsafe {
				ptr::write_volatile(x, hint::black_box(0));
			}
		}

		for idx in std::iter::from_fn(|| Some(thread_rng.gen_range(0..TOTAL_BYTES))).take(PASS_BYTES) {
			let x = &v[idx];

			// SAFETY: Target is valid for reads.
			// Note: We simply want to avoid the write being elided
			unsafe {
				hint::black_box(ptr::read_volatile(x));
			}
		}
	}
}
