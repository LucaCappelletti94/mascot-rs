//! Fuzzing harness to test whether the intersection estimation works as expected.
#![no_main]

use arbitrary::Arbitrary;
use mascot_rs::prelude::*;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct FuzzCase {
    document: Vec<String>,
}


fuzz_target!(|data: FuzzCase| {
    let _ = MGFVec::<usize, f32>::try_from_iter(data.document.iter().map(|s| s.as_str()));
});