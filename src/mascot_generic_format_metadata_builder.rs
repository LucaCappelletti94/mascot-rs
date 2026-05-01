use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::ops::Add;
use core::{fmt::Debug, str::FromStr};

use crate::mascot_generic_format_metadata::insert_sorted_arbitrary_metadata;
use crate::prelude::*;

/// Builder for metadata parsed from MGF header lines.
#[derive(Debug, Clone)]
pub struct MascotGenericFormatMetadataBuilder<I, P: SpectrumFloat = f64> {
    feature_id: Option<I>,
    level: Option<u8>,
    precursor_mz: Option<P>,
    retention_time: Option<f64>,
    charge: Option<i8>,
    merged_scan_count: Option<I>,
    retained_merged_scan_count: Option<I>,
    merged_scans_removed_due_to_low_quality: Option<I>,
    merged_scans_removed_due_to_low_cosine: Option<I>,
    merged_total_scan_count: Option<I>,
    filename: Option<String>,
    smiles: Option<Smiles>,
    ion_mode: Option<IonMode>,
    source_instrument: Option<Instrument>,
    arbitrary_metadata: Vec<(String, String)>,
}

impl<I, P: SpectrumFloat> Default for MascotGenericFormatMetadataBuilder<I, P> {
    fn default() -> Self {
        Self {
            feature_id: None,
            level: None,
            precursor_mz: None,
            retention_time: None,
            charge: None,
            merged_scan_count: None,
            retained_merged_scan_count: None,
            merged_scans_removed_due_to_low_quality: None,
            merged_scans_removed_due_to_low_cosine: None,
            merged_total_scan_count: None,
            filename: None,
            smiles: None,
            ion_mode: None,
            source_instrument: None,
            arbitrary_metadata: Vec::new(),
        }
    }
}

impl<I, P: SpectrumFloat> MascotGenericFormatMetadataBuilder<I, P> {
    const fn has_merged_scan_metadata(&self) -> bool {
        self.merged_scan_count.is_some()
            || self.retained_merged_scan_count.is_some()
            || self.merged_scans_removed_due_to_low_quality.is_some()
            || self.merged_scans_removed_due_to_low_cosine.is_some()
            || self.merged_total_scan_count.is_some()
    }

    const fn merged_scan_metadata_is_complete(&self) -> bool {
        self.merged_scan_count.is_some()
            && self.retained_merged_scan_count.is_some()
            && self.merged_scans_removed_due_to_low_quality.is_some()
            && self.merged_scans_removed_due_to_low_cosine.is_some()
            && self.merged_total_scan_count.is_some()
    }
}

impl<
        I: Copy + PartialEq + Eq + From<usize> + Debug + FromStr + Add<Output = I>,
        P: SpectrumFloat,
    > MascotGenericFormatMetadataBuilder<I, P>
{
    fn validate_merged_scan_metadata(&self) -> Result<()> {
        if !self.has_merged_scan_metadata() {
            return Ok(());
        }

        let merged_scan_count = self.merged_scan_count.ok_or(MascotError::MissingField {
            builder: "MascotGenericFormatMetadata",
            field: "merged_scan_count",
        })?;
        let retained_merged_scan_count =
            self.retained_merged_scan_count
                .ok_or(MascotError::MissingField {
                    builder: "MascotGenericFormatMetadata",
                    field: "retained_merged_scan_count",
                })?;
        let removed_due_to_low_quality =
            self.merged_scans_removed_due_to_low_quality
                .ok_or(MascotError::MissingField {
                    builder: "MascotGenericFormatMetadata",
                    field: "merged_scans_removed_due_to_low_quality",
                })?;
        let removed_due_to_low_cosine =
            self.merged_scans_removed_due_to_low_cosine
                .ok_or(MascotError::MissingField {
                    builder: "MascotGenericFormatMetadata",
                    field: "merged_scans_removed_due_to_low_cosine",
                })?;
        let total_scan_count = self
            .merged_total_scan_count
            .ok_or(MascotError::MissingField {
                builder: "MascotGenericFormatMetadata",
                field: "merged_total_scan_count",
            })?;

        if retained_merged_scan_count + removed_due_to_low_quality + removed_due_to_low_cosine
            != total_scan_count
            || merged_scan_count != retained_merged_scan_count
        {
            return Err(MascotError::MergedScanStatisticsMismatch);
        }

        Ok(())
    }

    /// Builds parsed MGF metadata and the precursor m/z.
    ///
    /// # Errors
    /// Returns an error if required fields are missing or merged-scan metadata
    /// is invalid.
    pub(super) fn build(self) -> Result<(MascotGenericFormatMetadata<I>, P)> {
        self.validate_merged_scan_metadata()?;

        let metadata = MascotGenericFormatMetadata::new_with_smiles_and_ion_mode(
            self.feature_id,
            self.level.ok_or(MascotError::MissingField {
                builder: "MascotGenericFormatMetadata",
                field: "level",
            })?,
            self.retention_time,
            self.charge.ok_or(MascotError::MissingField {
                builder: "MascotGenericFormatMetadata",
                field: "charge",
            })?,
            self.filename,
            self.smiles,
            self.ion_mode,
        )?
        .with_source_instrument(self.source_instrument)
        .with_arbitrary_metadata(self.arbitrary_metadata);
        let precursor_mz = self.precursor_mz.ok_or(MascotError::MissingField {
            builder: "MascotGenericFormat",
            field: "precursor_mz",
        })?;

        Ok((metadata, precursor_mz))
    }
}

