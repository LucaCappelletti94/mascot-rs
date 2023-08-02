use crate::prelude::*;

pub struct MascotGenericFormatData<F> {
    level: FragmentationSpectraLevel,
    mass_divided_by_charge_ratios: Vec<F>,
    fragment_intensities: Vec<F>,
}

impl<F> MascotGenericFormatData<F> {

    /// Creates a new [`MascotGenericFormatData`].
    /// 
    /// # Arguments
    /// * `level` - The [`FragmentationSpectraLevel`] of the data.
    /// * `mass_divided_by_charge_ratios` - The mass divided by charge ratios of the data.
    /// * `fragment_intensities` - The fragment intensities of the data.
    /// 
    /// # Returns
    /// A new [`MascotGenericFormatData`].
    /// 
    /// # Errors
    /// * If the length of `mass_divided_by_charge_ratios` and `fragment_intensities` are not equal.
    /// * If `mass_divided_by_charge_ratios` is empty.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use mascot_rs::prelude::*;
    /// 
    /// let level = FragmentationSpectraLevel::Two;
    /// let mass_divided_by_charge_ratios = vec![60.5425, 60.5426, 60.5427];
    /// let fragment_intensities = vec![2.4E5, 2.3E5, 2.2E5];
    /// 
    /// let mascot_generic_format_data: MascotGenericFormatData<f64> = MascotGenericFormatData::new(
    ///    level,
    ///    mass_divided_by_charge_ratios.clone(),
    ///    fragment_intensities.clone(),
    /// ).unwrap();
    /// 
    /// assert_eq!(mascot_generic_format_data.level(), level);
    /// assert_eq!(mascot_generic_format_data.mass_divided_by_charge_ratios(), mass_divided_by_charge_ratios.as_slice());
    /// assert_eq!(mascot_generic_format_data.fragment_intensities(), fragment_intensities.as_slice());
    /// 
    /// assert!(
    ///     MascotGenericFormatData::new(
    ///         level,
    ///         Vec::new(),
    ///         fragment_intensities.clone(),
    ///     ).is_err()
    /// );
    /// 
    /// assert!(
    ///     MascotGenericFormatData::new(
    ///         level,
    ///         mass_divided_by_charge_ratios.clone(),
    ///         Vec::new(),
    ///     ).is_err()
    /// );
    /// ```
    /// 
    pub fn new(
        level: FragmentationSpectraLevel,
        mass_divided_by_charge_ratios: Vec<F>,
        fragment_intensities: Vec<F>,
    ) -> Result<Self, String> {
        if mass_divided_by_charge_ratios.len() != fragment_intensities.len() {
            return Err(format!(
                "Could not create MascotGenericFormatData: mass_divided_by_charge_ratios and fragment_intensities have different lengths: {} and {}",
                mass_divided_by_charge_ratios.len(),
                fragment_intensities.len(),
            ));
        }

        if mass_divided_by_charge_ratios.is_empty() {
            return Err(
                "Could not create MascotGenericFormatData: empty vectors were provided."
                    .to_string(),
            );
        }

        Ok(Self {
            level,
            mass_divided_by_charge_ratios,
            fragment_intensities,
        })
    }

    /// Returns the [`FragmentationSpectraLevel`] of the data.
    pub fn level(&self) -> FragmentationSpectraLevel {
        self.level
    }

    /// Returns the mass divided by charge ratios of the data.
    pub fn mass_divided_by_charge_ratios(&self) -> &[F] {
        &self.mass_divided_by_charge_ratios
    }

    /// Returns the fragment intensities of the data.
    pub fn fragment_intensities(&self) -> &[F] {
        &self.fragment_intensities
    }
}
