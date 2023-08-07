use std::{fmt::Debug, ops::Add, str::FromStr};

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
    F: Copy + StrictlyPositive + PartialEq + PartialOrd + Debug,
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
}

impl<I, F> LineParser for MascotGenericFormatBuilder<I, F>
where
    I: Copy + FromStr + Eq + Add<Output = I> + Debug,
    F: Copy + StrictlyPositive + FromStr + PartialEq + Debug + NaN,
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
    /// let mut mascot_generic_format_builder = MascotGenericFormatBuilder::<usize, f64>::default();
    ///
    /// assert!(mascot_generic_format_builder.digest_line("BEGIN IONS").is_ok());
    /// assert!(mascot_generic_format_builder.digest_line("END IONS").is_ok());
    /// assert!(mascot_generic_format_builder.digest_line("TITLE=File:").is_err());
    /// ```
    fn digest_line(&mut self, line: &str) -> Result<(), String> {
        if line == "BEGIN IONS" {
            self.section_open = true;
            self.data_builders
                .push(MascotGenericFormatDataBuilder::default());
        } else if line == "END IONS" {
            self.section_open = false;
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