impl<I: FromStr + Eq + Copy + Add<Output = I> + From<usize>, P: SpectrumFloat>
    MascotGenericFormatMetadataBuilder<I, P>
{
    fn digest_feature_id_line(&mut self, stripped: &str, line: &str) -> Result<()> {
        let feature_id = stripped.parse::<I>().map_err(|_| MascotError::ParseField {
            field: "feature ID",
            line: line.to_string(),
        })?;
        match self.feature_id {
            Some(observed_feature_id) if observed_feature_id != feature_id => {
                Err(MascotError::ConflictingField {
                    field: "feature_id",
                    line: line.to_string(),
                })
            }
            Some(_) => Ok(()),
            None => {
                self.feature_id = Some(feature_id);
                Ok(())
            }
        }
    }

    fn digest_precursor_mz_line(&mut self, stripped: &str, line: &str) -> Result<()> {
        let precursor_mz = stripped
            .parse::<f64>()
            .map_err(|_| MascotError::ParseField {
                field: "precursor m/z",
                line: line.to_string(),
            })?;
        if !precursor_mz.is_finite() {
            return Err(MascotError::NonFiniteField {
                field: "precursor m/z",
                line: line.to_string(),
            });
        }
        if precursor_mz <= 0.0 {
            return Err(MascotError::NonPositiveField {
                field: "precursor m/z",
                line: line.to_string(),
            });
        }
        let precursor_mz = P::from_f64(precursor_mz).ok_or_else(|| {
            MascotError::UnrepresentablePrecisionField {
                field: "precursor m/z",
                line: line.to_string(),
            }
        })?;
        if precursor_mz.to_f64() <= 0.0 {
            return Err(MascotError::NonPositiveField {
                field: "precursor m/z",
                line: line.to_string(),
            });
        }

        match self.precursor_mz {
            Some(observed_precursor_mz)
                if precursor_mz.to_f64().to_bits() != observed_precursor_mz.to_f64().to_bits() =>
            {
                Err(MascotError::ConflictingField {
                    field: "precursor_mz",
                    line: line.to_string(),
                })
            }
            Some(_) => Ok(()),
            None => {
                self.precursor_mz = Some(precursor_mz);
                Ok(())
            }
        }
    }

    fn parse_ms_level(line: &str) -> Result<u8> {
        let level = line
            .strip_prefix("MSLEVEL=")
            .ok_or_else(|| MascotError::ParseField {
                field: "fragmentation level",
                line: line.to_string(),
            })?
            .parse::<u8>()
            .map_err(|_| MascotError::ParseField {
                field: "fragmentation level",
                line: line.to_string(),
            })?;

        if level == 0 {
            return Err(MascotError::NonPositiveField {
                field: "fragmentation level",
                line: line.to_string(),
            });
        }

        Ok(level)
    }

    fn digest_ms_level_line(&mut self, line: &str) -> Result<()> {
        let level = Self::parse_ms_level(line)?;
        match self.level {
            Some(observed_level) if observed_level != level => Err(MascotError::ConflictingField {
                field: "level",
                line: line.to_string(),
            }),
            Some(_) => Ok(()),
            None => {
                self.level = Some(level);
                Ok(())
            }
        }
    }

    fn digest_scans_line(&mut self, stripped: &str, line: &str) -> Result<()> {
        if stripped == "-1" {
            return Ok(());
        }

        let scans = stripped.parse::<I>().map_err(|_| MascotError::ParseField {
            field: "scans",
            line: line.to_string(),
        })?;
        match self.feature_id {
            Some(feature_id) if scans != feature_id => Err(MascotError::ScanFeatureIdMismatch {
                line: line.to_string(),
            }),
            Some(_) => Ok(()),
            None => {
                self.feature_id = Some(scans);
                Ok(())
            }
        }
    }

    fn parse_trailing_sign_charge(magnitude: &str, sign: i8, line: &str) -> Result<i8> {
        if magnitude.starts_with('+') || magnitude.starts_with('-') {
            return Err(MascotError::InvalidCharge {
                line: line.to_string(),
                reason: "signed magnitude is ambiguous",
            });
        }

        let magnitude = magnitude
            .parse::<u8>()
            .map_err(|_| MascotError::InvalidCharge {
                line: line.to_string(),
                reason: "could not parse charge magnitude",
            })?;
        if sign.is_positive() {
            i8::try_from(magnitude).map_err(|_| MascotError::InvalidCharge {
                line: line.to_string(),
                reason: "positive charge is out of range",
            })
        } else if magnitude == 128 {
            Ok(i8::MIN)
        } else {
            i8::try_from(magnitude)
                .map(|charge| -charge)
                .map_err(|_| MascotError::InvalidCharge {
                    line: line.to_string(),
                    reason: "negative charge is out of range",
                })
        }
    }

    fn parse_charge_line(line: &str) -> Result<i8> {
        let charge = line
            .strip_prefix("CHARGE=")
            .ok_or_else(|| MascotError::InvalidCharge {
                line: line.to_string(),
                reason: "missing prefix",
            })?;

        let charge = if let Some(magnitude) = charge.strip_suffix('+') {
            Self::parse_trailing_sign_charge(magnitude, 1, line)?
        } else if let Some(magnitude) = charge.strip_suffix('-') {
            Self::parse_trailing_sign_charge(magnitude, -1, line)?
        } else {
            charge
                .parse::<i8>()
                .map_err(|_| MascotError::InvalidCharge {
                    line: line.to_string(),
                    reason: "could not parse charge",
                })?
        };

        Ok(charge)
    }

    fn digest_charge_line(&mut self, line: &str) -> Result<()> {
        let charge = Self::parse_charge_line(line)?;
        match self.charge {
            Some(observed_charge) if observed_charge != charge => {
                Err(MascotError::ConflictingField {
                    field: "charge",
                    line: line.to_string(),
                })
            }
            Some(_) => Ok(()),
            None => {
                self.charge = Some(charge);
                Ok(())
            }
        }
    }

    fn digest_retention_time_line(&mut self, stripped: &str, line: &str) -> Result<()> {
        let retention_time = stripped
            .parse::<f64>()
            .map_err(|_| MascotError::ParseField {
                field: "retention time",
                line: line.to_string(),
            })?;
        if !retention_time.is_finite() {
            return Err(MascotError::NonFiniteField {
                field: "retention time",
                line: line.to_string(),
            });
        }
        if retention_time <= 0.0 {
            return Err(MascotError::NonPositiveField {
                field: "retention time",
                line: line.to_string(),
            });
        }
        match self.retention_time {
            Some(observed_retention_time)
                if observed_retention_time.to_bits() != retention_time.to_bits() =>
            {
                Err(MascotError::ConflictingField {
                    field: "retention_time",
                    line: line.to_string(),
                })
            }
            Some(_) => Ok(()),
            None => {
                self.retention_time = Some(retention_time);
                Ok(())
            }
        }
    }

    fn digest_filename_line(&mut self, stripped: &str, line: &str) -> Result<()> {
        let filename = stripped.to_string();
        match self.filename.as_ref() {
            Some(observed_filename) if observed_filename != &filename => {
                Err(MascotError::ConflictingField {
                    field: "filename",
                    line: line.to_string(),
                })
            }
            Some(_) => Ok(()),
            None => {
                self.filename = Some(filename);
                Ok(())
            }
        }
    }

    const fn missing_optional_metadata_value(stripped: &str) -> bool {
        stripped.is_empty()
            || stripped.eq_ignore_ascii_case("N/A")
            || stripped.eq_ignore_ascii_case("NA")
            || stripped.eq_ignore_ascii_case("NONE")
            || stripped.eq_ignore_ascii_case("NULL")
    }

    fn digest_smiles_line(&mut self, stripped: &str, line: &str) -> Result<()> {
        let stripped = stripped.trim();
        if Self::missing_optional_metadata_value(stripped) {
            return Ok(());
        }

        let smiles = stripped
            .parse::<Smiles>()
            .map_err(|error| MascotError::InvalidSmiles {
                line: line.to_string(),
                error,
            })?;
        match self.smiles.as_ref() {
            Some(observed_smiles) if observed_smiles.to_string() != smiles.to_string() => {
                Err(MascotError::ConflictingField {
                    field: "smiles",
                    line: line.to_string(),
                })
            }
            Some(_) => Ok(()),
            None => {
                self.smiles = Some(smiles);
                Ok(())
            }
        }
    }

    fn digest_ion_mode_line(&mut self, stripped: &str, line: &str) -> Result<()> {
        let stripped = stripped.trim();
        if Self::missing_optional_metadata_value(stripped) {
            return Ok(());
        }

        let ion_mode = stripped
            .parse::<IonMode>()
            .map_err(|_| MascotError::ParseField {
                field: "ion mode",
                line: line.to_string(),
            })?;
        match self.ion_mode {
            Some(observed_ion_mode) if observed_ion_mode != ion_mode => {
                Err(MascotError::ConflictingField {
                    field: "ion_mode",
                    line: line.to_string(),
                })
            }
            Some(_) => Ok(()),
            None => {
                self.ion_mode = Some(ion_mode);
                Ok(())
            }
        }
    }

    fn digest_source_instrument_line(&mut self, stripped: &str, line: &str) -> Result<()> {
        let stripped = stripped.trim();
        if Self::missing_optional_metadata_value(stripped)
            || stripped.eq_ignore_ascii_case("N/A-N/A")
        {
            return Ok(());
        }

        let source_instrument =
            stripped
                .parse::<Instrument>()
                .map_err(|_| MascotError::ParseField {
                    field: "source instrument",
                    line: line.to_string(),
                })?;
        match self.source_instrument {
            Some(observed_source_instrument) if observed_source_instrument != source_instrument => {
                Err(MascotError::ConflictingField {
                    field: "source_instrument",
                    line: line.to_string(),
                })
            }
            Some(_) => Ok(()),
            None => {
                self.source_instrument = Some(source_instrument);
                Ok(())
            }
        }
    }

    pub(super) fn digest_arbitrary_metadata_line(&mut self, line: &str) -> Result<()> {
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| Self::unsupported_merged_scan_line_error(line))?;
        let _ = insert_sorted_arbitrary_metadata(
            &mut self.arbitrary_metadata,
            key.to_string(),
            value.to_string(),
        );
        Ok(())
    }

    fn is_merged_scan_metadata_line(line: &str) -> bool {
        line.starts_with("MERGED_SCANS=") || line.starts_with("MERGED_STATS=")
    }

    fn unsupported_merged_scan_line_error(line: &str) -> MascotError {
        MascotError::UnsupportedLine {
            parser: "MascotGenericFormatMetadataBuilder",
            line: line.to_string(),
        }
    }

    fn parse_merged_scan_count(value: &str, line: &str, label: &'static str) -> Result<I> {
        value
            .trim()
            .parse::<I>()
            .map_err(|_| MascotError::ParseField {
                field: label,
                line: line.to_string(),
            })
    }

    fn parse_first_merged_scan_count(fragment: &str, line: &str, label: &'static str) -> Result<I> {
        let value = fragment
            .split_whitespace()
            .next()
            .ok_or_else(|| Self::unsupported_merged_scan_line_error(line))?;
        Self::parse_merged_scan_count(value, line, label)
    }

    fn digest_merged_scans_line(&mut self, line: &str) -> Result<()> {
        let stripped = line
            .strip_prefix("MERGED_SCANS=")
            .ok_or_else(|| Self::unsupported_merged_scan_line_error(line))?;
        let mut scan_count = 0_usize;
        for scan in stripped.split(',') {
            scan.parse::<I>().map_err(|_| MascotError::ParseField {
                field: "merged scan numbers",
                line: line.to_string(),
            })?;
            scan_count += 1;
        }

        let scan_count = I::from(scan_count);
        if self
            .retained_merged_scan_count
            .is_some_and(|retained_count| retained_count != scan_count)
        {
            return Err(MascotError::MergedScanStatisticsMismatch);
        }
        self.merged_scan_count = Some(scan_count);
        Ok(())
    }

    fn digest_merged_stats_line(&mut self, line: &str) -> Result<()> {
        let stripped = line
            .strip_prefix("MERGED_STATS=")
            .ok_or_else(|| Self::unsupported_merged_scan_line_error(line))?;
        let (fraction, removed_scans) = stripped
            .split_once('(')
            .ok_or_else(|| Self::unsupported_merged_scan_line_error(stripped))?;
        let (scans_merged, total_scans) = fraction
            .split_once('/')
            .ok_or_else(|| Self::unsupported_merged_scan_line_error(stripped))?;
        let (low_quality, low_cosine) = removed_scans
            .split_once(',')
            .ok_or_else(|| Self::unsupported_merged_scan_line_error(stripped))?;

        let scans_merged = Self::parse_merged_scan_count(
            scans_merged,
            stripped,
            "the number of scans that were merged",
        )?;
        let total_scans =
            Self::parse_merged_scan_count(total_scans, stripped, "the total number of scans")?;
        let removed_due_to_low_quality = Self::parse_first_merged_scan_count(
            low_quality,
            stripped,
            "the number of scans that were removed due to low quality",
        )?;
        let removed_due_to_low_cosine = Self::parse_first_merged_scan_count(
            low_cosine,
            stripped,
            "the number of scans that were removed due to low cosine",
        )?;

        if scans_merged + removed_due_to_low_quality + removed_due_to_low_cosine != total_scans {
            return Err(MascotError::MergedScanStatisticsMismatch);
        }
        if self
            .merged_scan_count
            .is_some_and(|scan_count| scan_count != scans_merged)
        {
            return Err(MascotError::MergedScanStatisticsMismatch);
        }

        self.retained_merged_scan_count = Some(scans_merged);
        self.merged_scans_removed_due_to_low_quality = Some(removed_due_to_low_quality);
        self.merged_scans_removed_due_to_low_cosine = Some(removed_due_to_low_cosine);
        self.merged_total_scan_count = Some(total_scans);
        Ok(())
    }

    fn digest_merge_scans_line(&mut self, line: &str) -> Result<()> {
        if line.starts_with("MERGED_SCANS=") {
            return self.digest_merged_scans_line(line);
        }

        if line.starts_with("MERGED_STATS=") {
            return self.digest_merged_stats_line(line);
        }

        Err(Self::unsupported_merged_scan_line_error(line))
    }
}

