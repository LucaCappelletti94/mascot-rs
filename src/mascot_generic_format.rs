use std::fmt::Debug;
use std::ops::{Add, Index};
use std::str::FromStr;

use mass_spectrometry::prelude::{GenericSpectrum, Spectra, Spectrum, SpectrumMut};

use crate::error::{MascotError, Result};
use crate::mascot_generic_format_builder::MascotGenericFormatBuilder;
use crate::mascot_generic_format_metadata::MascotGenericFormatMetadata;

/// A single Mascot Generic Format ion block with metadata and spectra.
///
/// When used as a [`Spectrum`], the record exposes the peaks from this MGF ion
/// block.
#[derive(Debug)]
pub struct MascotGenericFormat<I> {
    metadata: MascotGenericFormatMetadata<I>,
    spectrum: GenericSpectrum,
}

impl<I: Copy> MascotGenericFormat<I> {
    /// Creates a new [`MascotGenericFormat`].
    ///
    /// # Errors
    /// Returns an error if no peak data is provided, if the peak values are
    /// invalid, or if first-level data is incompatible with the metadata parent
    /// ion mass.
    pub fn new(
        metadata: MascotGenericFormatMetadata<I>,
        mass_divided_by_charge_ratios: Vec<f64>,
        fragment_intensities: Vec<f64>,
    ) -> Result<Self> {
        if mass_divided_by_charge_ratios.len() != fragment_intensities.len() {
            return Err(MascotError::PeakVectorLengthMismatch {
                mz_len: mass_divided_by_charge_ratios.len(),
                intensity_len: fragment_intensities.len(),
            });
        }

        if mass_divided_by_charge_ratios.is_empty() {
            return Err(MascotError::EmptyPeakVectors);
        }

        let peak_capacity = mass_divided_by_charge_ratios.len();
        let mut spectrum =
            GenericSpectrum::try_with_capacity(metadata.parent_ion_mass(), peak_capacity)?;

        let mut peaks = mass_divided_by_charge_ratios
            .into_iter()
            .zip(fragment_intensities)
            .collect::<Vec<_>>();
        peaks.sort_by(|(left_mz, _), (right_mz, _)| left_mz.total_cmp(right_mz));

        let mut merged_peaks: Vec<(f64, f64)> = Vec::with_capacity(peaks.len());
        for (mz, intensity) in peaks {
            if let Some((last_mz, last_intensity)) = merged_peaks.last_mut() {
                if last_mz.to_bits() == mz.to_bits() {
                    *last_intensity += intensity;
                    continue;
                }
            }
            merged_peaks.push((mz, intensity));
        }

        for (mz, intensity) in merged_peaks {
            spectrum.add_peak(mz, intensity)?;
        }

        if metadata.level() == 1
            && metadata.parent_ion_mass().to_bits() != spectrum.mz_nth(0).to_bits()
        {
            return Err(MascotError::FirstLevelParentIonMassMismatch {
                parent_ion_mass: metadata.parent_ion_mass(),
                first_level_min_mz: spectrum.mz_nth(0),
            });
        }

        Ok(Self { metadata, spectrum })
    }

    /// Returns the feature ID of the metadata.
    pub const fn feature_id(&self) -> I {
        self.metadata.feature_id()
    }

    /// Returns the MS fragmentation level.
    pub const fn level(&self) -> u8 {
        self.metadata.level()
    }

    /// Returns the charge of the metadata.
    pub const fn charge(&self) -> i8 {
        self.metadata.charge()
    }

    /// Returns the metadata for this MGF record.
    #[must_use]
    pub const fn metadata(&self) -> &MascotGenericFormatMetadata<I> {
        &self.metadata
    }
}

impl<I> AsRef<GenericSpectrum> for MascotGenericFormat<I> {
    fn as_ref(&self) -> &GenericSpectrum {
        &self.spectrum
    }
}

impl<I> From<MascotGenericFormat<I>> for GenericSpectrum {
    fn from(value: MascotGenericFormat<I>) -> Self {
        value.spectrum
    }
}

impl<I: Copy> Spectrum for MascotGenericFormat<I> {
    type SortedIntensitiesIter<'a>
        = <GenericSpectrum as Spectrum>::SortedIntensitiesIter<'a>
    where
        Self: 'a;
    type SortedMzIter<'a>
        = <GenericSpectrum as Spectrum>::SortedMzIter<'a>
    where
        Self: 'a;
    type SortedPeaksIter<'a>
        = <GenericSpectrum as Spectrum>::SortedPeaksIter<'a>
    where
        Self: 'a;

