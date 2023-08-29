use std::{fmt::Debug, ops::Add, ops::Sub, str::FromStr};

use crate::prelude::*;

#[derive(Debug, Clone)]
/// A builder for [`MascotGenericFormat`].
pub struct MascotGenericFormatBuilder<I, F> {
    metadata_builder: MascotGenericFormatMetadataBuilder<I, F>,
    data_builders: Vec<MascotGenericFormatDataBuilder<F>>,
    section_open: bool,
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
        + Sub<F, Output = F>
        + Add<F, Output = F>,
{
    /// Builds a [`MascotGenericFormat`] from the given data.
    pub fn build(self) -> Result<MascotGenericFormat<I, F>, String> {
        MascotGenericFormat::new(
            self.metadata_builder.build()?,
            self.data_builders
                .into_iter()
                .map(|builder| builder.build())
                .collect::<Result<Vec<_>, String>>()?,
        )
    }

    /// Returns whether the level of any of the data builders is equal to two.
    pub fn is_level_two(&self) -> bool {
        self.data_builders
            .iter()
            .any(|builder| builder.is_level_two().unwrap_or(false))
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
        + Add<F, Output = F>
        + Sub<F, Output = F>,
{
    fn can_parse_line(line: &str) -> bool {
        line == "BEGIN IONS"
            || line == "END IONS"
            || MascotGenericFormatMetadataBuilder::<I, F>::can_parse_line(line)
            || MascotGenericFormatDataBuilder::<F>::can_parse_line(line)
    }

    fn can_build(&self) -> bool {
        !self.section_open
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
        if line == "BEGIN IONS" {
            self.section_open = true;
            self.data_builders
                .push(MascotGenericFormatDataBuilder::default());
        } else if line == "END IONS" {
            // IF we have reached a "END IONS" line, then we must have previously reached a "BEGIN IONS" line.
            if !self.section_open {
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
                return Err(format!(
                    concat!(
                        "While attempting to digest line \"END IONS\": the data builder stack is not buildable. ",
                        "The current object looks like this: {self:?}"
                    ),
                    self = self
                ));
            }

            // If we are currently building a second-level fragmentation data, them when
            // we reach the end of the first-level fragmentation data, it must be buildable.
            self.section_open = false;

            if self.is_level_two() && !self.can_build() {
                return Err(format!(
                    concat!(
                        "While attempting to digest line \"END IONS\": the object is not buildable. ",
                        "Specifically, the metadata builder status is {} and the data builder stack status is {}. ",
                        "The metadata look like this: {metadata:?}.",
                    ),
                    self.metadata_builder.can_build(),
                    self.data_builders.iter().all(|builder| builder.can_build()),
                    metadata = self.metadata_builder
                ));
            }
        } else if MascotGenericFormatMetadataBuilder::<I, F>::can_parse_line(line) {
            self.metadata_builder.digest_line(line)?;
        } else if let Some(data_builder) = self.data_builders.last_mut() {
            data_builder.digest_line(line)?;
        } else {
            return Err(format!(
                concat!(
                    "While attempting to digest line \"{line}\": no data builder was found, ",
                    "meaning that the line \"{line}\" was not preceded by \"BEGIN IONS\". ",
                    "The current object looks like this: {self:?}"
                ),
                line = line,
                self = self
            ));
        }

        Ok(())
    }
}
