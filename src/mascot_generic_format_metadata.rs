use alloc::string::{String, ToString};

use crate::prelude::*;

/// Metadata for one Mascot Generic Format ion block.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
pub struct MascotGenericFormatMetadata<I> {
    feature_id: I,
    level: u8,
    retention_time: Option<f64>,
    charge: i8,
    filename: Option<String>,
}

impl<I: Copy> MascotGenericFormatMetadata<I> {
    /// Creates a new [`MascotGenericFormatMetadata`].
    ///
    /// # Arguments
    /// * `feature_id` - The feature ID of the metadata.
    /// * `level` - The MS fragmentation level.
    /// * `retention_time` - The retention time of the metadata, if present.
    /// * `charge` - The precursor charge of the metadata.
    /// * `filename` - The filename of the metadata.
    ///
    /// # Returns
    /// A new [`MascotGenericFormatMetadata`].
    ///
    /// # Errors
    /// * If `level` is zero.
    /// * If `retention_time` is present but not finite and strictly positive.
    /// * If `filename` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    ///
    /// let feature_id = 1;
    /// let level = 2;
    /// let retention_time = Some(37.083);
    /// let charge = 1;
    /// let filename = Some("20220513_PMA_DBGI_01_04_003.mzML".to_string());
    ///
    /// let mascot_generic_format_metadata: MascotGenericFormatMetadata<usize> = MascotGenericFormatMetadata::new(
    ///     feature_id,
    ///     level,
    ///     retention_time,
    ///     charge,
    ///     filename.clone(),
    /// ).unwrap();
    ///
    /// assert_eq!(mascot_generic_format_metadata.feature_id(), feature_id);
    /// assert_eq!(mascot_generic_format_metadata.level(), level);
    /// assert_eq!(mascot_generic_format_metadata.retention_time(), retention_time);
    /// assert_eq!(mascot_generic_format_metadata.charge(), charge);
    /// assert_eq!(mascot_generic_format_metadata.filename(), filename.as_deref());
    ///
    /// assert!(
    ///     MascotGenericFormatMetadata::new(
    ///         feature_id,
    ///         0,
    ///         retention_time,
    ///         charge,
    ///         filename.clone(),
    ///     ).is_err()
    /// );
    ///
    /// assert!(
    ///     MascotGenericFormatMetadata::new(
    ///         feature_id,
    ///         level,
    ///         Some(-1.0),
    ///         charge,
    ///         filename.clone(),
    ///     ).is_err()
    /// );
    ///
    /// assert!(
    ///     MascotGenericFormatMetadata::new(
    ///         feature_id,
    ///         level,
    ///         retention_time,
    ///         charge,
    ///         Some("".to_string()),
    ///     ).is_err()
    /// );
    ///
    /// ```
    ///
    pub fn new(
        feature_id: I,
        level: u8,
        retention_time: Option<f64>,
        charge: i8,
        filename: Option<String>,
    ) -> Result<Self> {
        if level == 0 {
            return Err(MascotError::NonPositiveField {
                field: "fragmentation level",
                line: level.to_string(),
            });
        }

        if let Some(retention_time) = retention_time {
            if !retention_time.is_finite() || retention_time <= 0.0 {
                return if retention_time.is_finite() {
                    Err(MascotError::NonPositiveField {
                        field: "retention time",
                        line: retention_time.to_string(),
                    })
                } else {
                    Err(MascotError::NonFiniteField {
                        field: "retention time",
                        line: retention_time.to_string(),
                    })
                };
            }
        }

        if let Some(filename) = &filename {
            if filename.is_empty() {
                return Err(MascotError::EmptyFilename);
            }
        }

        Ok(Self {
            feature_id,
            level,
            retention_time,
            charge,
            filename,
        })
    }

    /// Returns the feature ID of the metadata.
    pub const fn feature_id(&self) -> I {
        self.feature_id
    }

    /// Returns the MS fragmentation level.
    pub const fn level(&self) -> u8 {
        self.level
    }

    /// Returns the retention time of the metadata.
    pub const fn retention_time(&self) -> Option<f64> {
        self.retention_time
    }

    /// Returns the charge of the metadata.
    pub const fn charge(&self) -> i8 {
        self.charge
    }

    /// Returns the filename of the metadata.
    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }
}
