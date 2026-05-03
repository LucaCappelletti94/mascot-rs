use alloc::{string::ToString, vec::Vec};
use core::{fmt::Debug, ops::Add, str::FromStr};

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
    peaks: Vec<(P, P)>,
    peaks_are_strictly_increasing: bool,
    section_open: bool,
}

impl<I, P: SpectrumFloat> Default for MascotGenericFormatBuilder<I, P> {
    fn default() -> Self {
        Self {
            metadata_builder: MascotGenericFormatMetadataBuilder::default(),
            peaks: Vec::new(),
            peaks_are_strictly_increasing: true,
            section_open: false,
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

        MascotGenericFormat::from_parsed_peaks(
            metadata,
            precursor_mz,
            self.peaks,
            self.peaks_are_strictly_increasing,
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
            .and_then(|value| numeric::parse_spectrum_float_lossy(value, MZ_FIELD, line))?;

        let fragment_intensity = split
            .next()
            .ok_or_else(|| MascotError::ParseField {
                field: INTENSITY_FIELD,
                line: line.to_string(),
            })
            .and_then(|value| numeric::parse_spectrum_float_lossy(value, INTENSITY_FIELD, line))?;

        MascotGenericFormat::<I, P>::push_peak_tracking_order(
            &mut self.peaks,
            &mut self.peaks_are_strictly_increasing,
            mass_divided_by_charge_ratio,
            fragment_intensity,
        );

        Ok(())
    }

    pub(super) const fn can_build(&self) -> bool {
        !self.section_open && self.metadata_builder.can_build() && !self.peaks.is_empty()
    }

    pub(super) const fn can_skip_empty_section(&self) -> bool {
        !self.section_open && self.metadata_builder.can_build() && self.peaks.is_empty()
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
        for line in [" ", "100.0", "not-a-number 1.0", "100.0 not-a-number"] {
            let mut builder = MascotGenericFormatBuilder::<usize>::default();
            builder.digest_line("BEGIN IONS")?;
            assert!(builder.digest_line(line).is_err());
        }

        for line in ["NaN 1.0", "0.0 1.0", "100.0 NaN", "100.0 0.0"] {
            let mut builder = MascotGenericFormatBuilder::<usize>::default();
            builder.digest_line("BEGIN IONS")?;
            builder.digest_line("PEPMASS=500.0")?;
            builder.digest_line("MSLEVEL=2")?;
            builder.digest_line("SCANS=-1")?;
            builder.digest_line("CHARGE=1")?;
            builder.digest_line(line)?;
            builder.digest_line("END IONS")?;
            assert!(matches!(
                builder.build(),
                Err(MascotError::SpectrumMutation(_))
            ));
        }

        Ok(())
    }
}
