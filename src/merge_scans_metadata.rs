use std::{fmt::Debug, ops::Add};

pub struct MergeScansMetadata<I> {
    scans: Vec<I>,
    removed_due_to_low_quality: I,
    removed_due_to_low_cosine: I,
}

impl<I: Default> Default for MergeScansMetadata<I> {
    fn default() -> Self {
        Self {
            scans: Vec::default(),
            removed_due_to_low_quality: I::default(),
            removed_due_to_low_cosine: I::default(),
        }
    }
}

impl<I: Add + Eq + Debug + Copy> MergeScansMetadata<I> {
    /// Returns the ids of the scans that were merged.
    ///
    /// # Example
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    ///
    /// let mascot: MergeScansMetadata<usize> = MergeScansMetadata::default();
    ///
    /// assert_eq!(mascot.scans().len(), 0);
    ///
    /// let mascot = MergeScansMetadata::new(vec![1, 2, 3], 4, 5).unwrap();
    ///
    /// assert_eq!(mascot.scans().len(), 3);
    /// assert_eq!(mascot.scans(), &[1, 2, 3]);
    /// ```
    pub fn scans(&self) -> &[I] {
        &self.scans
    }

    /// Returns the number of scans that were removed due to low quality.
    ///
    /// # Example
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    ///
    /// let mascot: MergeScansMetadata<usize> = MergeScansMetadata::default();
    ///
    /// assert_eq!(mascot.removed_due_to_low_quality(), 0);
    ///
    /// let mascot = MergeScansMetadata::new(vec![1, 2, 3], 4, 5).unwrap();
    ///
    /// assert_eq!(mascot.removed_due_to_low_quality(), 4);
    /// ```
    pub fn removed_due_to_low_quality(&self) -> I {
        self.removed_due_to_low_quality
    }

    /// Returns the number of scans that were removed due to low cosine.
    ///
    /// # Example
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    ///
    /// let mascot: MergeScansMetadata<usize> = MergeScansMetadata::default();
    ///
    /// assert_eq!(mascot.removed_due_to_low_cosine(), 0);
    ///
    /// let mascot = MergeScansMetadata::new(vec![1, 2, 3], 4, 5).unwrap();
    ///
    /// assert_eq!(mascot.removed_due_to_low_cosine(), 5);
    /// ```
    pub fn removed_due_to_low_cosine(&self) -> I {
        self.removed_due_to_low_cosine
    }

    /// Create new instance of `MergeScansMetadata`.
    ///
    /// # Example
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    ///
    /// let mascot = MergeScansMetadata::new(vec![1, 2, 3], 4, 5).unwrap();
    ///
    /// assert_eq!(mascot.scans().len(), 3);
    ///
    /// assert_eq!(mascot.removed_due_to_low_quality(), 4);
    ///
    /// assert_eq!(mascot.removed_due_to_low_cosine(), 5);
    ///
    /// let mascot = MergeScansMetadata::new(vec![], 4, 5);
    ///
    /// assert!(mascot.is_err());
    ///
    /// let mascot = MergeScansMetadata::new(vec![1, 2, 3], 4, 5);
    ///
    /// assert!(mascot.is_ok());
    ///
    /// ```
    pub fn new(
        scans: Vec<I>,
        removed_due_to_low_quality: I,
        removed_due_to_low_cosine: I,
    ) -> Result<Self, String> {
        if scans.is_empty() {
            return Err(concat!("No scans were provided.",).to_string());
        }

        Ok(Self {
            scans,
            removed_due_to_low_quality,
            removed_due_to_low_cosine,
        })
    }
}
