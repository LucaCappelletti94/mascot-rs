//! Integration tests for reading fixture MGF documents.

use mascot_rs::prelude::*;

#[test]
fn test_read_mgf_documents() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut mgf_files: Vec<String> = Vec::new();
    for entry in std::fs::read_dir("tests/data")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|extension| extension == "mgf") {
            mgf_files.push(
                path.to_str()
                    .ok_or_else(|| std::io::Error::other("MGF path is not valid UTF-8"))?
                    .to_string(),
            );
        }
    }

    for mgf_file in mgf_files {
        let vec: MGFVec<usize> = MGFVec::from_path(&mgf_file)?;
        assert!(!vec.is_empty());
    }

    Ok(())
}

#[test]
fn test_unsorted_duplicate_peaks_are_normalized() -> Result<()> {
    let lines = [
        "FEATURE_ID=1",
        "PEPMASS=500.0",
        "SCANS=1",
        "CHARGE=1",
        "RTINSECONDS=10.0",
        "BEGIN IONS",
        "MSLEVEL=2",
        "200.0 1.0",
        "100.0 2.0",
        "100.0 3.0",
        "END IONS",
    ];

    let mgf: MGFVec<usize> = MGFVec::try_from_iter(lines)?;
    let spectrum = &mgf[0];

    let mz_bits = spectrum.mz().map(f64::to_bits).collect::<Vec<_>>();
    assert_eq!(mz_bits, vec![100.0_f64.to_bits(), 200.0_f64.to_bits()]);

    let intensity_bits = spectrum.intensities().map(f64::to_bits).collect::<Vec<_>>();
    assert_eq!(intensity_bits, vec![5.0_f64.to_bits(), 1.0_f64.to_bits()]);

    Ok(())
}

#[test]
fn test_ms_level_is_u8_not_two_value_enum() -> Result<()> {
    let lines = [
        "FEATURE_ID=1",
        "PEPMASS=500.0",
        "SCANS=1",
        "CHARGE=1",
        "RTINSECONDS=10.0",
        "BEGIN IONS",
        "MSLEVEL=3",
        "100.0 2.0",
        "END IONS",
    ];

    let mgf: MGFVec<usize> = MGFVec::try_from_iter(lines)?;

    assert_eq!(mgf[0].level(), 3);
    assert_eq!(mgf[0].metadata().level(), 3);

    Ok(())
}

#[test]
fn test_duplicate_feature_ids_are_distinct_ion_blocks() -> Result<()> {
    let lines = [
        "BEGIN IONS",
        "FEATURE_ID=1",
        "PEPMASS=500.0",
        "CHARGE=1",
        "RTINSECONDS=10.0",
        "MSLEVEL=1",
        "500.0 1.0",
        "SCANS=-1",
        "END IONS",
        "BEGIN IONS",
        "FEATURE_ID=1",
        "PEPMASS=500.0",
        "CHARGE=1",
        "RTINSECONDS=10.0",
        "MSLEVEL=2",
        "100.0 2.0",
        "SCANS=1",
        "END IONS",
    ];

    let mgf: MGFVec<usize> = MGFVec::try_from_iter(lines)?;
    let records: &[MascotGenericFormat<usize>] = mgf.as_ref();

    assert_eq!(mgf.len(), 2);
    assert_eq!(records.len(), 2);
    assert_eq!(mgf[0].feature_id(), 1);
    assert_eq!(mgf[0].level(), 1);
    assert_eq!(mgf[1].feature_id(), 1);
    assert_eq!(mgf[1].level(), 2);

    Ok(())
}

#[test]
fn test_spectrum_access_uses_standard_traits() -> Result<()> {
    let lines = [
        "BEGIN IONS",
        "FEATURE_ID=1",
        "PEPMASS=500.0",
        "CHARGE=1",
        "RTINSECONDS=10.0",
        "MSLEVEL=2",
        "100.0 2.0",
        "200.0 3.0",
        "SCANS=1",
        "END IONS",
    ];

    let mgf: MGFVec<usize> = MGFVec::try_from_iter(lines)?;

    let spectrum_ref: &GenericSpectrum = mgf[0].as_ref();
    assert_eq!(spectrum_ref.len(), 2);
    assert_eq!(spectrum_ref.mz_nth(0).to_bits(), 100.0_f64.to_bits());

    let metadata = MascotGenericFormatMetadata::new(1, 2, 500.0, 10.0, 1, None)?;
    let record = MascotGenericFormat::new(metadata, vec![100.0, 200.0], vec![2.0, 3.0])?;
    let spectrum: GenericSpectrum = record.into();
    assert_eq!(spectrum.len(), 2);
    assert_eq!(spectrum.intensity_nth(1).to_bits(), 3.0_f64.to_bits());

    Ok(())
}
