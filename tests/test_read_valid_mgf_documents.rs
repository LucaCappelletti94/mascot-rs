/// Test to load all of the *.mgf documents available in the data directory.
use mascot_rs::prelude::*;
use std::collections::HashMap;

#[test]
fn test_valid_read_mgf_documents() {
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
        let vec: Result<MGFVec<usize, f32>, String> =
            MGFVec::valid_from_path_with_error_log(mgf_file, Some(&format!("{}.log", mgf_file)));
        failures += vec.is_err() as usize;
        if vec.is_err() {
            let error = vec.err().unwrap();
            let counter = error_hashmap_counter.entry(error).or_insert(0);
            *counter += 1;
        }
    }
    println!("{} failures our of {} tests", failures, mgf_files.len());
}
