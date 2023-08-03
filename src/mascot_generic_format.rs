use crate::prelude::*;
use std::fmt::Debug;
use std::ops::{Add, Index, IndexMut};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct MascotGenericFormat<I, F> {
    metadata: MascotGenericFormatMetadata<I, F>,
    data: Vec<MascotGenericFormatData<F>>,
}

impl<I: Copy + Zero + PartialEq + Debug + Add<Output = I> + Eq, F: Copy + StrictlyPositive>
    MascotGenericFormat<I, F>
{
    pub fn new(
        metadata: MascotGenericFormatMetadata<I, F>,
        data: Vec<MascotGenericFormatData<F>>,
    ) -> Self {
        Self { metadata, data }
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

    /// Returns the minimum fragmentation level.
    pub fn min_fragmentation_level(&self) -> Option<FragmentationSpectraLevel> {
        self.data.iter().map(|d| d.level()).min()
    }

    /// Returns the maximum fragmentation level.
    pub fn max_fragmentation_level(&self) -> Option<FragmentationSpectraLevel> {
        self.data.iter().map(|d| d.level()).max()
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
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug + Zero,
        F: Copy + StrictlyPositive + FromStr + PartialEq + Debug,
    {
        let file = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        Ok(file
            .lines()
            .filter(|line| !line.is_empty())
            .collect::<MGFVec<I, F>>())
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

impl<I, F> From<Vec<MascotGenericFormat<I, F>>> for MGFVec<I, F> {
    fn from(mascot_generic_formats: Vec<MascotGenericFormat<I, F>>) -> Self {
        Self {
            mascot_generic_formats,
        }
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

impl<I, F> IntoIterator for MGFVec<I, F> {
    type Item = MascotGenericFormat<I, F>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.mascot_generic_formats.into_iter()
    }
}

impl<
        'a,
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug + Zero,
        F: Copy + StrictlyPositive + FromStr + PartialEq + Debug,
    > FromIterator<&'a str> for MGFVec<I, F>
{
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let mut mascot_generic_formats = MGFVec::new();
        let mut mascot_generic_format_builder = MascotGenericFormatBuilder::default();

        for line in iter {
            mascot_generic_format_builder.digest_line(line).unwrap();
            if mascot_generic_format_builder.can_build() {
                mascot_generic_formats.push(mascot_generic_format_builder.build().unwrap());
                mascot_generic_format_builder = MascotGenericFormatBuilder::default();
            }
        }

        mascot_generic_formats
    }
}
