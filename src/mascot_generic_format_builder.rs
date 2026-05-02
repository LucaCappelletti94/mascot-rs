use alloc::{string::ToString, vec::Vec};
use core::{fmt::Debug, marker::PhantomData, ops::Add, str::FromStr};

use crate::mascot_generic_format::MascotGenericFormat;
use crate::mascot_generic_format_metadata_builder::MascotGenericFormatMetadataBuilder;
use crate::numeric;
use crate::prelude::*;

const MZ_FIELD: &str = "mass divided by charge ratio";
const INTENSITY_FIELD: &str = "fragment intensity";

#[derive(Debug, Clone)]
/// A builder for [`MascotGenericFormat`].
pub struct MascotGenericFormatBuilder<I, P: SpectrumFloat = f64> {
    metadata_builder: MascotGenericFormatMetadataBuilder<I, P>,
    mass_divided_by_charge_ratios: Vec<P>,
    fragment_intensities: Vec<P>,
    section_open: bool,
    precision: PhantomData<P>,
}

impl<I, P: SpectrumFloat> Default for MascotGenericFormatBuilder<I, P> {
    fn default() -> Self {
        Self {
            metadata_builder: MascotGenericFormatMetadataBuilder::default(),
            mass_divided_by_charge_ratios: Vec::new(),
            fragment_intensities: Vec::new(),
            section_open: false,
            precision: PhantomData,
        }
    }
}

impl<I, P: SpectrumFloat> MascotGenericFormatBuilder<I, P> {
    pub(super) const fn section_open(&self) -> bool {
        self.section_open
    }
}

impl<I, P: SpectrumFloat> MascotGenericFormatBuilder<I, P>
where
    I: Copy + Eq + Debug + Add<Output = I> + FromStr + From<usize>,
{
    /// Builds a [`MascotGenericFormat`] from the given data.
    ///
    /// # Errors
    /// Returns an error if the parsed metadata or data blocks are incomplete or
    /// invalid.
    pub(super) fn build(self) -> Result<MascotGenericFormat<I, P>> {
        let (metadata, precursor_mz) = self.metadata_builder.build()?;

        MascotGenericFormat::new(
            metadata,
            precursor_mz,
            self.mass_divided_by_charge_ratios,
            self.fragment_intensities,
        )
    }
}

impl<I, P: SpectrumFloat> MascotGenericFormatBuilder<I, P>
where
    I: Copy + Eq + Debug + Add<Output = I> + FromStr + From<usize>,
{
    fn digest_peak_line(&mut self, line: &str) -> Result<()> {
        let mut split = line.split_whitespace();

        let mass_divided_by_charge_ratio = split
            .next()
            .ok_or_else(|| MascotError::ParseField {
                field: MZ_FIELD,
                line: line.to_string(),
            })
            .and_then(|value| numeric::parse_positive_spectrum_float(value, MZ_FIELD, line))?;

        let fragment_intensity = split
            .next()
            .ok_or_else(|| MascotError::ParseField {
                field: INTENSITY_FIELD,
                line: line.to_string(),
            })
            .and_then(|value| {
                numeric::parse_positive_spectrum_float(value, INTENSITY_FIELD, line)
            })?;

        self.mass_divided_by_charge_ratios
            .push(mass_divided_by_charge_ratio);
        self.fragment_intensities.push(fragment_intensity);

        Ok(())
    }

    pub(super) const fn can_build(&self) -> bool {
        !self.section_open
            && self.metadata_builder.can_build()
            && self.mass_divided_by_charge_ratios.len() == self.fragment_intensities.len()
            && !self.mass_divided_by_charge_ratios.is_empty()
    }

    pub(super) const fn can_skip_empty_section(&self) -> bool {
        !self.section_open
            && self.metadata_builder.can_build()
            && self.mass_divided_by_charge_ratios.is_empty()
            && self.fragment_intensities.is_empty()
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
    pub(super) fn digest_line(&mut self, line: &str) -> Result<()> {
        if line == "BEGIN IONS" {
            self.section_open = true;
        } else if line == "END IONS" {
            self.section_open = false;
        } else if MascotGenericFormatMetadataBuilder::<I, P>::can_parse_line(line) {
            self.metadata_builder.digest_line(line)?;
        } else if self.section_open
            && MascotGenericFormatMetadataBuilder::<I, P>::can_parse_arbitrary_metadata_line(line)
        {
            self.metadata_builder.digest_arbitrary_metadata_line(line)?;
        } else if self.section_open {
            self.digest_peak_line(line)?;
        } else {
            return Err(MascotError::LineOutsideIonSection {
                line: line.to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digests_ion_section_boundaries_and_rejects_unknown_lines() -> Result<()> {
        let mut mascot_generic_format_builder = MascotGenericFormatBuilder::<usize>::default();

        mascot_generic_format_builder.digest_line("BEGIN IONS")?;
        mascot_generic_format_builder.digest_line("END IONS")?;
        assert!(mascot_generic_format_builder
            .digest_line("TITLE=File:")
            .is_err());
        Ok(())
    }

    #[test]
    fn rejects_invalid_peak_lines() -> Result<()> {
        for line in [
            " ",
            "100.0",
            "not-a-number 1.0",
            "100.0 not-a-number",
            "NaN 1.0",
            "0.0 1.0",
            "100.0 NaN",
            "100.0 0.0",
        ] {
            let mut builder = MascotGenericFormatBuilder::<usize>::default();
            builder.digest_line("BEGIN IONS")?;
            assert!(builder.digest_line(line).is_err());
        }

        Ok(())
    }
}
