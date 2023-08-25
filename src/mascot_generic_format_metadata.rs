use std::{fmt::Debug, ops::Add};

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct MascotGenericFormatMetadata<I, F> {
    feature_id: I,
    parent_ion_mass: F,
    retention_time: F,
    charge: Charge,
    merged_scans_metadata: Option<MergeScansMetadata<I>>,
}

impl<I: Copy + Add<Output = I> + Eq + Debug + Copy + Zero, F: StrictlyPositive + Copy>
    MascotGenericFormatMetadata<I, F>
{
    /// Creates a new [`MascotGenericFormatMetadata`].
    ///
    /// # Arguments
    /// * `feature_id` - The feature ID of the metadata.
    /// * `parent_ion_mass` - The parent ion mass of the metadata.
    /// * `retention_time` - The retention time of the metadata.
    /// * `charge` - The charge of the metadata.
    ///
    /// # Returns
    /// A new [`MascotGenericFormatMetadata`].
    ///
    /// # Errors
    /// * If `parent_ion_mass` is not strictly positive.
    /// * If `retention_time` is not strictly positive.
    ///
    /// # Examples
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    ///
    /// let feature_id = 1;
    /// let parent_ion_mass = 381.0795;
    /// let retention_time = 37.083;
    /// let charge = Charge::One;
    ///
    /// let mascot_generic_format_metadata: MascotGenericFormatMetadata<usize, f64> = MascotGenericFormatMetadata::new(
    ///     feature_id,
    ///     parent_ion_mass,
    ///     retention_time,
    ///     charge,
    ///     None,
    /// ).unwrap();
    ///
    /// assert_eq!(mascot_generic_format_metadata.feature_id(), feature_id);
    /// assert_eq!(mascot_generic_format_metadata.parent_ion_mass(), parent_ion_mass);
    /// assert_eq!(mascot_generic_format_metadata.retention_time(), retention_time);
    /// assert_eq!(mascot_generic_format_metadata.charge(), charge);
    ///
    /// assert!(
    ///     MascotGenericFormatMetadata::new(
    ///         feature_id,
    ///         -1.0,
    ///         retention_time,
    ///         charge,
    ///         None,
    ///     ).is_err()
    /// );
    ///
    /// assert!(
    ///     MascotGenericFormatMetadata::new(
    ///         feature_id,
    ///         parent_ion_mass,
    ///         -1.0,
    ///         charge,
    ///         None,
    ///     ).is_err()
    /// );
    ///
    /// ```
    ///
    pub fn new(
        feature_id: I,
        parent_ion_mass: F,
        retention_time: F,
        charge: Charge,
        merged_scans_metadata: Option<MergeScansMetadata<I>>,
    ) -> Result<Self, String> {
        if !parent_ion_mass.is_strictly_positive() {
            return Err("Could not create MascotGenericFormatMetadata: parent_ion_mass must be strictly positive".to_string());
        }

        if !retention_time.is_strictly_positive() {
            return Err("Could not create MascotGenericFormatMetadata: retention_time must be strictly positive".to_string());
        }

        Ok(Self {
            feature_id,
            parent_ion_mass,
            retention_time,
            charge,
            merged_scans_metadata,
        })
    }

    /// Returns the feature ID of the metadata.
    pub fn feature_id(&self) -> I {
        self.feature_id
    }

    /// Returns the parent ion mass of the metadata.
    pub fn parent_ion_mass(&self) -> F {
        self.parent_ion_mass
    }

    /// Returns the retention time of the metadata.
    pub fn retention_time(&self) -> F {
        self.retention_time
    }

    /// Returns the charge of the metadata.
    pub fn charge(&self) -> Charge {
        self.charge
    }

    /// Returns the number of scans removed due to low quality.
    pub fn number_of_scans_removed_due_to_low_quality(&self) -> I {
        self.merged_scans_metadata
            .as_ref()
            .map(|m| m.removed_due_to_low_quality())
            .unwrap_or(I::ZERO)
    }
}
