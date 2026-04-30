use std::{fmt::Debug, ops::Add, str::FromStr};

use crate::{line_parser::LineParser, prelude::*};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
/// Builder for [`MergeScansMetadata`].
///
/// # Example
/// The data structure is meant to digest lines from MGF files such as:
///
/// ```text
/// MERGED_SCANS=1567,1540
/// MERGED_STATS=2 / 2 (0 removed due to low quality, 0 removed due to low cosine).
/// ```
///
/// Where the first line, `MERGED_SCANS=1567,1540` contains a list of comma-separated scan numbers that were merged,
/// and the second line, `MERGED_STATS=2 / 2 (0 removed due to low quality, 0 removed due to low cosine)` contains
/// the number of scans that were merged, the number of scans that were removed due to low quality, and the number
/// of scans that were removed due to low cosine.
pub struct MergeScansMetadataBuilder<I> {
    scans: Vec<I>,
    removed_due_to_low_quality: Option<I>,
    removed_due_to_low_cosine: Option<I>,
    total_scans: Option<I>,
}

impl<I> Default for MergeScansMetadataBuilder<I> {
    fn default() -> Self {
        Self {
            scans: Vec::new(),
            removed_due_to_low_quality: None,
            removed_due_to_low_cosine: None,
            total_scans: None,
        }
    }
}

impl<I: FromStr + Add<Output = I> + Eq + Copy + From<usize> + Debug> MergeScansMetadataBuilder<I> {
    /// Builds parsed merged-scan metadata.
    ///
    /// # Errors
    /// Returns an error if the merged-scan statistics are incomplete or if the
    /// number of retained and removed scans does not match the total scan count.
    pub fn build(self) -> Result<MergeScansMetadata<I>, String> {
        let removed_due_to_low_quality = self.removed_due_to_low_quality.ok_or_else(|| {
            concat!(
                "No information regarding whether any scans were removed ",
                "due to low quality was provided.",
            )
            .to_string()
        })?;
        let removed_due_to_low_cosine = self.removed_due_to_low_cosine.ok_or_else(|| {
            concat!(
                "No information regarding whether any scans were removed ",
                "due to low cosine was provided.",
            )
            .to_string()
        })?;
        let total_scans = self
            .total_scans
            .ok_or_else(|| "No total scan count was provided.".to_string())?;

        // We check that the total number of scans is equal to the sum of the
        // number of scans that were merged and the number of scans that were
        // removed.
        if total_scans
            != I::from(self.scans.len()) + removed_due_to_low_quality + removed_due_to_low_cosine
        {
            return Err(concat!(
                "The sum of the number of scans that were merged ",
                "and the number of scans that were removed does not ",
                "equal the total number of scans.",
            )
            .to_string());
        }

        MergeScansMetadata::new(
            self.scans,
            removed_due_to_low_quality,
            removed_due_to_low_cosine,
        )
    }
}

impl<I: FromStr + Add<Output = I> + Eq + Copy> MergeScansMetadataBuilder<I> {
    fn unsupported_line_error(line: &str) -> String {
        format!(
            "The builder for the data structure `MergeScansMetadata` does not support the line: \"{line}\"."
        )
    }

    fn parse_count(value: &str, line: &str, label: &str) -> Result<I, String> {
        value
            .trim()
            .parse::<I>()
            .map_err(|_| format!("Failed to parse {label} from the line: \"{line}\""))
    }

    fn parse_first_count(fragment: &str, line: &str, label: &str) -> Result<I, String> {
        let value = fragment
            .split_whitespace()
            .next()
            .ok_or_else(|| Self::unsupported_line_error(line))?;
        Self::parse_count(value, line, label)
    }

    fn digest_merged_scans_line(&mut self, line: &str) -> Result<(), String> {
        let stripped = line
            .strip_prefix("MERGED_SCANS=")
            .ok_or_else(|| Self::unsupported_line_error(line))?;
        self.scans = stripped
            .split(',')
            .map(str::parse::<I>)
            .collect::<Result<Vec<I>, _>>()
            .map_err(|_| format!("Failed to parse the scan numbers from the line: \"{line}\""))?;
        Ok(())
    }

