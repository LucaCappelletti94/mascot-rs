//! Integration tests for structured MGF header validation.

use mascot_rs::prelude::*;

const VALID_SPLASH: &str = "splash10-0udi-0490000000-4425acda10ed7d4709bd";

fn valid_document_with(splash: &str, formula: &str) -> String {
    valid_document_with_smiles(splash, formula, "CCO")
}

fn valid_document_with_smiles(splash: &str, formula: &str, smiles: &str) -> String {
    format!(
        "BEGIN IONS\n\
PEPMASS=250.0\n\
CHARGE=1\n\
MSLEVEL=2\n\
SMILES={smiles}\n\
FORMULA={formula}\n\
SPLASH={splash}\n\
100.0 10.0\n\
200.0 20.0\n\
END IONS\n"
    )
}

#[test]
fn parses_structured_formula_and_validates_splash() -> Result<()> {
    let document = valid_document_with(VALID_SPLASH, "C2H6O");

    let spectra: MGFVec = document.parse()?;

    assert_eq!(spectra.len(), 1);
    assert!(spectra[0].metadata().formula().is_some());
    assert_eq!(spectra[0].metadata().splash(), Some(VALID_SPLASH));
    assert_eq!(SpectrumSplash::splash(&spectra[0])?, VALID_SPLASH);

    Ok(())
}

#[test]
fn accepts_formula_that_matches_smiles_after_mixture_merge() -> Result<()> {
    let document = valid_document_with_smiles(VALID_SPLASH, "C8H14BrNO2", "Br.COC(=O)C1=CCCN(C)C1");

    let spectra: MGFVec = document.parse()?;

    assert_eq!(spectra.len(), 1);
    assert!(spectra[0].metadata().formula().is_some());

    Ok(())
}

#[test]
fn accepts_formula_that_matches_smiles_after_isotopic_normalization() -> Result<()> {
    let document = valid_document_with_smiles(VALID_SPLASH, "CH3Br", "C[79Br]");

    let spectra: MGFVec = document.parse()?;

    assert_eq!(spectra.len(), 1);
    assert!(spectra[0].metadata().formula().is_some());

    Ok(())
}

#[test]
fn treats_missing_formula_and_splash_markers_as_absent() -> Result<()> {
    let document = concat!(
        "BEGIN IONS\n",
        "PEPMASS=250.0\n",
        "CHARGE=1\n",
        "MSLEVEL=2\n",
        "SMILES=CCO\n",
        "FORMULA=N/A\n",
        "SPLASH=N/A\n",
        "100.0 10.0\n",
        "200.0 20.0\n",
        "END IONS\n",
    );

    let spectra: MGFVec = document.parse()?;

    assert_eq!(spectra.len(), 1);
    assert!(spectra[0].metadata().formula().is_none());
    assert_eq!(spectra[0].metadata().splash(), None);

    Ok(())
}

#[test]
fn accepts_mass_spec_gym_isotopic_bromine_formula_order() -> Result<()> {
    let document = valid_document_with_smiles(
        VALID_SPLASH,
        "C24H47BrNO8P",
        "C[N+](C)(C)CCOP(=O)([O-])OCC(CO)OC(=O)CCCC(CCCCCC/C=C\\CCC[79Br])O",
    );

    let spectra: MGFVec = document.parse()?;

    assert_eq!(spectra.len(), 1);
    assert!(spectra[0].metadata().formula().is_some());

    Ok(())
}

#[test]
fn rejects_formula_that_disagrees_with_smiles() -> std::result::Result<(), String> {
    let document = valid_document_with(VALID_SPLASH, "C3H8O");
    let result: Result<MGFVec> = document.parse();

    let source = match result {
        Err(MascotError::InputLine { source, .. }) => source,
        result => return Err(format!("expected formula/SMILES mismatch, got {result:?}")),
    };
    assert!(matches!(
        source.as_ref(),
        MascotError::FormulaSmilesMismatch { .. }
    ));
    let message = source.to_string();
    assert!(message.contains("FORMULA/SMILES validation failed"));
    assert!(message.contains("MGF FORMULA header is C3H8O"));
    assert!(message.contains("SMILES-derived formula is C₂H₆O"));
    assert!(message.contains("isotope-insensitive atom-count vectors are different"));

    Ok(())
}

#[test]
fn rejects_splash_that_disagrees_with_peaks() -> std::result::Result<(), String> {
    let document = valid_document_with("splash10-0000-0000000000-00000000000000000000", "C2H6O");
    let result: Result<MGFVec> = document.parse();

    let source = match result {
        Err(MascotError::InputLine { source, .. }) => source,
        result => return Err(format!("expected SPLASH mismatch, got {result:?}")),
    };
    assert!(matches!(
        source.as_ref(),
        MascotError::SplashMismatch { .. }
    ));
    let message = source.to_string();
    assert!(message.contains("SPLASH validation failed"));
    assert!(message.contains("the MGF header reports"));
    assert!(message.contains("calculated from the parsed peaks"));

    Ok(())
}

#[test]
fn rejects_peak_changes_that_would_invalidate_splash() -> Result<()> {
    let document = valid_document_with(VALID_SPLASH, "C2H6O");
    let mut record: MascotGenericFormat = document.parse()?;

    assert!(matches!(
        record.add_peak(300.0, 30.0),
        Err(MascotError::SplashMismatch { .. })
    ));
    assert_eq!(record.len(), 2);
    assert_eq!(record.metadata().splash(), Some(VALID_SPLASH));

    let document = valid_document_with(VALID_SPLASH, "C2H6O");
    let record: MascotGenericFormat = document.parse()?;
    assert!(matches!(
        record.top_k_peaks(1),
        Err(MascotError::SplashMismatch { .. })
    ));

    Ok(())
}