impl<I: FromStr + Eq + Copy + Add<Output = I> + From<usize>, P: SpectrumFloat>
    MascotGenericFormatMetadataBuilder<I, P>
{
    /// Returns whether the line can be parsed by this parser.
    ///
    /// # Arguments
    /// * `line` - The line to parse.
    ///
    /// # Examples
    /// The known-field parser should be able to parse any of the following lines:
    /// feature IDs, precursor m/z values, scan ids, charges, retention times,
    /// filenames, SMILES, ion-mode metadata, source-instrument metadata,
    /// partial-read scan markers, and merged-scan metadata lines.
    pub(super) fn can_parse_line(line: &str) -> bool {
        line.starts_with("FEATURE_ID=")
            || line.starts_with("PEPMASS=")
            || line.starts_with("MSLEVEL=")
            || line.starts_with("SCANS=")
            || line.starts_with("RTINSECONDS=")
            || line.starts_with("FILENAME=")
            || line.starts_with("SMILES=")
            || line.starts_with("IONMODE=")
            || line.starts_with("SOURCE_INSTRUMENT=")
            || line.starts_with("CHARGE=")
            || Self::is_merged_scan_metadata_line(line)
    }

    /// Returns whether the line can be stored as arbitrary metadata.
    pub(super) fn can_parse_arbitrary_metadata_line(line: &str) -> bool {
        line.contains('=')
    }

    /// Returns whether the parser can build a [`MascotGenericFormatMetadata`] from the lines
    pub(super) const fn can_build(&self) -> bool {
        self.level.is_some()
            && self.precursor_mz.is_some()
            && self.charge.is_some()
            && (!self.has_merged_scan_metadata() || self.merged_scan_metadata_is_complete())
    }

    /// Parses a line to a [`MascotGenericFormatMetadataBuilder`].
    ///
    /// # Arguments
    /// * `line` - The line to parse.
    ///
    /// # Error
    /// * If feature ID was already encountered and it is now different.
    /// * If scans is not -1 or equal to the feature ID.
    /// * If PEPMASS was already encountered and it is now different.
    /// * If rtinseconds was already encountered and it is now different.
    pub(super) fn digest_line(&mut self, line: &str) -> Result<()> {
        if let Some(stripped) = line.strip_prefix("FEATURE_ID=") {
            return self.digest_feature_id_line(stripped, line);
        }

        if let Some(stripped) = line.strip_prefix("PEPMASS=") {
            return self.digest_precursor_mz_line(stripped, line);
        }

        if line.starts_with("MSLEVEL=") {
            return self.digest_ms_level_line(line);
        }

        if let Some(stripped) = line.strip_prefix("SCANS=") {
            return self.digest_scans_line(stripped, line);
        }

        if line.starts_with("CHARGE=") {
            return self.digest_charge_line(line);
        }

        if let Some(stripped) = line.strip_prefix("RTINSECONDS=") {
            return self.digest_retention_time_line(stripped, line);
        }

        if let Some(stripped) = line.strip_prefix("FILENAME=") {
            return self.digest_filename_line(stripped, line);
        }

        if let Some(stripped) = line.strip_prefix("SMILES=") {
            return self.digest_smiles_line(stripped, line);
        }

        if let Some(stripped) = line.strip_prefix("IONMODE=") {
            return self.digest_ion_mode_line(stripped, line);
        }

        if let Some(stripped) = line.strip_prefix("SOURCE_INSTRUMENT=") {
            return self.digest_source_instrument_line(stripped, line);
        }

        if Self::is_merged_scan_metadata_line(line) {
            return self.digest_merge_scans_line(line);
        }

        Err(MascotError::UnsupportedLine {
            parser: "MascotGenericFormatMetadataBuilder",
            line: line.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_expected_metadata_lines() {
        for line in [
            "FEATURE_ID=1",
            "PEPMASS=381.0795",
            "MSLEVEL=2",
            "SCANS=1",
            "CHARGE=1",
            "CHARGE=1+",
            "CHARGE=2+",
            "CHARGE=3+",
            "CHARGE=4+",
            "CHARGE=5+",
            "CHARGE=-1",
            "CHARGE=1-",
            "RTINSECONDS=37.083",
            "FILENAME=20220513_PMA_DBGI_01_04_003.mzML",
            "SMILES=CCO",
            "SMILES=N/A",
            "IONMODE=Positive",
            "IONMODE=Negative",
            "IONMODE=N/A",
            "SOURCE_INSTRUMENT=LC-ESI-Orbitrap",
            "SOURCE_INSTRUMENT=N/A-N/A",
            "SCANS=-1",
        ] {
            assert!(MascotGenericFormatMetadataBuilder::<usize>::can_parse_line(
                line
            ));
        }
    }

    #[test]
    fn builds_metadata_from_lines() -> Result<()> {
        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();

        parser.digest_line("FEATURE_ID=1")?;
        parser.digest_line("PEPMASS=381.0795")?;
        parser.digest_line("MSLEVEL=2")?;
        parser.digest_line("SCANS=1")?;
        parser.digest_line("CHARGE=1")?;
        parser.digest_line("MERGED_SCANS=1567,1540")?;
        parser.digest_line(
            "MERGED_STATS=2 / 2 (0 removed due to low quality, 0 removed due to low cosine).",
        )?;
        parser.digest_line("RTINSECONDS=37.083")?;
        parser.digest_line("FILENAME=20220513_PMA_DBGI_01_04_003.mzML")?;
        parser.digest_line("SMILES=CCO")?;
        parser.digest_line("IONMODE=Positive")?;
        parser.digest_line("SOURCE_INSTRUMENT=LC-ESI-Orbitrap")?;

        let (mascot_generic_format_metadata, precursor_mz) = parser.build()?;

        assert_eq!(mascot_generic_format_metadata.feature_id(), Some(1));
        assert_eq!(mascot_generic_format_metadata.level(), 2);
        assert_eq!(precursor_mz.to_bits(), 381.0795_f64.to_bits());
        assert_eq!(
            mascot_generic_format_metadata
                .retention_time()
                .map(f64::to_bits),
            Some(37.083_f64.to_bits())
        );
        assert_eq!(mascot_generic_format_metadata.charge(), 1);
        assert_eq!(
            mascot_generic_format_metadata.filename(),
            Some("20220513_PMA_DBGI_01_04_003.mzML")
        );
        assert_eq!(
            mascot_generic_format_metadata
                .smiles()
                .map(ToString::to_string)
                .as_deref(),
            Some("CCO")
        );
        assert_eq!(
            mascot_generic_format_metadata.ion_mode(),
            Some(IonMode::Positive)
        );
        assert_eq!(
            mascot_generic_format_metadata.source_instrument(),
            Some(Instrument::Orbitrap)
        );
        Ok(())
    }

    #[test]
    fn stores_arbitrary_metadata_sorted_by_key() -> Result<()> {
        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();

        parser.digest_line("FEATURE_ID=1")?;
        parser.digest_line("PEPMASS=381.0795")?;
        parser.digest_line("MSLEVEL=2")?;
        parser.digest_line("SCANS=1")?;
        parser.digest_line("CHARGE=1")?;
        parser.digest_arbitrary_metadata_line("SPECTRUMID=CCMSLIB00000000001")?;
        parser.digest_arbitrary_metadata_line("NAME=Old name")?;
        parser.digest_arbitrary_metadata_line("NAME=New name")?;

        let (mascot_generic_format_metadata, _precursor_mz) = parser.build()?;

        assert_eq!(
            mascot_generic_format_metadata.arbitrary_metadata(),
            &[
                ("NAME".to_string(), "New name".to_string()),
                ("SPECTRUMID".to_string(), "CCMSLIB00000000001".to_string(),),
            ]
        );
        assert_eq!(
            mascot_generic_format_metadata.arbitrary_metadata_value("NAME"),
            Some("New name")
        );
        assert_eq!(
            mascot_generic_format_metadata.arbitrary_metadata_value("UNKNOWN"),
            None
        );
        Ok(())
    }

    #[test]
    fn builds_metadata_without_feature_id() -> Result<()> {
        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();

        parser.digest_line("PEPMASS=381.0795")?;
        parser.digest_line("MSLEVEL=2")?;
        parser.digest_line("SCANS=-1")?;
        parser.digest_line("CHARGE=1")?;

        let (mascot_generic_format_metadata, precursor_mz) = parser.build()?;

        assert_eq!(mascot_generic_format_metadata.feature_id(), None);
        assert_eq!(mascot_generic_format_metadata.level(), 2);
        assert_eq!(precursor_mz.to_bits(), 381.0795_f64.to_bits());
        assert_eq!(mascot_generic_format_metadata.charge(), 1);
        Ok(())
    }

    #[test]
    fn build_returns_precursor_mz_in_requested_precision() -> Result<()> {
        let mut parser = MascotGenericFormatMetadataBuilder::<usize, f32>::default();

        parser.digest_line("FEATURE_ID=1")?;
        parser.digest_line("PEPMASS=381.0795")?;
        parser.digest_line("MSLEVEL=2")?;
        parser.digest_line("SCANS=1")?;
        parser.digest_line("CHARGE=1")?;

        let (_mascot_generic_format_metadata, precursor_mz) = parser.build()?;

        assert_eq!(precursor_mz.to_bits(), 381.0795_f32.to_bits());
        Ok(())
    }

    #[test]
    fn partial_merged_scan_metadata_prevents_building() -> Result<()> {
        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();

        parser.digest_line("FEATURE_ID=1")?;
        parser.digest_line("PEPMASS=381.0795")?;
        parser.digest_line("MSLEVEL=2")?;
        parser.digest_line("SCANS=1")?;
        parser.digest_line("CHARGE=1")?;
        parser.digest_line("RTINSECONDS=37.083")?;
        parser.digest_line("MERGED_SCANS=1567,1540")?;

        assert!(!parser.can_build());
        Ok(())
    }

    #[test]
    fn rejects_mismatched_merged_scan_metadata() -> Result<()> {
        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();
        parser.digest_line("MERGED_SCANS=1567,1540")?;

        assert!(matches!(
            parser.digest_line(
                "MERGED_STATS=1 / 1 (0 removed due to low quality, 0 removed due to low cosine)."
            ),
            Err(MascotError::MergedScanStatisticsMismatch)
        ));
        Ok(())
    }

    #[test]
    fn rejects_conflicting_feature_id() -> Result<()> {
        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();
        parser.digest_line("FEATURE_ID=1")?;
        assert!(parser.digest_line("FEATURE_ID=2").is_err());
        Ok(())
    }

    #[test]
    fn rejects_conflicting_scan_id() -> Result<()> {
        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();
        parser.digest_line("FEATURE_ID=1")?;
        assert!(parser.digest_line("SCANS=2").is_err());
        Ok(())
    }

    #[test]
    fn rejects_conflicting_precursor_mz() -> Result<()> {
        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();
        parser.digest_line("PEPMASS=381.0795")?;
        assert!(parser.digest_line("PEPMASS=381.0796").is_err());
        Ok(())
    }

    #[test]
    fn rejects_conflicting_retention_time() -> Result<()> {
        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();
        parser.digest_line("RTINSECONDS=37.083")?;
        assert!(parser.digest_line("RTINSECONDS=37.084").is_err());
        Ok(())
    }

    #[test]
    fn rejects_conflicting_charge() -> Result<()> {
        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();
        parser.digest_line("CHARGE=1")?;
        assert!(parser.digest_line("CHARGE=2").is_err());
        Ok(())
    }

    #[test]
    fn rejects_conflicting_smiles() -> Result<()> {
        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();
        parser.digest_line("SMILES=CCO")?;
        assert!(parser.digest_line("SMILES=CCC").is_err());
        Ok(())
    }

    #[test]
    fn rejects_conflicting_ion_mode() -> Result<()> {
        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();
        parser.digest_line("IONMODE=Positive")?;
        assert!(parser.digest_line("IONMODE=Negative").is_err());
        Ok(())
    }

    #[test]
    fn rejects_conflicting_source_instrument() -> Result<()> {
        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();
        parser.digest_line("SOURCE_INSTRUMENT=LC-ESI-Orbitrap")?;
        parser.digest_line("SOURCE_INSTRUMENT=ESI-Orbitrap")?;
        assert!(parser.digest_line("SOURCE_INSTRUMENT=ESI-qTof").is_err());
        Ok(())
    }

    #[test]
    fn accepts_repeated_identical_metadata_lines() -> Result<()> {
        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();

        for line in [
            "FEATURE_ID=1",
            "FEATURE_ID=1",
            "PEPMASS=381.0795",
            "PEPMASS=381.0795",
            "MSLEVEL=2",
            "MSLEVEL=2",
            "SCANS=1",
            "SCANS=1",
            "CHARGE=1",
            "CHARGE=1",
            "RTINSECONDS=37.083",
            "RTINSECONDS=37.083",
            "FILENAME=20220513_PMA_DBGI_01_04_003.mzML",
            "FILENAME=20220513_PMA_DBGI_01_04_003.mzML",
            "SMILES=CCO",
            "SMILES=CCO",
            "SMILES=N/A",
            "IONMODE=Positive",
            "IONMODE=pos",
            "IONMODE=N/A",
            "SOURCE_INSTRUMENT=LC-ESI-qTof",
            "SOURCE_INSTRUMENT=ESI-LC-ESI-QTOF",
            "SOURCE_INSTRUMENT=N/A-N/A",
        ] {
            parser.digest_line(line)?;
        }

        assert!(parser.can_build());
        Ok(())
    }

    #[test]
    fn rejects_invalid_scalar_metadata_lines() {
        for line in [
            "FEATURE_ID=not-a-number",
            "PEPMASS=not-a-number",
            "PEPMASS=NaN",
            "PEPMASS=0",
            "MSLEVEL=not-a-number",
            "MSLEVEL=0",
            "SCANS=not-a-number",
            "CHARGE=+1+",
            "CHARGE=abc+",
            "CHARGE=not-a-number",
            "CHARGE=128+",
            "CHARGE=129+",
            "CHARGE=129-",
            "RTINSECONDS=not-a-number",
            "RTINSECONDS=NaN",
            "RTINSECONDS=0",
            "SMILES=C(",
            "IONMODE=unknown",
            "MERGED_SCANS=not-a-number",
            "MERGED_STATS=not-a-fraction",
            "MERGED_STATS=1 / 1",
            "MERGED_STATS=1 / 1 (",
            "MERGED_STATS=one / 1 (0 removed due to low quality, 0 removed due to low cosine).",
            "MERGED_STATS=1 / one (0 removed due to low quality, 0 removed due to low cosine).",
            "MERGED_STATS=1 / 1 (one removed due to low quality, 0 removed due to low cosine).",
            "MERGED_STATS=1 / 1 (0 removed due to low quality, one removed due to low cosine).",
            "MERGED_STATS=1 / 2 (0 removed due to low quality, 0 removed due to low cosine).",
            "UNKNOWN=1",
        ] {
            let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();
            assert!(parser.digest_line(line).is_err(), "{line}");
        }

        assert!(MascotGenericFormatMetadataBuilder::<usize>::parse_ms_level("LEVEL=2").is_err());

        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();
        assert!(parser.digest_merge_scans_line("MERGED_OTHER=1").is_err());
    }

    #[test]
    fn parses_minimum_negative_charge() -> Result<()> {
        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();
        parser.digest_line("CHARGE=128-")?;
        assert_eq!(parser.charge, Some(i8::MIN));
        Ok(())
    }

    #[test]
    fn reports_missing_required_fields_on_build() -> Result<()> {
        let parser = MascotGenericFormatMetadataBuilder::<usize>::default();
        assert!(matches!(
            parser.build(),
            Err(MascotError::MissingField { field: "level", .. })
        ));

        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();
        parser.digest_line("MSLEVEL=2")?;
        parser.digest_line("CHARGE=1")?;
        assert!(matches!(
            parser.build(),
            Err(MascotError::MissingField {
                field: "precursor_mz",
                ..
            })
        ));

        let mut parser = MascotGenericFormatMetadataBuilder::<usize>::default();
        parser.digest_line("PEPMASS=381.0795")?;
        parser.digest_line("MSLEVEL=2")?;
        assert!(matches!(
            parser.build(),
            Err(MascotError::MissingField {
                field: "charge",
                ..
            })
        ));

        Ok(())
    }

    #[test]
    fn validates_complete_merged_scan_metadata_on_build() {
        let parser = MascotGenericFormatMetadataBuilder {
            feature_id: Some(1_usize),
            level: Some(2),
            precursor_mz: Some(381.0795),
            retention_time: None,
            charge: Some(1),
            merged_scan_count: Some(1),
            retained_merged_scan_count: Some(1),
            merged_scans_removed_due_to_low_quality: Some(1),
            merged_scans_removed_due_to_low_cosine: Some(0),
            merged_total_scan_count: Some(1),
            filename: None,
            smiles: None,
            ion_mode: None,
            source_instrument: None,
            arbitrary_metadata: Vec::new(),
        };

        assert!(matches!(
            parser.build(),
            Err(MascotError::MergedScanStatisticsMismatch)
        ));
    }
}
