use std::{fmt::{Debug, Display}, ops::Add, ops::Sub, str::FromStr};

use crate::prelude::*;

#[derive(Debug, Clone)]
/// A builder for [`MascotGenericFormat`].
pub struct MascotGenericFormatBuilder<I, F> {
    metadata_builder: MascotGenericFormatMetadataBuilder<I, F>,
    data_builders: Vec<MascotGenericFormatDataBuilder<F>>,
    section_open: bool,
    corruption: bool,
}

impl<I, F> Default for MascotGenericFormatBuilder<I, F>
where
    I: Copy + Eq + Debug + Add<Output = I> + FromStr + From<usize> + Zero,
    F: Copy + StrictlyPositive + FromStr + PartialEq + Debug,
{
    fn default() -> Self {
        Self {
            metadata_builder: MascotGenericFormatMetadataBuilder::default(),
            data_builders: Vec::new(),
            section_open: false,
            corruption: false,
        }
    }
}

impl<I, F> MascotGenericFormatBuilder<I, F>
where
    I: Copy + Eq + Debug + Add<Output = I> + FromStr + From<usize> + Zero,
    F: Copy
        + StrictlyPositive
        + PartialEq
        + PartialOrd
        + Debug
        + Display
        + Sub<F, Output = F>
        + Add<F, Output = F>,
{
    /// Builds a [`MascotGenericFormat`] from the given data.
    pub fn build(self) -> Result<MascotGenericFormat<I, F>, String> {
        if self.is_corrupted() {
            return Err(format!(
                concat!(
                    "While attempting to build a MascotGenericFormat from the current builder: ",
                    "the builder is corrupted. ",
                    "The current object looks like this: {self:?}"
                ),
                self = self
            ));
        }

        MascotGenericFormat::new(
            self.metadata_builder.build()?,
            self.data_builders
                .into_iter()
                .map(|builder| builder.build())
                .collect::<Result<Vec<_>, String>>()?,
        )
    }

    /// Returns whether the builder is now half-ready to build a [`MascotGenericFormat`].
    ///
    /// # Implementation details
    /// A builder is half-ready to build a [`MascotGenericFormat`] if it has reached the end of the first-level fragmentation data,
    /// and has encountered a SCAN=-1 entry in the metadata. This means that there should be a second-level fragmentation data
    /// to be added, but this is not necessarily always the case: sometimes, in fact, the data is corrupted and the second-level
    /// fragmentation data is missing. In this case, the builder is still half-ready to build a [`MascotGenericFormat`], and this
    /// information is useful to detect these corner cases and avoid corrupted entries to corrupt the following entries.
    pub fn is_partial(&self) -> bool {
        self.metadata_builder.is_partial() && !self.is_level_two() && !self.section_open
    }

    /// Returns whether the current MGF builder has become corrupted.
    pub fn is_corrupted(&self) -> bool {
        self.corruption
    }

    /// Returns whether the level of any of the data builders is equal to two.
    pub fn is_level_two(&self) -> bool {
        self.data_builders
            .iter()
            .any(|builder| builder.is_level_two().unwrap_or(false))
    }

    /// Return whether any of the data builder is empty.
    pub fn has_empty_data_builders(&self) -> bool {
        self.data_builders.iter().any(|builder| builder.is_empty())
    }

    /// Returns the feature ID of the current MGF builder.
    pub fn feature_id(&self) -> Option<I> {
        self.metadata_builder.feature_id()
    }

    pub fn is_start_of_new_entry(line: &str) -> bool {
        line == "BEGIN IONS"
    }

    pub fn is_end_of_entry(line: &str) -> bool {
        line == "END IONS"
    }
}

