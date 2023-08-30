use crate::prelude::*;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::io::Write;
use std::ops::{Add, Index, IndexMut, Sub};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct MascotGenericFormat<I, F> {
    metadata: MascotGenericFormatMetadata<I, F>,
    data: Vec<MascotGenericFormatData<F>>,
}

impl<
        I: Copy + Zero + PartialEq + Debug + Add<Output = I> + Eq,
        F: Copy
            + StrictlyPositive
            + PartialEq
            + PartialOrd
            + Debug
            + Display
            + Add<F, Output = F>
            + Sub<F, Output = F>,
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
        let mut mgf = Self { metadata, data };

        // If the provided metadata does not have a parent ion mass, we
        // can set the value to the minimum mass divided by charge ratio
        // from the first fragmentation level, if available.
        if mgf.parent_ion_mass().is_none() && mgf.has_first_level() {
            mgf.metadata.set_parent_ion_mass(
                mgf.get_first_fragmentation_level()?
                    .min_mass_divided_by_charge_ratio(),
            );
        }

        // If the provided metadata provides fragmentation data of the first level,
        // we need to check that the parent ion mass is equal to one of the masses
        // reported in the first level.

        if let Ok(first_mgf) = mgf.get_first_fragmentation_level() {
            if !first_mgf.has_mass_divided_by_charge_ratio(mgf.parent_ion_mass().unwrap()) {
                return Err(format!(
                    concat!(
                        "The parent ion mass {} is not equal to any of the ",
                        "masses reported in the first level of fragmentation spectra."
                    ),
                    mgf.parent_ion_mass().unwrap()
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
    pub fn parent_ion_mass(&self) -> Option<F> {
        self.metadata.parent_ion_mass()
    }

    /// Returns the retention time of the metadata.
    pub fn retention_time(&self) -> Option<F> {
        self.metadata.retention_time()
    }

    /// Returns the charge of the metadata.
    pub fn charge(&self) -> Charge {
        self.metadata.charge()
    }

    /// Returns a reference to the first fragmentation level, if available.
    pub fn get_first_fragmentation_level(&self) -> Result<&MascotGenericFormatData<F>, String> {
        if let Some(mgf) = self
            .data
            .iter()
            .find(|mgf| mgf.level() == FragmentationSpectraLevel::One)
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
            .find(|mgf| mgf.level() == FragmentationSpectraLevel::Two)
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

    /// Returns iterator over the mass over charge ratios of the first fragmentation level.
    pub fn first_fragmentation_level_mass_divided_by_charge_ratios_iter(
        &self,
    ) -> Result<std::slice::Iter<F>, String> {
        Ok(self
            .get_first_fragmentation_level()?
            .mass_divided_by_charge_ratios_iter())
    }

    /// Returns iterator over the mass over charge ratios of the second fragmentation level.
    pub fn second_fragmentation_level_mass_divided_by_charge_ratios_iter(
        &self,
    ) -> Result<std::slice::Iter<F>, String> {
        Ok(self
            .get_second_fragmentation_level()?
            .mass_divided_by_charge_ratios_iter())
    }

    /// Returns iterator over the intensities of the first fragmentation level.
    pub fn first_fragmentation_level_intensities_iter(
        &self,
    ) -> Result<std::slice::Iter<F>, String> {
        Ok(self
            .get_first_fragmentation_level()?
            .fragment_intensities_iter())
    }

    /// Returns iterator over the intensities of the second fragmentation level.
    pub fn second_fragmentation_level_intensities_iter(
        &self,
    ) -> Result<std::slice::Iter<F>, String> {
        Ok(self
            .get_second_fragmentation_level()?
            .fragment_intensities_iter())
    }

    /// Returns the minimum fragmentation level.
    pub fn min_fragmentation_level(&self) -> FragmentationSpectraLevel {
        self.data.iter().map(|d| d.level()).min().unwrap()
    }

    /// Returns the maximum fragmentation level.
    pub fn max_fragmentation_level(&self) -> FragmentationSpectraLevel {
        self.data.iter().map(|d| d.level()).max().unwrap()
    }

    /// Returns whether the current MGF has first level fragmentation data.
    pub fn has_first_level(&self) -> bool {
        self.min_fragmentation_level() == FragmentationSpectraLevel::One
    }

    /// Returns whether the current MGF has second level fragmentation data.
    pub fn has_second_level(&self) -> bool {
        self.max_fragmentation_level() == FragmentationSpectraLevel::Two
    }

    /// Returns indices associated to matching mass-charge ratios of the second level.
    ///
    /// # Arguments
    /// * `other` - The other [`MascotGenericFormat`] object.
    /// * `tolerance` - The tolerance to use when matching mass-charge ratios.
    /// * `shift` - The shift to apply to the mass-charge ratios of the other
    ///
    /// # Safety
    /// This function is unsafe because it does not check that the
    /// mass-charge ratios are sorted in ascending order. The results
    /// when the requirement is not met are undefined. Also, it does not
    /// check whether the MGF files have a second level.
    pub fn find_sorted_matches(
        &self,
        other: &MascotGenericFormat<I, F>,
        tolerance: F,
        shift: F,
    ) -> Result<Vec<(usize, usize)>, String> {
        let mut matches = Vec::new();
        let mut lowest_index = 0;

        for (i, first_mz) in self
            .second_fragmentation_level_mass_divided_by_charge_ratios_iter()?
            .copied()
            .enumerate()
        {
            let low_bound = first_mz - tolerance;
            let high_bound = first_mz + tolerance;

            for (j, shifted_second_mz) in other
                .second_fragmentation_level_mass_divided_by_charge_ratios_iter()?
                .skip(lowest_index)
                .copied()
                .map(|second_mz| second_mz + shift)
                .enumerate()
            {
                if shifted_second_mz > high_bound {
                    break;
                }
                if shifted_second_mz < low_bound {
                    lowest_index = j;
                    continue;
                }
                matches.push((i, j));
            }
        }

        Ok(matches)
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
    /// let mascot_generic_formats: MGFVec<usize, f64> = MGFVec::try_from_path(path).unwrap();
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
    /// let mascot_generic_formats: MGFVec<usize, f64> = MGFVec::try_from_path(path).unwrap();
    ///
    /// assert_eq!(mascot_generic_formats.len(), 139);
    ///
    /// ```
    ///
    ///
    pub fn try_from_path(path: &str) -> Result<Self, String>
    where
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug + Zero + Hash,
        F: Copy
            + StrictlyPositive
            + FromStr
            + PartialEq
            + Debug
            + PartialOrd
            + NaN
            + Zero
            + Display
            + Sub<F, Output = F>
            + Add<F, Output = F>,
    {
        let file = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

        // If the try from iter fails, we return the error that needs to be extended
        // to als include the path to the file.

        match Self::try_from_iter(file.lines().filter(|line| !line.is_empty())) {
            Ok(mascot_generic_formats) => Ok(mascot_generic_formats),
            Err(e) => Err(format!(
                concat!(
                    "The file at the provided path {} could not be parsed. ",
                    "The error message was: {}"
                ),
                path, e
            )),
        }
    }

    /// Create a new vector of valid MGF objects from the file at the provided path, writing
    /// the error log to the provided path.
    ///
    /// # Arguments
    /// * `path`: &str - The path to the file to read.
    /// * `error_log_path`: Option<&str> - The path to the file to write the error log to.
    ///
    /// # Returns
    /// A new vector of vaklid MGF objects.
    ///
    /// # Errors
    /// * If the file at the provided path cannot be read.
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
    /// let error_log_path = "tests/data/20220513_PMA_DBGI_01_04_003.mgf.error.log";
    ///
    /// let mascot_generic_formats: MGFVec<usize, f64> = MGFVec::valid_from_path_with_error_log(path, Some(error_log_path)).unwrap();
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
    /// let error_log_path = "tests/data/20220513_PMA_DBGI_01_04_001.mzML_chromatograms_deconvoluted_deisotoped_filtered_enpkg_sirius.mgf.error.log";
    ///
    /// let mascot_generic_formats: MGFVec<usize, f64> = MGFVec::valid_from_path_with_error_log(path, Some(error_log_path)).unwrap();
    ///
    /// assert_eq!(mascot_generic_formats.len(), 139);
    ///
    /// ```
    ///
    ///
    pub fn valid_from_path_with_error_log(
        path: &str,
        error_log_path: Option<&str>,
    ) -> Result<Self, String>
    where
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug + Zero + Hash + Display,
        F: Copy
            + StrictlyPositive
            + FromStr
            + PartialEq
            + Debug
            + PartialOrd
            + NaN
            + Zero
            + Display
            + Sub<F, Output = F>
            + Add<F, Output = F>,
    {
        let file = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

        // If the try from iter fails, we return the error that needs to be extended
        // to als include the path to the file.

        Ok(Self::from_iter_with_error_log(
            file.lines().filter(|line| !line.is_empty()),
            error_log_path,
        ))
    }

    /// Create a new vector of valid MGF objects from the file at the provided path.
    ///
    /// # Arguments
    /// * `path` - The path to the file to read.
    ///
    /// # Returns
    /// A new vector of vaklid MGF objects.
    ///
    /// # Errors
    /// * If the file at the provided path cannot be read.
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
    /// let mascot_generic_formats: MGFVec<usize, f64> = MGFVec::valid_from_path(path).unwrap();
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
    /// let mascot_generic_formats: MGFVec<usize, f64> = MGFVec::valid_from_path(path).unwrap();
    ///
    /// assert_eq!(mascot_generic_formats.len(), 139);
    ///
    /// ```
    ///
    ///
    pub fn valid_from_path(path: &str) -> Result<Self, String>
    where
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug + Zero + Hash + Display,
        F: Copy
            + StrictlyPositive
            + FromStr
            + PartialEq
            + Debug
            + PartialOrd
            + NaN
            + Zero
            + Display
            + Sub<F, Output = F>
            + Add<F, Output = F>,
    {
        Self::valid_from_path_with_error_log(path, None)
    }

    pub fn try_from_iter<'a, T>(iter: T) -> Result<Self, String>
    where
        T: IntoIterator<Item = &'a str>,
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug + Zero + Hash,
        F: Copy
            + StrictlyPositive
            + FromStr
            + PartialEq
            + Debug
            + PartialOrd
            + NaN
            + Zero
            + Display
            + Sub<F, Output = F>
            + Add<F, Output = F>,
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

        Ok(mascot_generic_formats)
    }

    /// Create a new vector of valid MGF objects from the file at the provided path, writing
    /// the error log to the provided path.
    ///
    /// # Arguments
    /// * `iter` - The iterator over the lines of the file to read.
    /// * `error_log_path` - The path to the file to write the error log to.
    ///
    /// # Returns
    /// A new vector of MGF objects, filtering out invalid ones.
    pub fn from_iter_with_error_log<'a, T>(iter: T, error_log_path: Option<&str>) -> Self
    where
        T: IntoIterator<Item = &'a str>,
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug + Zero + Hash + Display,
        F: Copy
            + StrictlyPositive
            + FromStr
            + PartialEq
            + Debug
            + PartialOrd
            + NaN
            + Zero
            + Display
            + Sub<F, Output = F>
            + Add<F, Output = F>,
    {
        let mut mascot_generic_formats = MGFVec::new();
        let mut mascot_generic_format_builder = MascotGenericFormatBuilder::default();
        // We create a backup of the builder to use when we find ourselves in the situation that
        // a corrupted MGF entry partially overlaps with a valid one. In this case, we want to
        // keep the valid one and discard the corrupted one, so we use the backup to delete the
        // corrupted one while keeping the valid one.
        let mut mascot_backup: Option<MascotGenericFormatBuilder<I, F>> = None;
        let mut error_log_file = error_log_path.map(|path| std::fs::File::create(path).unwrap());

        for line in iter {
            // If the current MGF is corrupted, it means we could not recover from the corruption
            // using the backup builder. We need to skip lines up until we find the next MGF entry.
            if mascot_generic_format_builder.is_corrupted() {
                if MascotGenericFormatBuilder::<I, F>::is_start_of_new_entry(line) {
                    mascot_generic_format_builder = MascotGenericFormatBuilder::default();
                    mascot_backup = None;
                } else {
                    continue;
                }
            }

            // If the current MGF has reached the partial state, we create a new backup
            // so that as the MGF starts to digest the next line, we can safely store it also
            // in the backup and handle the corner cases where the MGF entry is corrupted.
            if mascot_generic_format_builder.is_partial() {
                mascot_backup = Some(MascotGenericFormatBuilder::default());
            }

            // We try to process the current line with the backup builder, if it exists.
            if let Some(mascot_backup) = mascot_backup.as_mut() {
                let _ = mascot_backup.digest_line(line);
            };

            if let Err(e) = mascot_generic_format_builder.digest_line(line) {
                if let Some(error_log_file) = error_log_file.as_mut() {
                    writeln!(error_log_file, "{}", e).unwrap();
                }
            }

            // If the current MGF builder has been corrupted and there is a backup builder
            // that is not corrupted, we replace the current builder with the backup builder.
            if mascot_generic_format_builder.is_corrupted()
                && mascot_backup
                    .as_ref()
                    .map_or(false, |backup| !backup.is_corrupted())
            {
                if let Some(error_log_file) = error_log_file.as_mut() {
                    writeln!(
                        error_log_file,
                        concat!(
                            "Corruption detected in the MGF entry with feature ID '{}'. ",
                            "Attempting to recover from the corruption by using the backup builder with ",
                            "feature ID '{}'.",
                        ),
                        mascot_generic_format_builder.feature_id().map_or_else(|| "Unknown".to_string(), |id: I| id.to_string()),
                        mascot_backup.as_ref().unwrap().feature_id().map_or_else(|| "Unknown".to_string(), |id: I| id.to_string()),
                    ).unwrap();
                }
                mascot_generic_format_builder = mascot_backup.take().unwrap();
            }

            // If both the current MGF builder and the backup builder are corrupted, we
            // need to skip lines up until we find the next MGF entry. We log an entry to
            // notify the user that we are skipping lines and that attempts to recover
            // from the corruption have failed.

            if mascot_generic_format_builder.is_corrupted()
                && mascot_backup
                    .as_ref()
                    .map_or(false, |backup| backup.is_corrupted())
            {
                if let Some(error_log_file) = error_log_file.as_mut() {
                    writeln!(
                        error_log_file,
                        concat!(
                            "Corruption detected in the MGF entry with feature ID '{}'. ",
                            "Attempts to recover from the corruption by using the backup builder with ",
                            "feature ID '{}' have failed. Skipping lines until the next MGF entry is found.",
                        ),
                        mascot_generic_format_builder.feature_id().map_or_else(|| "Unknown".to_string(), |id: I| id.to_string()),
                        mascot_backup.as_ref().unwrap().feature_id().map_or_else(|| "Unknown".to_string(), |id: I| id.to_string()),
                    ).unwrap();
                }
            }

            if mascot_generic_format_builder.can_build() {
                match mascot_generic_format_builder.build() {
                    Ok(mascot_generic_format) => {
                        mascot_generic_formats.push(mascot_generic_format);
                    }
                    Err(e) => {
                        if let Some(error_log_file) = error_log_file.as_mut() {
                            writeln!(error_log_file, "{}", e).unwrap();
                        }
                    }
                }
                mascot_backup = None;
                mascot_generic_format_builder = MascotGenericFormatBuilder::default();
            }
        }

        mascot_generic_formats
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
