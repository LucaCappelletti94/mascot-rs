/// Test to load all of the *.mgf documents available in the data directory.
use mascot_rs::prelude::*;

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

    for mgf_file in mgf_files {
        let vec: MGFVec<usize, f32> = MGFVec::from_path(&mgf_file).unwrap();
        assert!(!vec.is_empty());
    }
}
