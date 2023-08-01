use crate::prelude::{Charge, FragmentationSpectraLevel};

pub struct MascotGenericFormatMetadata<I, F> {
    feature_id: I,
    parent_ion_mass: F,
    retention_time: F,
    charge: Charge,
    filename: Option<String>,
}
