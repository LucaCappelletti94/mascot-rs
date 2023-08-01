use crate::prelude::{Charge, FragmentationSpectraLevel};

pub struct MascotGenericFormat<I, F> {
    feature_id: I,
    parent_ion_mass: F,
    retention_time: F,
    charge: Charge,
    fragmentation_spectra_level: FragmentationSpectraLevel,
    mass_divided_by_charge_ratios: Vec<F>,
    fragment_intensities: Vec<F>,
}
