use core::ops::Add;
use std::{fmt::Debug, str::FromStr};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MascotGenericFormatMetadataBuilder<I, F> {
    feature_id: Option<I>,
    parent_ion_mass: Option<F>,
    retention_time: Option<F>,
    charge: Option<Charge>,
    organism: Option<String>,
    sequence: Option<String>,
    source_instrument: Option<String>,
    ion_mode: Option<IonMode>,
    minus_one_scans: bool,
    merge_scans_metadata_builder: Option<MergeScansMetadataBuilder<I>>,
}

impl<I, F> Default for MascotGenericFormatMetadataBuilder<I, F> {
    fn default() -> Self {
        Self {
            feature_id: None,
            parent_ion_mass: None,
            retention_time: None,
            charge: None,
            organism: None,
            sequence: None,
            source_instrument: None,
            ion_mode: None,
            minus_one_scans: false,
            merge_scans_metadata_builder: None,
        }
    }
}

impl<
        I: Copy + PartialEq + Eq + From<usize> + Debug + FromStr + Add<Output = I> + Zero,
        F: StrictlyPositive + Copy,
    > MascotGenericFormatMetadataBuilder<I, F>
{
    pub fn build(self) -> Result<MascotGenericFormatMetadata<I, F>, String> {
        if self.minus_one_scans {
            return Err(concat!(
                "Could not build MascotGenericFormatMetadata as the scan status is ",
                "currently set to -1, which indicates a partial read fragment ion spectrum."
            )
            .to_string());
        }

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
            self.source_instrument,
            self.sequence,
            self.organism,
            self.charge.ok_or_else(|| {
                "Could not build MascotGenericFormatMetadata: charge is missing".to_string()
            })?,
            self.ion_mode,
            self.merge_scans_metadata_builder
                .map(|builder| builder.build())
                .transpose()?,
        )
    }
}