    fn len(&self) -> usize {
        self.spectrum.len()
    }

    fn intensities(&self) -> Self::SortedIntensitiesIter<'_> {
        self.spectrum.intensities()
    }

    fn intensity_nth(&self, n: usize) -> f64 {
        self.spectrum.intensity_nth(n)
    }

    fn mz(&self) -> Self::SortedMzIter<'_> {
        self.spectrum.mz()
    }

    fn mz_from(&self, index: usize) -> Self::SortedMzIter<'_> {
        self.spectrum.mz_from(index)
    }

    fn mz_nth(&self, n: usize) -> f64 {
        self.spectrum.mz_nth(n)
    }

    fn peaks(&self) -> Self::SortedPeaksIter<'_> {
        self.spectrum.peaks()
    }

    fn peak_nth(&self, n: usize) -> (f64, f64) {
        self.spectrum.peak_nth(n)
    }

    fn precursor_mz(&self) -> f64 {
        self.metadata.parent_ion_mass()
    }
}

#[repr(transparent)]
/// A collection of parsed [`MascotGenericFormat`] records.
#[derive(Debug)]
pub struct MGFVec<I> {
    mascot_generic_formats: Vec<MascotGenericFormat<I>>,
}

impl<I> MGFVec<I> {
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
    /// An example of a document that contains one fragmentation spectrum per
    /// ion block:
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    ///
    /// let path = "tests/data/20220513_PMA_DBGI_01_04_003.mgf";
    ///
    /// let mascot_generic_formats: MGFVec<usize> = MGFVec::from_path(path).unwrap();
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
    /// let mascot_generic_formats: MGFVec<usize> = MGFVec::from_path(path).unwrap();
    ///
    /// assert_eq!(mascot_generic_formats.len(), 278);
    ///
    /// ```
    ///
    ///
    pub fn from_path(path: &str) -> Result<Self>
    where
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug,
    {
        let file = std::fs::read_to_string(path).map_err(|source| MascotError::Io {
            path: path.to_string(),
            source,
        })?;
        Self::try_from_iter(file.lines().filter(|line| !line.is_empty()))
    }

    /// Creates a new vector of MGF objects from an iterator over document lines.
    ///
    /// # Errors
    /// Returns an error if any input line cannot be parsed, if any MGF section
    /// cannot be built.
    pub fn try_from_iter<'a, T>(iter: T) -> Result<Self>
    where
        T: IntoIterator<Item = &'a str>,
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug,
    {
        let mut mascot_generic_formats = Self {
            mascot_generic_formats: Vec::new(),
        };
        let mut mascot_generic_format_builder = MascotGenericFormatBuilder::default();

        for line in iter {
            mascot_generic_format_builder.digest_line(line)?;
            if mascot_generic_format_builder.can_build() {
                mascot_generic_formats
                    .mascot_generic_formats
                    .push(mascot_generic_format_builder.build()?);
                mascot_generic_format_builder = MascotGenericFormatBuilder::default();
            }
        }

        Ok(mascot_generic_formats)
    }

    /// Returns the number of MGF records in the collection.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.mascot_generic_formats.len()
    }

    /// Returns `true` if the collection contains no MGF records.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.mascot_generic_formats.is_empty()
    }
}

impl<I> Default for MGFVec<I> {
    fn default() -> Self {
        Self {
            mascot_generic_formats: Vec::new(),
        }
    }
}

impl<I> AsRef<[MascotGenericFormat<I>]> for MGFVec<I> {
    fn as_ref(&self) -> &[MascotGenericFormat<I>] {
        self.mascot_generic_formats.as_slice()
    }
}

impl<I: Copy> Spectra for MGFVec<I> {
    type Spectrum = MascotGenericFormat<I>;
    type SpectraIter<'a>
        = std::slice::Iter<'a, MascotGenericFormat<I>>
    where
        Self: 'a;

    fn spectra(&self) -> Self::SpectraIter<'_> {
        self.mascot_generic_formats.iter()
    }

    fn len(&self) -> usize {
        self.mascot_generic_formats.len()
    }
}

impl<I> Index<usize> for MGFVec<I> {
    type Output = MascotGenericFormat<I>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.mascot_generic_formats[index]
    }
}
