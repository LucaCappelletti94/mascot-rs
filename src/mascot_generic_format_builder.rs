use std::str::FromStr;

use crate::prelude::*;

/// A builder for [`MascotGenericFormat`].
pub struct MascotGenericFormatBuilder<I, F> {
    metadata_builder: MascotGenericFormatMetadataBuilder<I, F>,
    data_builders: Vec<MascotGenericFormatDataBuilder<F>>,
}

impl<I, F> MascotGenericFormatBuilder<I, F>
where
    I: Copy,
    F: Copy + StrictlyPositive,
{
    /// Creates a new [`MascotGenericFormatBuilder`].
    pub fn new() -> Self {
        Self {
            metadata_builder: MascotGenericFormatMetadataBuilder::default(),
            data_builders: Vec::new(),
        }
    }

    /// Builds a [`MascotGenericFormat`] from the given data.
    pub fn build(self) -> Result<MascotGenericFormat<I, F>, String> {
        Ok(MascotGenericFormat::new(
            self.metadata_builder.build()?,
            self.data_builders
                .into_iter()
                .map(|builder| builder.build())
                .collect::<Result<Vec<_>, String>>()?,
        ))
    }
}

impl<I, F> LineParser for MascotGenericFormatBuilder<I, F>
where
    I: Copy + FromStr + Eq,
    F: Copy + StrictlyPositive + FromStr + PartialEq,
{
    fn can_parse_line(&self, line: &str) -> bool {
        line == "BEGIN IONS"
            || line == "END IONS"
            || self.metadata_builder.can_parse_line(line)
            || self
                .data_builders
                .iter()
                .any(|builder| builder.can_parse_line(line))
    }

    /// Digests the given line.
    fn digest_line(&mut self, line: &str) -> Result<(), String> {
        if line == "BEGIN IONS" {
            self.data_builders
                .push(MascotGenericFormatDataBuilder::default());
        } else if line == "END IONS" {
            // Do nothing.
        } else if self.metadata_builder.can_parse_line(line) {
            self.metadata_builder.digest_line(line)?;
        } else {
            self.data_builders
                .last_mut()
                .expect("No data builder found.")
                .digest_line(line)?;
        }

        Ok(())
    }
}
