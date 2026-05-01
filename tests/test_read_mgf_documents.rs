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
fn test_mgf_vec_from_str_accepts_multiple_ion_blocks() -> Result<()> {
    let document = concat!(
        "BEGIN IONS\n",
        "FEATURE_ID=1\n",
        "PEPMASS=500.0\n",
        "CHARGE=1\n",
        "RTINSECONDS=10.0\n",
        "MSLEVEL=2\n",
        "100.0 2.0\n",
        "SCANS=1\n",
        "END IONS\n",
        "\n",
        "BEGIN IONS\n",
        "FEATURE_ID=2\n",
        "PEPMASS=600.0\n",
        "CHARGE=1\n",
        "RTINSECONDS=12.0\n",
        "MSLEVEL=2\n",
        "200.0 3.0\n",
        "SCANS=2\n",
        "END IONS\n",
    );

    let mgf: MGFVec<usize, f32> = document.parse()?;

    assert_eq!(mgf.len(), 2);
    assert_eq!(mgf[0].feature_id(), 1);
    assert_eq!(mgf[1].feature_id(), 2);
    assert_eq!(mgf[1].precursor_mz().to_bits(), 600.0_f32.to_bits());

    Ok(())
}

#[test]
fn test_mgf_from_str_requires_exactly_one_ion_block() -> Result<()> {
    let document = concat!(
        "BEGIN IONS\n",
        "FEATURE_ID=1\n",
        "PEPMASS=500.0\n",
        "CHARGE=1\n",
        "RTINSECONDS=10.0\n",
        "MSLEVEL=2\n",
        "100.0 2.0\n",
        "SCANS=1\n",
        "END IONS\n",
    );
    let two_record_document = concat!(
        "BEGIN IONS\n",
        "FEATURE_ID=1\n",
        "PEPMASS=500.0\n",
        "CHARGE=1\n",
        "RTINSECONDS=10.0\n",
        "MSLEVEL=2\n",
        "100.0 2.0\n",
        "SCANS=1\n",
        "END IONS\n",
        "BEGIN IONS\n",
        "FEATURE_ID=2\n",
        "PEPMASS=600.0\n",
        "CHARGE=1\n",
        "RTINSECONDS=12.0\n",
        "MSLEVEL=2\n",
        "200.0 3.0\n",
        "SCANS=2\n",
        "END IONS\n",
    );

    let record: MascotGenericFormat<usize> = document.parse()?;

    assert_eq!(record.feature_id(), 1);
    assert!(matches!(
        "".parse::<MascotGenericFormat<usize>>(),
        Err(MascotError::SingleRecordExpected { found: 0 })
    ));
    assert!(matches!(
        two_record_document.parse::<MascotGenericFormat<usize>>(),
        Err(MascotError::SingleRecordExpected { found: 2 })
    ));

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

    let metadata = MascotGenericFormatMetadata::new(1, 2, Some(10.0), 1, None)?;
    let record = MascotGenericFormat::new(metadata, 500.0, vec![100.0, 200.0], vec![2.0, 3.0])?;
    let spectrum: GenericSpectrum = record.into();
    assert_eq!(spectrum.len(), 2);
    assert_eq!(spectrum.intensity_nth(1).to_bits(), 3.0_f64.to_bits());

    Ok(())
}

#[test]
fn test_precision_generic_can_store_f32_spectra() -> Result<()> {
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

    let mgf: MGFVec<usize, f32> = MGFVec::try_from_iter(lines)?;
    let records: &[MascotGenericFormat<usize, f32>] = mgf.as_ref();
    let spectrum_ref: &GenericSpectrum<f32> = records[0].as_ref();

    assert_eq!(mgf[0].precursor_mz().to_bits(), 500.0_f32.to_bits());
    assert_eq!(spectrum_ref.mz_nth(0).to_bits(), 100.0_f32.to_bits());
    assert_eq!(spectrum_ref.intensity_nth(1).to_bits(), 3.0_f32.to_bits());

    let metadata = MascotGenericFormatMetadata::new(1, 2, Some(10.0), 1, None)?;
    let record: MascotGenericFormat<usize, f32> =
        MascotGenericFormat::new(metadata, 500.0, vec![100.0, 200.0], vec![2.0, 3.0])?;
    let spectrum: GenericSpectrum<f32> = record.into();
    assert_eq!(spectrum.precursor_mz().to_bits(), 500.0_f32.to_bits());

    Ok(())
}

#[test]
fn test_memory_footprint_is_available_from_prelude() -> Result<()> {
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
    let size = mgf.mem_size(SizeFlags::default());
    let capacity_size = mgf.mem_size(SizeFlags::CAPACITY);
    let metadata_size = mgf[0].metadata().mem_size(SizeFlags::default());
    let builder = MGFVec::<usize>::gnps();
    let builder_size = builder.mem_size(SizeFlags::default());
    let mut report = String::new();

    assert!(size >= std::mem::size_of_val(&mgf));
    assert!(capacity_size >= size);
    assert!(metadata_size >= std::mem::size_of_val(mgf[0].metadata()));
    assert!(builder_size >= std::mem::size_of_val(&builder));
    assert!(mgf
        .mem_dbg_depth_on(&mut report, 2, DbgFlags::default())
        .is_ok());
    assert!(report.contains("MGFVec"));

    Ok(())
}

