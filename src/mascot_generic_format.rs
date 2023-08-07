use crate::prelude::*;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, Index, IndexMut};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct MascotGenericFormat<I, F> {
    metadata: MascotGenericFormatMetadata<I, F>,
    data: Vec<MascotGenericFormatData<F>>,
}

impl<
        I: Copy + Zero + PartialEq + Debug + Add<Output = I> + Eq,
        F: Copy + StrictlyPositive + PartialEq + PartialOrd + Debug,
    > MascotGenericFormat<I, F>
{
    pub fn new(
        metadata: MascotGenericFormatMetadata<I, F>,
        data: Vec<MascotGenericFormatData<F>>,
    ) -> Result<Self, String> {
        // We need to check that, if the data provided is compatible with
        // the metadata provided. Specifically, if the minimum MSLEVEL
        // of the data is equal to one, then the PEPMASS must be equal to
        // the minimum mass value reported in the data associated to the
        // first level.
        let mgf = Self { metadata, data };

        if let Ok(first_mgf) = mgf.get_first_fragmentation_level() {
            if mgf.parent_ion_mass() != first_mgf.min_mass_divided_by_charge_ratio() {
                return Err(format!(
                    concat!(
                        "When the MGF contains data relative to fragmentation level one, ",
                        "it is necessary for the parent ion mass entry in the metadata (PEPMASS) ",
                        "to be equal to the minimum mass ratio reported in the data of the associated ",
                        "first fragmentation level. In this case, we encounted a metadata pepmass ",
                        "of {:?}, while the minimum mass-charge ratio was {:?}. This may be a data bug ",
                        "derived from how the file was created."
                    ),
                    mgf.parent_ion_mass(),
                    first_mgf.min_mass_divided_by_charge_ratio()
                ));
            }
        }

        Ok(mgf)
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

    /// Returns a reference to the first fragmentation level, if available.
    pub fn get_first_fragmentation_level(&self) -> Result<&MascotGenericFormatData<F>, String> {
        if let Some(mgf) = self
            .data
            .iter()
            .filter(|mgf| mgf.level() == FragmentationSpectraLevel::One)
            .next()
        {
            Ok(mgf)
        } else {
            Err(concat!(
                "There is no first fragmentation level available for the ",
                "corrent mascot fragmentation object."
            )
            .to_string())
        }
    }

    /// Returns a reference to the second fragmentation level, if available.
    pub fn get_second_fragmentation_level(&self) -> Result<&MascotGenericFormatData<F>, String> {
        if let Some(mgf) = self
            .data
            .iter()
            .filter(|mgf| mgf.level() == FragmentationSpectraLevel::Two)
            .next()
        {
            Ok(mgf)
        } else {
            Err(concat!(
                "There is no second fragmentation level available for the ",
                "corrent mascot fragmentation object."
            )
            .to_string())
        }
    }

    /// Returns the minimum fragmentation level.
    pub fn min_fragmentation_level(&self) -> FragmentationSpectraLevel {
        self.data.iter().map(|d| d.level()).min().unwrap()
    }

    /// Returns the maximum fragmentation level.
    pub fn max_fragmentation_level(&self) -> FragmentationSpectraLevel {
        self.data.iter().map(|d| d.level()).max().unwrap()
    }
}

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct MGFVec<I, F> {
    mascot_generic_formats: Vec<MascotGenericFormat<I, F>>,
}

impl<I, F> MGFVec<I, F> {
    pub fn new() -> Self {
        Self {
            mascot_generic_formats: Vec::new(),
        }
    }

    /// Create a new vector of MGF objects from the file at the provided path.
    ///
    /// # Arguments
    /// * `path` - The path to the file to read.
    ///
    /// # Returns
    /// A new vector of MGF objects.
    ///
    /// # Errors
    /// * If the file at the provided path cannot be read.
    /// * If the file at the provided path cannot be parsed.
    ///
    /// # Examples
    ///
    /// An example of a document that contains only the first level of
    /// fragmentation spectra:
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    ///
    /// let path = "tests/data/20220513_PMA_DBGI_01_04_003.mgf";
    ///
    /// let mascot_generic_formats: MGFVec<usize, f64> = MGFVec::from_path(path).unwrap();
    ///
    /// assert_eq!(mascot_generic_formats.len(), 74, concat!(
    ///     "The number of MascotGenericFormat objects in the vector should be 74, ",
    ///     "but it is {}."
    /// ), mascot_generic_formats.len());
    /// ```
    ///
    /// An example of another type of documents that contains both the first and
    /// second level of fragmentation spectra:
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    ///
    /// let path = "tests/data/20220513_PMA_DBGI_01_04_001.mzML_chromatograms_deconvoluted_deisotoped_filtered_enpkg_sirius.mgf";
    ///
    /// let mascot_generic_formats: MGFVec<usize, f64> = MGFVec::from_path(path).unwrap();
    ///
    /// assert_eq!(mascot_generic_formats.len(), 139);
    ///
    /// ```
    ///
    ///
    pub fn from_path(path: &str) -> Result<Self, String>
    where
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug + Zero + Hash,
        F: Copy + StrictlyPositive + FromStr + PartialEq + Debug + PartialOrd,
    {
        let file = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        Self::try_from_iter(file.lines().filter(|line| !line.is_empty()))
    }

    pub fn try_from_iter<'a, T>(iter: T) -> Result<Self, String>
    where
        T: IntoIterator<Item = &'a str>,
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug + Zero + Hash,
        F: Copy + StrictlyPositive + FromStr + PartialEq + Debug + PartialOrd,
    {
        let mut mascot_generic_formats = MGFVec::new();
        let mut mascot_generic_format_builder = MascotGenericFormatBuilder::default();

        for line in iter {
            mascot_generic_format_builder.digest_line(line)?;
            if mascot_generic_format_builder.can_build() {
                mascot_generic_formats.push(mascot_generic_format_builder.build()?);
                mascot_generic_format_builder = MascotGenericFormatBuilder::default();
            }
        }

        // We check that the feature id values are unique.
        let number_of_unique_feature_ids = mascot_generic_formats
            .iter()
            .map(|mgf| mgf.feature_id())
            .collect::<HashSet<I>>()
            .len();
        if number_of_unique_feature_ids != mascot_generic_formats.len() {
            return Err(format!(
                concat!(
                    "We have identified {} duplicated feature ids in the MGF document provided. ",
                    "Specifically, there were {} entries, but only {} unique feature IDs."
                ),
                mascot_generic_formats.len() - number_of_unique_feature_ids,
                mascot_generic_formats.len(),
                number_of_unique_feature_ids
            ));
        }

        Ok(mascot_generic_formats)
    }

    pub fn push(&mut self, mascot_generic_format: MascotGenericFormat<I, F>) {
        self.mascot_generic_formats.push(mascot_generic_format);
    }

    pub fn len(&self) -> usize {
        self.mascot_generic_formats.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mascot_generic_formats.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &MascotGenericFormat<I, F>> {
        self.mascot_generic_formats.iter()
    }

    pub fn as_slice(&self) -> &[MascotGenericFormat<I, F>] {
        self.mascot_generic_formats.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [MascotGenericFormat<I, F>] {
        self.mascot_generic_formats.as_mut_slice()
    }

    pub fn into_vec(self) -> Vec<MascotGenericFormat<I, F>> {
        self.mascot_generic_formats
    }

    pub fn clear(&mut self) {
        self.mascot_generic_formats.clear();
    }
}

impl<I, F> Default for MGFVec<I, F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I, F> Index<usize> for MGFVec<I, F> {
    type Output = MascotGenericFormat<I, F>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.mascot_generic_formats[index]
    }
}

impl<I, F> IndexMut<usize> for MGFVec<I, F> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.mascot_generic_formats[index]
    }
}
