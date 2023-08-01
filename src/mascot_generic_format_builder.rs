use crate::prelude::*;

/// A builder for [`MascotGenericFormat`].
pub struct MascotGenericFormatBuilder<I, F> {
    feature_id: Option<I>,
    parent_ion_mass: Option<F>,
    retention_time: Option<F>,
    charge: Option<Charge>,
    fragmentation_spectra_level: Option<FragmentationSpectraLevel>,
    mass_divided_by_charge_ratios: Vec<F>,
    fragment_intensities: Vec<F>,
}

impl<I, F> Default for MascotGenericFormatBuilder<I, F> {
    fn default() -> Self {
        Self {
            feature_id: None,
            parent_ion_mass: None,
            retention_time: None,
            charge: None,
            fragmentation_spectra_level: None,
            mass_divided_by_charge_ratios: Vec::new(),
            fragment_intensities: Vec::new(),
        }
    }
}