#[test]
fn test_gnps_library_records_parse_annotation_metadata() -> Result<()> {
    let lines = [
        "BEGIN IONS",
        "PEPMASS=561.365",
        "CHARGE=1",
        "MSLEVEL=2",
        "SOURCE_INSTRUMENT=DI-ESI-LTQ-FT-ICR",
        "FILENAME=Desferrioxamine_B_1H_561_3647.mzXML",
        "SEQ=..",
        "IONMODE=Positive",
        "ORGANISM=GNPS-LIBRARY",
        "NAME=Desferrioxamine B M+H",
        "PI=Dorrestein",
        "DATACOLLECTOR=J Watrous",
        "SMILES=N/A",
        "INCHI=N/A",
        "INCHIAUX=N/A",
        "PUBMED=N/A",
        "SUBMITUSER=jdwatrou",
        "TAGS=",
        "LIBRARYQUALITY=3",
        "SPECTRUMID=CCMSLIB00000072100",
        "SCANS=1",
        "161.0 2.216415",
        "161.27272 3.386504",
        "165.181824 94.756683",
        "END IONS",
    ];

    let mgf: MGFVec<usize> = MGFVec::try_from_iter(lines)?;

    assert_eq!(mgf.len(), 1);
    assert_eq!(mgf[0].feature_id(), 1);
    assert_eq!(mgf[0].metadata().retention_time(), None);
    assert_eq!(mgf[0].level(), 2);
    assert_eq!(mgf[0].len(), 3);

    Ok(())
}

#[test]
fn test_empty_records_are_rejected_by_strict_parser() {
    let lines = [
        "BEGIN IONS",
        "PEPMASS=763.0",
        "CHARGE=1",
        "MSLEVEL=2",
        "FILENAME=f.smascuch/Standards/STANDARD_Ferrichrome_FT01_50K_MS2_mz763.mzXML;",
        "NAME=Ferrichrome M+Na",
        "SPECTRUMID=CCMSLIB00000078897",
        "SCANS=1",
        "END IONS",
    ];

    assert!(matches!(
        MGFVec::<usize>::try_from_iter(lines),
        Err(MascotError::EmptyPeakVectors)
    ));
}

#[test]
fn test_gnps_builder_loads_existing_downloaded_file() -> Result<()> {
    let target_directory =
        std::env::temp_dir().join(format!("mascot-rs-gnps-test-{}", std::process::id()));
    std::fs::create_dir_all(&target_directory).map_err(|source| MascotError::Io {
        path: target_directory.display().to_string(),
        source,
    })?;
    let path = target_directory.join("ALL_GNPS.mgf");
    let document = concat!(
        "BEGIN IONS\n",
        "PEPMASS=0.0\n",
        "CHARGE=1\n",
        "MSLEVEL=2\n",
        "NAME=Invalid GNPS record\n",
        "SPECTRUMID=CCMSLIB00000000000\n",
        "SCANS=1\n",
        "161.0 2.216415\n",
        "END IONS\n",
        "BEGIN IONS\n",
        "PEPMASS=763.0\n",
        "CHARGE=1\n",
        "MSLEVEL=2\n",
        "NAME=Empty GNPS record\n",
        "SPECTRUMID=CCMSLIB00000078897\n",
        "SCANS=2\n",
        "END IONS\n",
        "BEGIN IONS\n",
        "PEPMASS=561.365\n",
        "CHARGE=1\n",
        "MSLEVEL=2\n",
        "NAME=Desferrioxamine B M+H\n",
        "SPECTRUMID=CCMSLIB00000072100\n",
        "SCANS=3\n",
        "161.0 2.216415\n",
        "END IONS\n",
    );
    std::fs::write(&path, document).map_err(|source| MascotError::Io {
        path: path.display().to_string(),
        source,
    })?;

    let gnps_load = pollster::block_on(
        MGFVec::<usize, f32>::gnps()
            .target_directory(&target_directory)
            .load(),
    )?;
    let _ = std::fs::remove_dir_all(&target_directory);

    assert!(gnps_load.mem_size(SizeFlags::default()) >= std::mem::size_of_val(&gnps_load));
    assert_eq!(gnps_load.spectra().len(), 1);
    assert_eq!(gnps_load.skipped_records(), 2);
    assert_eq!(gnps_load.spectra()[0].metadata().retention_time(), None);
    assert_eq!(
        gnps_load.spectra()[0].precursor_mz().to_bits(),
        561.365_f32.to_bits()
    );

    Ok(())
}
