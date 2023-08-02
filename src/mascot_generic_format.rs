use crate::prelude::*;

pub struct MascotGenericFormat<I, F> {
    metadata: MascotGenericFormatMetadata<I, F>,
    data: Vec<MascotGenericFormatData<F>>,
}

impl<I: Copy, F: Copy + StrictlyPositive> MascotGenericFormat<I, F> {
    pub fn new(
        metadata: MascotGenericFormatMetadata<I, F>,
        data: Vec<MascotGenericFormatData<F>>,
    ) -> Self {
        Self {
            metadata,
            data,
        }
    }

    /// Returns the feature ID of the metadata.
    pub fn feature_id(&self) -> I {
        self.metadata.feature_id()
    }

    /// Returns the parent ion mass of the metadata.
    pub fn parent_ion_mass(&self) -> F {
        self.metadata.parent_ion_mass()
    }

    /// Returns the retention time of the metadata.
    pub fn retention_time(&self) -> F {
        self.metadata.retention_time()
    }

    /// Returns the charge of the metadata.
    pub fn charge(&self) -> Charge {
        self.metadata.charge()
    }

    /// Returns the filename of the metadata.
    pub fn filename(&self) -> Option<&str> {
        self.metadata.filename()
    }
}