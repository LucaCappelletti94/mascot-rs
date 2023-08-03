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
    pub fn build(self) -> Result<MergeScansMetadata<I>, String> {
        if self.removed_due_to_low_quality.is_none() {
            return Err(concat!(
                "No information regarding whether any scans were removed ",
                "due to low quality was provided.",
            )
            .to_string());
        }
        if self.removed_due_to_low_cosine.is_none() {
            return Err(concat!(
                "No information regarding whether any scans were removed ",
                "due to low cosine was provided.",
            )
            .to_string());
        }

        // We check that the total number of scans is equal to the sum of the
        // number of scans that were merged and the number of scans that were
        // removed.
        if self.total_scans.unwrap()
            != I::from(self.scans.len())
                + self.removed_due_to_low_quality.unwrap()
                + self.removed_due_to_low_cosine.unwrap()
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
            self.removed_due_to_low_quality.unwrap(),
            self.removed_due_to_low_cosine.unwrap(),
        )
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
        // This first check is meant to capture lines such as:
        //
        // ```text
        // MERGED_SCANS=1567,1540
        // ```
        //
        // We expect to populate the `scans` field with the values
        // `1567` and `1540` which represent the scan numbers that
        // were merged.
        if line.starts_with("MERGED_SCANS=") {
            let scans = line
                .trim_start_matches("MERGED_SCANS=")
                .split(',')
                .map(|scan| scan.parse::<I>())
                .collect::<Result<Vec<I>, _>>()
                .map_err(|_| {
                    format!(
                        concat!("Failed to parse the scan numbers from the line: ", "\"{}\"",),
                        line
                    )
                })?;
            self.scans = scans;
            return Ok(());
        }

        // This second check is meant to capture lines such as:
        //
        // ```text
        // MERGED_STATS=2 / 5 (1 removed due to low quality, 2 removed due to low cosine).
        // ```
        //
        // We expect to populate the `removed_due_to_low_quality` field
        // with the value `1` which represents the number of scans that
        // were removed due to low quality, and the `removed_due_to_low_cosine`
        // field with the value `2` which represents the number of scans
        // that were removed due to low cosine. We expect the total number
        // of scans to be `5` which is the sum of the number of scans that
        // were merged and the number of scans that were removed. This last
        // value is not stored in the data structure, but used in the builder to
        // check that the number of scans that were merged and the number of
        // scans that were removed add up to the total number of scans.

        if line.starts_with("MERGED_STATS=") {
            // First, we remove the prefix `MERGED_STATS=`.
            let line = line.trim_start_matches("MERGED_STATS=");
            // Then, we split the line into three parts:
            // 1. The number of scans that were merged.
            // 2. The total number of scans.
            // 3. The number of scans that were removed.
            let mut parts = line.split('(');
            // We expect the line to have two parts, the first containing
            // the fraction and the second containing the number of scans
            // that were removed. We assign the two parts and proceed to split
            // the first part into two parts:
            // 1. The number of scans that were merged.
            // 2. The total number of scans.
            let mut fraction_parts = parts.next().unwrap().split('/');

            // We obtain the number of scans that were merged and the total
            // number of scans.
            let scans_merged: I = if let Some(scans_merged) = fraction_parts.next() {
                scans_merged.trim().parse::<I>().map_err(|_| {
                    format!(
                        concat!(
                            "Failed to parse the number of scans that were merged ",
                            "from the line: ",
                            "\"{}\"",
                        ),
                        line
                    )
                })
            } else {
                Err(format!(
                    concat!(
                        "The builder for the data structure ",
                        "`MergeScansMetadata` ",
                        "does not support the line: ",
                        "\"{}\"",
                    ),
                    line,
                ))
            }?;

            // We obtain the number of scans that were merged and the total
            // number of scans.
            let total_scans: I = if let Some(total_scans) = fraction_parts.next() {
                total_scans.trim().parse::<I>().map_err(|_| {
                    format!(
                        concat!(
                            "Failed to parse the number of scans that were merged ",
                            "from the line: ",
                            "\"{}\"",
                        ),
                        line
                    )
                })
            } else {
                Err(format!(
                    concat!(
                        "The builder for the data structure ",
                        "`MergeScansMetadata` ",
                        "does not support the line: ",
                        "\"{}\"",
                    ),
                    line,
                ))
            }?;

            // We expect the fraction to have two parts, the first containing
            // the number of scans that were merged and the second containing
            // the total number of scans. We assign the two parts and proceed
            // to split the second part into two parts:
            // 1. The number of scans that were removed due to low quality.
            // 2. The number of scans that were removed due to low cosine.
            let (low_quality, low_cosine) = if let Some(removed_scans) = parts.next() {
                let mut removed_scans = removed_scans.split(',');
                let low_quality = if let Some(low_quality) = removed_scans.next() {
                    Ok(low_quality)
                } else {
                    Err(format!(
                        concat!(
                            "The builder for the data structure ",
                            "`MergeScansMetadata` ",
                            "does not extract the low quality ",
                            "scans count from the line: ",
                            "\"{}\"",
                        ),
                        line,
                    ))
                }?;

                let low_cosine = if let Some(low_cosine) = removed_scans.next() {
                    Ok(low_cosine)
                } else {
                    Err(format!(
                        concat!(
                            "The builder for the data structure ",
                            "`MergeScansMetadata` ",
                            "does not extract the low cosine ",
                            "scans count from the line: ",
                            "\"{}\"",
                        ),
                        line,
                    ))
                }?;
                Ok((low_quality, low_cosine))
            } else {
                Err(format!(
                    concat!(
                        "The builder for the data structure ",
                        "`MergeScansMetadata` ",
                        "does not support the line: ",
                        "\"{}\"",
                    ),
                    line,
                ))
            }?;

            // We expect the number of scans that were removed to have two parts,
            // the first containing the number of scans that were removed due to
            // low quality and the second containing the number of scans that were
            // removed due to low cosine. We assign the two parts and proceed to
            // parse the values.
            let removed_due_to_low_quality =
                if let Some(low_quality) = low_quality.trim().split(' ').next() {
                    low_quality.parse::<I>().map_err(|_| {
                        format!(
                            concat!(
                                "Failed to parse the number of scans that were removed ",
                                "due to low quality from the line: ",
                                "\"{}\"",
                            ),
                            line
                        )
                    })
                } else {
                    Err(format!(
                        concat!(
                            "The builder for the data structure ",
                            "`MergeScansMetadata` ",
                            "does not support the line: ",
                            "\"{}\"",
                        ),
                        line,
                    ))
                }?;

            let removed_due_to_low_cosine =
                if let Some(low_cosine) = low_cosine.trim().split(' ').next() {
                    low_cosine.parse::<I>().map_err(|_| {
                        format!(
                            concat!(
                                "Failed to parse the number of scans that were removed ",
                                "due to low cosine from the line: ",
                                "\"{}\"",
                            ),
                            line
                        )
                    })
                } else {
                    Err(format!(
                        concat!(
                            "The builder for the data structure ",
                            "`MergeScansMetadata` ",
                            "does not support the line: ",
                            "\"{}\"",
                        ),
                        line,
                    ))
                }?;

            // We check whether the sum of removed scans plus the number of scans
            // that were merged equals the total number of scans.
            if scans_merged + removed_due_to_low_quality + removed_due_to_low_cosine != total_scans
            {
                return Err(format!(
                    concat!(
                        "The sum of the number of scans that were merged ",
                        "and the number of scans that were removed does not ",
                        "equal the total number of scans in the line: ",
                        "\"{}\"",
                    ),
                    line,
                ));
            }

            self.removed_due_to_low_cosine = Some(removed_due_to_low_cosine);
            self.removed_due_to_low_quality = Some(removed_due_to_low_quality);
            self.total_scans = Some(total_scans);
            return Ok(());
        }

        Err(format!(
            concat!(
                "The builder for the data structure ",
                "`MergeScansMetadata` ",
                "does not support the line: ",
                "\"",
                "{}",
                "\".",
            ),
            line,
        ))
    }
}
