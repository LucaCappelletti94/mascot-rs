use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct MascotGenericFormatMetadata<I, F> {
    feature_id: I,
    parent_ion_mass: F,
    retention_time: F,
    charge: Charge,
    merged_scans_metadata: Option<MergeScansMetadata<I>>,
    filename: Option<String>,
}

impl<I: Copy, F: StrictlyPositive + Copy> MascotGenericFormatMetadata<I, F> {
    /// Creates a new [`MascotGenericFormatMetadata`].
    /// 
    /// # Arguments
    /// * `feature_id` - The feature ID of the metadata.
    /// * `parent_ion_mass` - The parent ion mass of the metadata.
    /// * `retention_time` - The retention time of the metadata.
    /// * `charge` - The charge of the metadata.
    /// * `filename` - The filename of the metadata.
    /// 
    /// # Returns
    /// A new [`MascotGenericFormatMetadata`].
    /// 
    /// # Errors
    /// * If `parent_ion_mass` is not strictly positive.
    /// * If `retention_time` is not strictly positive.
    /// * If `filename` is empty.
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
    /// let filename = Some("20220513_PMA_DBGI_01_04_003.mzML".to_string());
    /// 
    /// let mascot_generic_format_metadata: MascotGenericFormatMetadata<i32, f64> = MascotGenericFormatMetadata::new(
    ///     feature_id,
    ///     parent_ion_mass,
    ///     retention_time,
    ///     charge,
    ///     None,
    ///     filename.clone(),
    /// ).unwrap();
    /// 
    /// assert_eq!(mascot_generic_format_metadata.feature_id(), feature_id);
    /// assert_eq!(mascot_generic_format_metadata.parent_ion_mass(), parent_ion_mass);
    /// assert_eq!(mascot_generic_format_metadata.retention_time(), retention_time);
    /// assert_eq!(mascot_generic_format_metadata.charge(), charge);
    /// assert_eq!(mascot_generic_format_metadata.filename(), filename.as_deref());
    /// 
    /// assert!(
    ///     MascotGenericFormatMetadata::new(
    ///         feature_id,
    ///         -1.0,
    ///         retention_time,
    ///         charge,
    ///         None,
    ///         filename.clone(),
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
    ///         filename.clone(),
    ///     ).is_err()
    /// );
    /// 
    /// assert!(
    ///     MascotGenericFormatMetadata::new(
    ///         feature_id,
    ///         parent_ion_mass,
    ///         retention_time,
    ///         charge,
    ///         None,
    ///         Some("".to_string()),
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
        filename: Option<String>,
    ) -> Result<Self, String> {
        if !parent_ion_mass.is_strictly_positive() {
            return Err("Could not create MascotGenericFormatMetadata: parent_ion_mass must be strictly positive".to_string());
        }

        if !retention_time.is_strictly_positive() {
            return Err("Could not create MascotGenericFormatMetadata: retention_time must be strictly positive".to_string());
        }

        if let Some(filename) = &filename {
            if filename.is_empty() {
                return Err("Could not create MascotGenericFormatMetadata: filename must not be empty".to_string());
            }
        }

        Ok(Self {
            feature_id,
            parent_ion_mass,
            retention_time,
            charge,
            merged_scans_metadata,
            filename,
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

    /// Returns the filename of the metadata.
    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }
}