    fn digest_merged_stats_line(&mut self, line: &str) -> Result<(), String> {
        let stripped = line
            .strip_prefix("MERGED_STATS=")
            .ok_or_else(|| Self::unsupported_line_error(line))?;
        let (fraction, removed_scans) = stripped
            .split_once('(')
            .ok_or_else(|| Self::unsupported_line_error(stripped))?;
        let (scans_merged, total_scans) = fraction
            .split_once('/')
            .ok_or_else(|| Self::unsupported_line_error(stripped))?;
        let (low_quality, low_cosine) = removed_scans
            .split_once(',')
            .ok_or_else(|| Self::unsupported_line_error(stripped))?;

        let scans_merged = Self::parse_count(
            scans_merged,
            stripped,
            "the number of scans that were merged",
        )?;
        let total_scans = Self::parse_count(total_scans, stripped, "the total number of scans")?;
        let removed_due_to_low_quality = Self::parse_first_count(
            low_quality,
            stripped,
            "the number of scans that were removed due to low quality",
        )?;
        let removed_due_to_low_cosine = Self::parse_first_count(
            low_cosine,
            stripped,
            "the number of scans that were removed due to low cosine",
        )?;

        if scans_merged + removed_due_to_low_quality + removed_due_to_low_cosine != total_scans {
            return Err(format!(
                "The sum of the number of scans that were merged and the number of scans that were removed does not equal the total number of scans in the line: \"{stripped}\""
            ));
        }

        self.removed_due_to_low_cosine = Some(removed_due_to_low_cosine);
        self.removed_due_to_low_quality = Some(removed_due_to_low_quality);
        self.total_scans = Some(total_scans);
        Ok(())
    }
}

impl<I: FromStr + Add<Output = I> + Eq + Copy> LineParser for MergeScansMetadataBuilder<I> {
    /// Returns `true` if the line can be parsed by the data structure.
    ///
    /// # Example
    /// ```rust
    /// use mascot_rs::prelude::*;
    ///
    /// assert!(
    ///     MergeScansMetadataBuilder::<usize>::can_parse_line("MERGED_SCANS=1567,1540"));
    /// assert!(
    ///     MergeScansMetadataBuilder::<usize>::can_parse_line("MERGED_STATS=2 / 2 (0 removed due to low quality, 0 removed due to low cosine)."),
    ///     "The line \"MERGED_STATS=2 / 2 (0 removed due to low quality, 0 removed due to low cosine).\" should be parsable by the data structure `MergeScansMetadataBuilder`."
    /// );
    /// assert!(!MergeScansMetadataBuilder::<usize>::can_parse_line("SCANS=1567,1540"));
    /// assert!(!MergeScansMetadataBuilder::<usize>::can_parse_line("STATS=2 / 2 (0 removed due to low quality, 0 removed due to low cosine)."));
    /// ```
    fn can_parse_line(line: &str) -> bool {
        line.starts_with("MERGED_SCANS=") || line.starts_with("MERGED_STATS=")
    }

    /// Returns whether the data structure can be built.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mascot_rs::prelude::*;
    ///
    /// let mut builder: MergeScansMetadataBuilder<usize> = MergeScansMetadataBuilder::default();
    /// assert!(!builder.can_build());
    /// builder.digest_line("MERGED_SCANS=1567,1540").unwrap();
    /// assert!(!builder.can_build());
    /// builder.digest_line("MERGED_STATS=2 / 2 (0 removed due to low quality, 0 removed due to low cosine).").unwrap();
    /// assert!(builder.can_build());
    /// ```
    fn can_build(&self) -> bool {
        self.removed_due_to_low_cosine.is_some()
            && self.removed_due_to_low_quality.is_some()
            && self.total_scans.is_some()
    }

    /// Parses the line and updates the data structure.
    ///
    /// # Example
    /// ```rust
    /// use mascot_rs::prelude::*;
    ///
    /// let mut builder: MergeScansMetadataBuilder<usize> = MergeScansMetadataBuilder::default();
    /// builder.digest_line("MERGED_SCANS=1567,1540").unwrap();
    /// builder.digest_line("MERGED_STATS=2 / 2 (0 removed due to low quality, 0 removed due to low cosine).").unwrap();
    /// let metadata = builder.build().unwrap();
    ///
    /// assert_eq!(metadata.scans(), &[1567, 1540]);
    /// assert_eq!(metadata.removed_due_to_low_quality(), 0);
    /// assert_eq!(metadata.removed_due_to_low_cosine(), 0);
    /// ```
    fn digest_line(&mut self, line: &str) -> Result<(), String> {
        if line.starts_with("MERGED_SCANS=") {
            return self.digest_merged_scans_line(line);
        }

        if line.starts_with("MERGED_STATS=") {
            return self.digest_merged_stats_line(line);
        }

        Err(Self::unsupported_line_error(line))
    }
}
