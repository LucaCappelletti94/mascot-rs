use core::ops::Add;
use std::{fmt::Debug, str::FromStr};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MascotGenericFormatMetadataBuilder<I, F> {
    feature_id: Option<I>,
    parent_ion_mass: Option<F>,
    retention_time: Option<F>,
    charge: Option<Charge>,
    merge_scans_metadata_builder: Option<MergeScansMetadataBuilder<I>>,
    filename: Option<String>,
}

impl<I, F> Default for MascotGenericFormatMetadataBuilder<I, F> {
    fn default() -> Self {
        Self {
            feature_id: None,
            parent_ion_mass: None,
            retention_time: None,
            charge: None,
            merge_scans_metadata_builder: None,
            filename: None,
        }
    }
}

impl<
        I: Copy + PartialEq + Eq + From<usize> + Debug + FromStr + Add<Output = I>,
        F: StrictlyPositive + Copy,
    > MascotGenericFormatMetadataBuilder<I, F>
{
    pub fn build(self) -> Result<MascotGenericFormatMetadata<I, F>, String> {
        MascotGenericFormatMetadata::new(
            self.feature_id.ok_or_else(|| {
                "Could not build MascotGenericFormatMetadata: feature_id is missing".to_string()
            })?,
            self.parent_ion_mass.ok_or_else(|| {
                "Could not build MascotGenericFormatMetadata: parent_ion_mass is missing"
                    .to_string()
            })?,
            self.retention_time.ok_or_else(|| {
                "Could not build MascotGenericFormatMetadata: retention_time is missing".to_string()
            })?,
            self.charge.ok_or_else(|| {
                "Could not build MascotGenericFormatMetadata: charge is missing".to_string()
            })?,
            self.merge_scans_metadata_builder
                .map(|builder| builder.build())
                .transpose()?,
            self.filename,
        )
    }
}