impl<I, F> LineParser for MascotGenericFormatBuilder<I, F>
where
    I: Copy + Eq + Debug + Add<Output = I> + FromStr + From<usize> + Zero,
    F: Copy
        + StrictlyPositive
        + FromStr
        + PartialEq
        + Debug
        + NaN
        + PartialOrd
        + Zero
        + Display
        + Add<F, Output = F>
        + Sub<F, Output = F>,
{
    fn can_parse_line(line: &str) -> bool {
        Self::is_start_of_new_entry(line)
            || Self::is_end_of_entry(line)
            || MascotGenericFormatMetadataBuilder::<I, F>::can_parse_line(line)
            || MascotGenericFormatDataBuilder::<F>::can_parse_line(line)
    }

    fn can_build(&self) -> bool {
        !self.section_open
            && !self.is_corrupted()
            && self.metadata_builder.can_build()
            && !self.data_builders.is_empty()
            && self.data_builders.iter().all(|builder| builder.can_build())
    }

    /// Digests the given line.
    ///
    /// # Arguments
    /// * `line` - The line to digest.
    ///
    /// # Returns
    /// Whether the line was successfully digested.
    ///
    /// # Errors
    /// * If the line cannot be digested.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mascot_rs::prelude::*;
    ///
    /// let mut parser = MascotGenericFormatBuilder::<usize, f64>::default();
    ///
    /// assert!(parser.digest_line("BEGIN IONS").is_ok());
    /// assert!(parser.digest_line("FEATURE_ID=1").is_ok());
    /// assert!(parser.digest_line("PEPMASS=381.0795").is_ok());
    /// assert!(parser.digest_line("SCANS=1").is_ok());
    /// assert!(parser.digest_line("CHARGE=1").is_ok());
    /// assert!(parser.digest_line("MERGED_SCANS=1567,1540").is_ok());
    /// assert!(parser.digest_line("SOURCE_INSTRUMENT=ESI-qTof").is_ok());
    /// assert!(parser.digest_line("IONMODE=positive").is_ok());
    /// assert!(parser.digest_line("PUBMED=15386517").is_ok());
    /// assert!(parser.digest_line("NAME=Phenazine-1-carboxylic acid CollisionEnergy:102040 M-H").is_ok());
    /// assert!(parser.digest_line("ORGANISM=GNPS-COLLECTIONS-PESTICIDES-POSITIVE").is_ok());
    /// assert!(parser.digest_line("SMILES=FC(F)(F)C1=C(C(N2CCCCC2)=O)N(C=CC=C3OC)C3=N1").is_ok());
    /// assert!(parser.digest_line("MERGED_STATS=2 / 2 (0 removed due to low quality, 0 removed due to low cosine).").is_ok());
    /// assert!(parser.digest_line("RTINSECONDS=37.083").is_ok());
    /// assert!(parser.digest_line("SEQ=*..*").is_ok());
    /// assert!(parser.digest_line("SPECTRUMID=CCMSLIB00000078679").is_ok());
    /// assert!(parser.digest_line("FILENAME=20220513_PMA_DBGI_01_04_003.mzML").is_ok());
    /// assert!(parser.digest_line("MSLEVEL=1").is_ok());
    /// assert!(parser.digest_line("60.5425 2.4E5").is_ok());
    /// assert!(parser.digest_line("119.0857 3.3E5").is_ok());
    /// assert!(parser.digest_line("72.6217 2.1E4").is_ok());
    /// assert!(parser.digest_line("79.0547 1.6E5").is_ok());
    /// assert!(parser.digest_line("81.0606\t1.1E4").is_ok());
    /// assert!(parser.digest_line("81.0704\t2.4E6").is_ok());
    /// assert!(parser.digest_line("END IONS").is_ok());
    /// assert!(parser.digest_line("TITLE=File:").is_err());
    /// ```
    fn digest_line(&mut self, line: &str) -> Result<(), String> {
        if Self::is_start_of_new_entry(line) {
            self.section_open = true;
            self.data_builders
                .push(MascotGenericFormatDataBuilder::default());
        } else if Self::is_end_of_entry(line) {
            // IF we have reached a "END IONS" line, then we must have previously reached a "BEGIN IONS" line.
            if !self.section_open {
                self.corruption = true;
                return Err(format!(
                    concat!(
                        "While attempting to digest line \"END IONS\": no \"BEGIN IONS\" line was found. ",
                        "The current object looks like this: {self:?}"
                    ),
                    self = self
                ));
            }
            // If we have reached a "END IONS" line, then the data builder stack must be buildable.
            if !self.data_builders.last().unwrap().can_build() {
                self.corruption = true;
                if self.has_empty_data_builders() {
                    return Err(format!(
                        concat!(
                            "While attempting to digest line \"END IONS\": the object is not buildable. ",
                            "Specifically, the metadata builder status is {} and the data builder stack is empty. ",
                        ),
                        self.metadata_builder.can_build(),
                    ));
                }
                return Err(format!(
                    concat!(
                        "While attempting to digest line \"END IONS\": the object is not buildable. ",
                        "Specifically, the metadata builder status is {} and the data builder stack status is {}. ",
                    ),
                    self.metadata_builder.can_build(),
                    self.data_builders.iter().all(|builder| builder.can_build()),
                ));
            }

            // If we are currently building a second-level fragmentation data, them when
            // we reach the end of the first-level fragmentation data, it must be buildable.
            self.section_open = false;

            if self.is_level_two() && !self.can_build() {
                self.corruption = true;

                // If the data builder is empty, we provide a more specific error message.
                if self.data_builders.is_empty() {
                    return Err(format!(
                        concat!(
                            "While attempting to digest line \"END IONS\": the object is not buildable. ",
                            "Specifically, the metadata builder status is {} and the data builder stack is empty. ",
                        ),
                        self.metadata_builder.can_build(),
                    ));
                }

                return Err(format!(
                    concat!(
                        "While attempting to digest line \"END IONS\": the object is not buildable. ",
                        "Specifically, the metadata builder status is {} and the data builder stack status is {}. ",
                    ),
                    self.metadata_builder.can_build(),
                    self.data_builders.iter().all(|builder| builder.can_build()),
                ));
            }
        } else if MascotGenericFormatMetadataBuilder::<I, F>::can_parse_line(line) {
            self.metadata_builder.digest_line(line).map_err(|e| {
                self.corruption = true;
                e
            })?;
        } else if let Some(data_builder) = self.data_builders.last_mut() {
            data_builder.digest_line(line).map_err(|e| {
                self.corruption = true;
                e
            })?;
        } else {
            self.corruption = true;
            return Err(format!(
                concat!(
                    "While attempting to digest line \"{line}\": no data builder was found, ",
                    "meaning that the line \"{line}\" was not preceded by \"BEGIN IONS\". ",
                ),
                line = line,
            ));
        }

        Ok(())
    }
}