impl<
        I: FromStr + Eq + Copy + Add<Output = I> + Debug,
        F: FromStr + PartialEq + Copy + NaN + StrictlyPositive,
    > LineParser for MascotGenericFormatMetadataBuilder<I, F>
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
    ///     "CHARGE=5+",
    ///     "IONMODE=positive",
    ///     "IONMODE=negative",
    ///     "IONMODE=N/A",
    ///     "ORGANISM=GNPS-COLLECTIONS-PESTICIDES-POSITIVE",
    ///     "RTINSECONDS=37.083",
    ///     "SEQ=*..*",
    ///     "FILENAME=20220513_PMA_DBGI_01_04_003.mzML",
    ///     "SCANS=-1",
    /// ] {
    ///     assert!(MascotGenericFormatMetadataBuilder::<usize, f64>::can_parse_line(line));
    /// }
    /// ```
    fn can_parse_line(line: &str) -> bool {
        line.starts_with("FEATURE_ID=")
            || line.starts_with("PEPMASS=")
            || line.starts_with("SCANS=")
            || line.starts_with("RTINSECONDS=")
            || line.starts_with("FILENAME=")
            || line.starts_with("SOURCE_INSTRUMENT=")
            || line.starts_with("IONMODE=")
            || line.starts_with("ORGANISM=")
            || line.starts_with("SEQ=")
            || line.starts_with("CHARGE=")
            || MergeScansMetadataBuilder::<I>::can_parse_line(line)
    }

    /// Returns whether the parser can build a [`MascotGenericFormatMetadata`] from the lines
    fn can_build(&self) -> bool {
        self.feature_id.is_some()
            && self.parent_ion_mass.is_some()
            && self.retention_time.is_some()
            && self.charge.is_some()
            && !self.minus_one_scans
            && self
                .merge_scans_metadata_builder
                .as_ref()
                .map_or(true, |builder| builder.can_build())
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
    /// parser.digest_line("MERGED_SCANS=1567,1540");
    /// parser.digest_line("SOURCE_INSTRUMENT=ESI-qTof");
    /// parser.digest_line("IONMODE=positive");
    /// parser.digest_line("ORGANISM=GNPS-COLLECTIONS-PESTICIDES-POSITIVE");
    /// parser.digest_line("SOURCE_INSTRUMENT=LC-ESI-Q-Exactive Plus Orbitrap Res 14k");
    /// parser.digest_line("MERGED_STATS=2 / 2 (0 removed due to low quality, 0 removed due to low cosine).");
    /// parser.digest_line("RTINSECONDS=37.083").unwrap();
    /// parser.digest_line("FILENAME=20220513_PMA_DBGI_01_04_003.mzML").unwrap();
    ///
    /// let mascot_generic_format_metadata = parser.build().unwrap();
    ///
    /// assert_eq!(mascot_generic_format_metadata.feature_id(), 1);
    /// assert_eq!(mascot_generic_format_metadata.parent_ion_mass(), 381.0795);
    /// assert_eq!(mascot_generic_format_metadata.retention_time(), 37.083);
    /// assert_eq!(mascot_generic_format_metadata.charge(), Charge::One);
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
                        "Could not parse FEATURE_ID line: feature_id was already encountered ({:?}) and it is now different: {}",
                        observed_feature_id,
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
            if parent_ion_mass.is_nan() {
                return Err(format!(
                    concat!(
                        "The provided line \"{}\" contains a parent ion mass ",
                        "that has been interpreted as a NaN."
                    ),
                    line
                ));
            }
            if !parent_ion_mass.is_strictly_positive() {
                return Err(format!(
                    concat!(
                        "The provided line \"{}\" contains a parent ion mass ",
                        "that has been interpreted as a zero or negative value. ",
                        "The parent ion mass must be a strictly positive value."
                    ),
                    line
                ));
            }
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
                self.minus_one_scans = true;
                return Ok(());
            }
            self.minus_one_scans = false;
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

        // If the line starts with IONMODE, we update the value of the ion mode.
        // If the value of the ion mode is already set, we check that the
        // new value is the same, and if the value we encounter is equal to "N/A"
        // we leave the value of the ion mode unchanged.
        if let Some(stripped) = line.strip_prefix("IONMODE=") {
            if IonMode::is_nan_ion_mode_from_str(stripped) {
                return Ok(());
            }
            let this_ion_mode = IonMode::from_str(stripped)?;
            if let Some(ion_mode) = self.ion_mode {
                if ion_mode != this_ion_mode {
                    return Err(format!(
                        "Could not parse IONMODE line: ion_mode was already encountered and it is now different: {}",
                        line
                    ));
                }
            }
            self.ion_mode = Some(this_ion_mode);
            return Ok(());
        }

        // If the line starts with SEQ, we update the value of the sequence.
        // If the value of the sequence is already set, we check that the
        // new value is the same, and if the value we encounter is equal to "*..*"
        // we leave the value of the sequence unchanged.
        if let Some(stripped) = line.strip_prefix("SEQ=") {
            if let Some(sequence) = &self.sequence {
                if sequence != stripped {
                    return Err(format!(
                        "Could not parse SEQ line: sequence was already encountered and it is now different: {}",
                        line
                    ));
                }
            } else if stripped != "*..*" {
                self.sequence = Some(stripped.to_string());
            }
            return Ok(());
        }

        if let Some(stripped) = line.strip_prefix("SOURCE_INSTRUMENT=") {
            if let Some(observed_source_instrument) = &self.source_instrument {
                if observed_source_instrument != stripped {
                    return Err(format!(
                        "Could not parse SOURCE_INSTRUMENT line: source_instrument was already encountered and it is now different: {}",
                        line
                    ));
                }
            } else {
                self.source_instrument = Some(stripped.to_string());
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
            if retention_time.is_nan() {
                return Err(format!(
                    concat!(
                        "The provided line \"{}\" contains a retention time ",
                        "that has been interpreted as a NaN."
                    ),
                    line
                ));
            }
            if !retention_time.is_strictly_positive() {
                return Err(format!(
                    concat!(
                        "The provided line \"{}\" contains a retention time ",
                        "that has been interpreted as a zero or negative value. ",
                        "The retention time must be a strictly positive value."
                    ),
                    line
                ));
            }
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

        if let Some(stripped) = line.strip_prefix("ORGANISM=") {
            if let Some(observed_organism) = &self.organism {
                if observed_organism != stripped {
                    return Err(format!(
                        "Could not parse ORGANISM line: organism was already encountered and it is now different: {}",
                        line
                    ));
                }
            } else {
                self.organism = Some(stripped.to_string());
            }
            return Ok(());
        }

        // if the line starts with FILENAME we skip it.
        if line.starts_with("FILENAME=") {
            return Ok(());
        }

        if MergeScansMetadataBuilder::<I>::can_parse_line(line) {
            if self.merge_scans_metadata_builder.is_none() {
                self.merge_scans_metadata_builder = Some(MergeScansMetadataBuilder::default());
            }
            self.merge_scans_metadata_builder
                .as_mut()
                .unwrap()
                .digest_line(line)?;
            return Ok(());
        }

        Err(format!(
            "Encountered unexpected line while parsing MascotGenericFormatMetadata: {}",
            line
        ))
    }
}