impl<I: FromStr + Eq + Copy, F: FromStr + PartialEq + Copy> LineParser
    for MascotGenericFormatMetadataBuilder<I, F>
{
    /// Returns whether the line can be parsed by this parser.
    ///
    /// # Arguments
    /// * `line` - The line to parse.
    ///
    /// # Examples
    /// The parser should be able to parse any of the following lines:
    ///
    /// ```rust
    /// use mascot_rs::prelude::*;
    ///
    /// for line in [
    ///     "FEATURE_ID=1",
    ///     "PEPMASS=381.0795",
    ///     "SCANS=1",
    ///     "CHARGE=1",
    ///     "CHARGE=1+",
    ///     "CHARGE=2+",
    ///     "CHARGE=3+",
    ///     "CHARGE=4+",
    ///     "RTINSECONDS=37.083",
    ///     "FILENAME=20220513_PMA_DBGI_01_04_003.mzML",
    ///     "SCANS=-1",
    /// ] {
    ///     let parser = MascotGenericFormatMetadataBuilder::<usize, f64>::default();
    ///     assert!(parser.can_parse_line(line));
    /// }
    /// ```
    fn can_parse_line(&self, line: &str) -> bool {
        line.starts_with("FEATURE_ID=")
            || line.starts_with("PEPMASS=")
            || line.starts_with("SCANS=")
            || line.starts_with("RTINSECONDS=")
            || line.starts_with("FILENAME=")
            || line.starts_with("CHARGE=")
    }

    /// Parses a line to a [`MascotGenericFormatMetadataBuilder`].
    ///
    /// # Arguments
    /// * `line` - The line to parse.
    ///
    /// # Error
    /// * If feature ID was already encountered and it is now different.
    /// * If scans is not -1 or equal to the feature ID.
    /// * If pepmass was already encountered and it is now different.
    /// * If rtinseconds was already encountered and it is now different.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mascot_rs::prelude::*;
    /// use std::str::FromStr;
    ///
    /// let mut parser = MascotGenericFormatMetadataBuilder::<usize, f64>::default();
    ///
    /// parser.digest_line("FEATURE_ID=1").unwrap();
    /// parser.digest_line("PEPMASS=381.0795").unwrap();
    /// parser.digest_line("SCANS=1").unwrap();
    /// parser.digest_line("CHARGE=1").unwrap();
    /// parser.digest_line("RTINSECONDS=37.083").unwrap();
    /// parser.digest_line("FILENAME=20220513_PMA_DBGI_01_04_003.mzML").unwrap();
    ///
    /// let mascot_generic_format_metadata = parser.build().unwrap();
    ///
    /// assert_eq!(mascot_generic_format_metadata.feature_id(), 1);
    /// assert_eq!(mascot_generic_format_metadata.parent_ion_mass(), 381.0795);
    /// assert_eq!(mascot_generic_format_metadata.retention_time(), 37.083);
    /// assert_eq!(mascot_generic_format_metadata.charge(), Charge::One);
    /// assert_eq!(mascot_generic_format_metadata.filename(), Some("20220513_PMA_DBGI_01_04_003.mzML"));
    ///
    /// let mut parser = MascotGenericFormatMetadataBuilder::<usize, f64>::default();
    ///
    /// parser.digest_line("FEATURE_ID=1").unwrap();
    /// assert!(parser.digest_line("FEATURE_ID=2").is_err());
    ///
    /// let mut parser = MascotGenericFormatMetadataBuilder::<usize, f64>::default();
    /// parser.digest_line("FEATURE_ID=1").unwrap();
    /// assert!(parser.digest_line("SCANS=2").is_err());
    ///
    /// let mut parser = MascotGenericFormatMetadataBuilder::<usize, f64>::default();
    /// parser.digest_line("PEPMASS=381.0795").unwrap();
    /// assert!(parser.digest_line("PEPMASS=381.0796").is_err(), concat!(
    ///     "Parser {:?} did not raise error."
    /// ), parser);
    ///
    /// let mut parser = MascotGenericFormatMetadataBuilder::<usize, f64>::default();
    /// parser.digest_line("RTINSECONDS=37.083").unwrap();
    /// assert!(parser.digest_line("RTINSECONDS=37.084").is_err());
    ///
    /// let mut parser = MascotGenericFormatMetadataBuilder::<usize, f64>::default();
    /// parser.digest_line("CHARGE=1").unwrap();
    /// assert!(parser.digest_line("CHARGE=2").is_err());
    ///
    /// ```
    ///
    fn digest_line(&mut self, line: &str) -> Result<(), String> {
        if let Some(stripped) = line.strip_prefix("FEATURE_ID=") {
            let feature_id = I::from_str(stripped).map_err(|_| {
                format!(
                    "Could not parse FEATURE_ID line: could not parse feature ID: {}",
                    line
                )
            })?;
            if let Some(observed_feature_id) = self.feature_id {
                if observed_feature_id != feature_id {
                    return Err(format!(
                        "Could not parse FEATURE_ID line: feature_id was already encountered and it is now different: {}",
                        line
                    ));
                }
            } else {
                self.feature_id = Some(feature_id);
            }
            return Ok(());
        }

        if let Some(stripped) = line.strip_prefix("PEPMASS=") {
            let parent_ion_mass = F::from_str(stripped).map_err(|_| {
                format!(
                    "Could not parse PEPMASS line: could not parse parent ion mass: {}",
                    line
                )
            })?;
            if let Some(observerd_parent_ion_mass) = self.parent_ion_mass {
                if parent_ion_mass != observerd_parent_ion_mass {
                    return Err(format!(
                        "Could not parse PEPMASS line: parent_ion_mass was already encountered and it is now different: {}",
                        line
                    ));
                }
            } else {
                self.parent_ion_mass = Some(parent_ion_mass);
            }
            return Ok(());
        }

        if let Some(stripped) = line.strip_prefix("SCANS=") {
            if stripped == "-1" {
                return Ok(());
            }
            let scans = I::from_str(stripped).map_err(|_| {
                format!(
                    "Could not parse SCANS line: could not parse scans: {}",
                    line
                )
            })?;
            if let Some(feature_id) = self.feature_id {
                if scans != feature_id {
                    return Err(format!(
                        "Could not parse SCANS line: scans is not -1 or equal to the feature ID: {}",
                        line
                    ));
                }
            } else {
                self.feature_id = Some(scans);
            }
            return Ok(());
        }

        if line.starts_with("CHARGE=") {
            let charge = Charge::from_str(line).map_err(|_| {
                format!(
                    "Could not parse CHARGE line: could not parse charge: {}",
                    line
                )
            })?;
            if let Some(observed_charge) = self.charge {
                if observed_charge != charge {
                    return Err(format!(
                        "Could not parse CHARGE line: charge was already encountered and it is now different: {}",
                        line
                    ));
                }
            } else {
                self.charge = Some(charge);
            }
            return Ok(());
        }

        if let Some(stripped) = line.strip_prefix("RTINSECONDS=") {
            let retention_time = F::from_str(stripped).map_err(|_| {
                format!(
                    "Could not parse RTINSECONDS line: could not parse retention time: {}",
                    line
                )
            })?;
            if let Some(observed_retention_time) = self.retention_time {
                if observed_retention_time != retention_time {
                    return Err(format!(
                        "Could not parse RTINSECONDS line: retention_time was already encountered and it is now different: {}",
                        line
                    ));
                }
            } else {
                self.retention_time = Some(retention_time);
            }
            return Ok(());
        }

        if let Some(stripped) = line.strip_prefix("FILENAME=") {
            let filename = stripped.to_string();
            if let Some(observed_filename) = &self.filename {
                if observed_filename != &filename {
                    return Err(format!(
                        "Could not parse FILENAME line: filename was already encountered and it is now different: {}",
                        line
                    ));
                }
            } else {
                self.filename = Some(filename);
            }
            return Ok(());
        }

        Err(format!(
            "Encountered unexpected line while parsing MascotGenericFormatMetadata: {}",
            line
        ))
    }
}
