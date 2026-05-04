//! Integration tests for structured MGF header validation.

use mascot_rs::prelude::*;

const VALID_SPLASH: &str = "splash10-0udi-0490000000-4425acda10ed7d4709bd";

fn valid_document_with(splash: &str, formula: &str) -> String {
    format!(
        "BEGIN IONS\n\
PEPMASS=250.0\n\
CHARGE=1\n\
MSLEVEL=2\n\
SMILES=CCO\n\
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

    let spectra: MGFVec<usize> = document.parse()?;

    assert_eq!(spectra.len(), 1);
    assert!(spectra[0].metadata().formula().is_some());
    assert_eq!(spectra[0].metadata().splash(), Some(VALID_SPLASH));
    assert_eq!(SpectrumSplash::splash(&spectra[0])?, VALID_SPLASH);

    Ok(())
}

#[test]
fn rejects_formula_that_disagrees_with_smiles() {
    let document = valid_document_with(VALID_SPLASH, "C3H8O");
    let result: Result<MGFVec<usize>> = document.parse();

    assert!(matches!(
        result,
        Err(MascotError::InputLine { source, .. })
            if matches!(source.as_ref(), MascotError::FormulaSmilesMismatch { .. })
    ));
}

#[test]
fn rejects_splash_that_disagrees_with_peaks() {
    let document = valid_document_with("splash10-0000-0000000000-00000000000000000000", "C2H6O");
    let result: Result<MGFVec<usize>> = document.parse();

    assert!(matches!(
        result,
        Err(MascotError::InputLine { source, .. })
            if matches!(source.as_ref(), MascotError::SplashMismatch { .. })
    ));
}

#[test]
fn rejects_peak_changes_that_would_invalidate_splash() -> Result<()> {
    let document = valid_document_with(VALID_SPLASH, "C2H6O");
    let mut record: MascotGenericFormat<usize> = document.parse()?;

    assert!(matches!(
        record.add_peak(300.0, 30.0),
        Err(MascotError::SplashMismatch { .. })
    ));
    assert_eq!(record.len(), 2);
    assert_eq!(record.metadata().splash(), Some(VALID_SPLASH));

    let document = valid_document_with(VALID_SPLASH, "C2H6O");
    let record: MascotGenericFormat<usize> = document.parse()?;
    assert!(matches!(
        record.top_k_peaks(1),
        Err(MascotError::SplashMismatch { .. })
    ));

    Ok(())
}
