/// Test to load all of the *.mgf documents available in the data directory.
use mascot_rs::prelude::*;
use std::collections::HashMap;
use serde_json;

#[test]
fn test_read_mgf_documents() {
    let mut mgf_files: Vec<String> = Vec::new();
    for entry in std::fs::read_dir("tests/data").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().unwrap() == "mgf" {
            mgf_files.push(path.to_str().unwrap().to_string());
        }
    }

    let mut failures = 0;

    let mut error_hashmap_counter: HashMap<String, usize> = HashMap::new();

    for mgf_file in mgf_files.iter() {
        let vec: Result<MGFVec<usize, f32>, String> = MGFVec::from_path(mgf_file);
        failures += vec.is_err() as usize;
        if vec.is_err() {
            let error = vec.err().unwrap();
            let counter = error_hashmap_counter.entry(error).or_insert(0);
            *counter += 1;
        }
        //assert!(!vec.is_empty());
    }
    println!("{} failures our of {} tests", failures, mgf_files.len());

    // Save hashmap to JSON
    let json = serde_json::to_string_pretty(&error_hashmap_counter).unwrap();

    // Write to file
    std::fs::write("tests/error_hashmap_counter.json", json).unwrap();
}
