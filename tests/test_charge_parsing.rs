//! Integration tests for numeric charge parsing.

use mascot_rs::prelude::*;

#[test]
fn test_parse_i8_charge_values() -> Result<()> {
    for (line, expected_charge) in [
        ("CHARGE=2", 2),
        ("CHARGE=5+", 5),
        ("CHARGE=0", 0),
        ("CHARGE=0+", 0),
        ("CHARGE=0-", 0),
        ("CHARGE=-1", -1),
        ("CHARGE=2-", -2),
    ] {
        let lines = [
            "FEATURE_ID=1",
            "PEPMASS=381.0795",
            "SCANS=1",
            line,
            "RTINSECONDS=37.083",
            "BEGIN IONS",
            "MSLEVEL=2",
            "60.5425 2.4E5",
            "END IONS",
        ];

        let mgf: MGFVec<usize> = MGFVec::try_from_iter(lines)?;
        assert_eq!(mgf[0].charge(), expected_charge);
    }

    Ok(())
}
