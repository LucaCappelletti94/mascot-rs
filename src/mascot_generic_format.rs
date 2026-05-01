#[cfg(feature = "std")]
use alloc::string::String;
use alloc::{boxed::Box, string::ToString, vec::Vec};
#[cfg(feature = "std")]
use core::fmt::Display;
use core::{
    fmt::Debug,
    ops::{Add, Index},
    str::FromStr,
};
#[cfg(feature = "std")]
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

use mass_spectrometry::prelude::{GenericSpectrum, Spectra, Spectrum, SpectrumFloat, SpectrumMut};

use crate::error::{MascotError, Result};
#[cfg(feature = "std")]
use crate::gnps::GNPSBuilder;
use crate::mascot_generic_format_builder::MascotGenericFormatBuilder;
use crate::mascot_generic_format_metadata::{Instrument, IonMode, MascotGenericFormatMetadata};

/// A single Mascot Generic Format ion block with metadata and spectra.
///
/// When used as a [`Spectrum`], the record exposes the peaks from this MGF ion
/// block.
#[derive(Debug)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
pub struct MascotGenericFormat<I, P: SpectrumFloat = f64> {
    metadata: MascotGenericFormatMetadata<I>,
    spectrum: GenericSpectrum<P>,
}

impl<I: Copy, P: SpectrumFloat> MascotGenericFormat<I, P> {
    /// Creates a new [`MascotGenericFormat`].
    ///
    /// # Errors
    /// Returns an error if no peak data is provided, if the peak values are
    /// invalid, or if first-level data is incompatible with the precursor m/z.
    pub fn new(
        metadata: MascotGenericFormatMetadata<I>,
        precursor_mz: f64,
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
        let mut spectrum = GenericSpectrum::<P>::try_with_capacity(precursor_mz, peak_capacity)?;

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

        let stored_precursor_mz = spectrum.precursor_mz().to_f64();
        let first_level_min_mz = spectrum.mz_nth(0).to_f64();
        if metadata.level() == 1 && stored_precursor_mz.to_bits() != first_level_min_mz.to_bits() {
            return Err(MascotError::FirstLevelPrecursorMzMismatch {
                precursor_mz: stored_precursor_mz,
                first_level_min_mz,
            });
        }

        Ok(Self { metadata, spectrum })
    }

    /// Returns the feature ID of the metadata, if present.
    pub const fn feature_id(&self) -> Option<I> {
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

    /// Returns the ionization polarity of the metadata, if present.
    pub const fn ion_mode(&self) -> Option<IonMode> {
        self.metadata.ion_mode()
    }

    /// Returns the normalized instrument metadata parsed from `SOURCE_INSTRUMENT`, if present.
    pub const fn source_instrument(&self) -> Option<Instrument> {
        self.metadata.source_instrument()
    }

    /// Returns the metadata for this MGF record.
    #[must_use]
    pub const fn metadata(&self) -> &MascotGenericFormatMetadata<I> {
        &self.metadata
    }

    #[cfg(feature = "std")]
    fn map_output_io(result: std::io::Result<()>) -> Result<()> {
        result.map_err(|source| MascotError::OutputIo { source })
    }
}

#[cfg(feature = "std")]
impl<I, P> MascotGenericFormat<I, P>
where
    I: Copy + Display,
    P: SpectrumFloat,
{
    /// Writes this record in canonical MGF syntax to a writer.
    ///
    /// # Errors
    /// Returns an error if the writer cannot be written.
    pub fn write_to<W>(&self, mut writer: W) -> Result<()>
    where
        W: Write,
    {
        self.write_record_to(&mut writer)
    }

    /// Writes this record to a path.
    ///
    /// Files ending in `.zst`, `.zstd`, `.gz`, or `.gzip` are compressed while
    /// they are written.
    ///
    /// # Errors
    /// Returns an error if the file cannot be created, compressed, or written.
    pub fn to_path<PathLike>(&self, path: PathLike) -> Result<()>
    where
        PathLike: AsRef<Path>,
    {
        MGFVec::<I, P>::write_to_path(path, |writer| self.write_record_to(writer))
    }

    fn write_record_to<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        Self::map_output_io(writeln!(writer, "BEGIN IONS"))?;
        if let Some(feature_id) = self.metadata.feature_id() {
            Self::map_output_io(writeln!(writer, "FEATURE_ID={feature_id}"))?;
        }
        Self::map_output_io(writeln!(
            writer,
            "PEPMASS={}",
            self.spectrum.precursor_mz().to_f64()
        ))?;
        Self::map_output_io(writeln!(writer, "CHARGE={}", self.metadata.charge()))?;
        if let Some(retention_time) = self.metadata.retention_time() {
            Self::map_output_io(writeln!(writer, "RTINSECONDS={retention_time}"))?;
        }
        Self::map_output_io(writeln!(writer, "MSLEVEL={}", self.metadata.level()))?;
        if let Some(filename) = self.metadata.filename() {
            Self::map_output_io(writeln!(writer, "FILENAME={filename}"))?;
        }
        if let Some(smiles) = self.metadata.smiles() {
            Self::map_output_io(writeln!(writer, "SMILES={smiles}"))?;
        }
        if let Some(ion_mode) = self.metadata.ion_mode() {
            Self::map_output_io(writeln!(writer, "IONMODE={ion_mode}"))?;
        }
        if let Some(source_instrument) = self.metadata.source_instrument() {
            Self::map_output_io(writeln!(writer, "SOURCE_INSTRUMENT={source_instrument}"))?;
        }
        for (mass_divided_by_charge_ratio, fragment_intensity) in self.spectrum.peaks() {
            Self::map_output_io(writeln!(
                writer,
                "{} {}",
                mass_divided_by_charge_ratio.to_f64(),
                fragment_intensity.to_f64()
            ))?;
        }
        match self.metadata.feature_id() {
            Some(feature_id) => Self::map_output_io(writeln!(writer, "SCANS={feature_id}"))?,
            None => Self::map_output_io(writeln!(writer, "SCANS=-1"))?,
        }
        Self::map_output_io(writeln!(writer, "END IONS"))?;

        Ok(())
    }
}

impl<I, P: SpectrumFloat> AsRef<GenericSpectrum<P>> for MascotGenericFormat<I, P> {
    fn as_ref(&self) -> &GenericSpectrum<P> {
        &self.spectrum
    }
}

impl<I, P: SpectrumFloat> From<MascotGenericFormat<I, P>> for GenericSpectrum<P> {
    fn from(value: MascotGenericFormat<I, P>) -> Self {
        value.spectrum
    }
}

impl<I, P> FromStr for MascotGenericFormat<I, P>
where
    I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug,
    P: SpectrumFloat,
{
    type Err = MascotError;

    fn from_str(s: &str) -> Result<Self> {
        let records = MGFVec::<I, P>::from_str(s)?;
        let found = records.mascot_generic_formats.len();
        if found != 1 {
            return Err(MascotError::SingleRecordExpected { found });
        }

        records
            .mascot_generic_formats
            .into_iter()
            .next()
            .ok_or(MascotError::SingleRecordExpected { found })
    }
}

impl<I: Copy, P: SpectrumFloat> Spectrum for MascotGenericFormat<I, P> {
    type Precision = P;
    type SortedIntensitiesIter<'a>
        = <GenericSpectrum<P> as Spectrum>::SortedIntensitiesIter<'a>
    where
        Self: 'a;
    type SortedMzIter<'a>
        = <GenericSpectrum<P> as Spectrum>::SortedMzIter<'a>
    where
        Self: 'a;
    type SortedPeaksIter<'a>
        = <GenericSpectrum<P> as Spectrum>::SortedPeaksIter<'a>
    where
        Self: 'a;

    fn len(&self) -> usize {
        self.spectrum.len()
    }

    fn intensities(&self) -> Self::SortedIntensitiesIter<'_> {
        self.spectrum.intensities()
    }

    fn intensity_nth(&self, n: usize) -> Self::Precision {
        self.spectrum.intensity_nth(n)
    }

    fn mz(&self) -> Self::SortedMzIter<'_> {
        self.spectrum.mz()
    }

    fn mz_from(&self, index: usize) -> Self::SortedMzIter<'_> {
        self.spectrum.mz_from(index)
    }

    fn mz_nth(&self, n: usize) -> Self::Precision {
        self.spectrum.mz_nth(n)
    }

    fn peaks(&self) -> Self::SortedPeaksIter<'_> {
        self.spectrum.peaks()
    }

    fn peak_nth(&self, n: usize) -> (Self::Precision, Self::Precision) {
        self.spectrum.peak_nth(n)
    }

    fn precursor_mz(&self) -> Self::Precision {
        self.spectrum.precursor_mz()
    }
}

#[repr(transparent)]
/// A collection of parsed [`MascotGenericFormat`] records.
#[derive(Debug)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
pub struct MGFVec<I, P: SpectrumFloat = f64> {
    mascot_generic_formats: Vec<MascotGenericFormat<I, P>>,
}

impl<I, P: SpectrumFloat> MGFVec<I, P> {
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
    /// Files ending in `.zst`, `.zstd`, `.gz`, or `.gzip` are decompressed
    /// while they are read.
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
    #[cfg(feature = "std")]
    pub fn from_path<PathLike>(path: PathLike) -> Result<Self>
    where
        PathLike: AsRef<Path>,
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug,
    {
        let path = path.as_ref();
        Self::from_reader(Self::reader_from_path(path)?)
    }

    #[cfg(feature = "std")]
    fn reader_from_path(path: &Path) -> Result<Box<dyn BufRead>> {
        let file = File::open(path).map_err(|source| MascotError::Io {
            path: path.display().to_string(),
            source,
        })?;

        if Self::has_extension(path, &["zst", "zstd"]) {
            let decoder =
                zstd::stream::read::Decoder::new(file).map_err(|source| MascotError::Io {
                    path: path.display().to_string(),
                    source,
                })?;
            return Ok(Box::new(BufReader::new(decoder)));
        }

        if Self::has_extension(path, &["gz", "gzip"]) {
            return Ok(Box::new(BufReader::new(flate2::read::MultiGzDecoder::new(
                file,
            ))));
        }

        Ok(Box::new(BufReader::new(file)))
    }

    #[cfg(feature = "std")]
    fn has_extension(path: &Path, expected_extensions: &[&str]) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                expected_extensions
                    .iter()
                    .any(|expected_extension| extension.eq_ignore_ascii_case(expected_extension))
            })
    }

    #[cfg(feature = "std")]
    fn write_to_path<PathLike, F>(path: PathLike, write: F) -> Result<()>
    where
        PathLike: AsRef<Path>,
        F: FnOnce(&mut dyn Write) -> Result<()>,
    {
        let path = path.as_ref();
        let file = File::create(path).map_err(|source| MascotError::Io {
            path: path.display().to_string(),
            source,
        })?;

        if Self::has_extension(path, &["zst", "zstd"]) {
            let writer = BufWriter::new(file);
            let mut encoder =
                zstd::stream::write::Encoder::new(writer, 0).map_err(|source| MascotError::Io {
                    path: path.display().to_string(),
                    source,
                })?;
            write(&mut encoder)?;
            encoder.finish().map_err(|source| MascotError::Io {
                path: path.display().to_string(),
                source,
            })?;
            return Ok(());
        }

        if Self::has_extension(path, &["gz", "gzip"]) {
            let writer = BufWriter::new(file);
            let mut encoder = flate2::write::GzEncoder::new(writer, flate2::Compression::default());
            write(&mut encoder)?;
            encoder.finish().map_err(|source| MascotError::Io {
                path: path.display().to_string(),
                source,
            })?;
            return Ok(());
        }

        let mut writer = BufWriter::new(file);
        write(&mut writer)?;
        writer.flush().map_err(|source| MascotError::Io {
            path: path.display().to_string(),
            source,
        })?;
        Ok(())
    }

    /// Creates a new vector of MGF objects from a buffered line reader.
    ///
    /// # Errors
    /// Returns an error if the reader cannot be read, if any input line cannot
    /// be parsed, or if any MGF section cannot be built.
    #[cfg(feature = "std")]
    pub fn from_reader<R>(mut reader: R) -> Result<Self>
    where
        R: BufRead,
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug,
    {
        let mut mascot_generic_formats = Self {
            mascot_generic_formats: Vec::new(),
        };
        let mut mascot_generic_format_builder = MascotGenericFormatBuilder::<I, P>::default();
        let mut line = String::new();
        let mut line_number = 0;

        while reader
            .read_line(&mut line)
            .map_err(|source| MascotError::InputIo { source })?
            != 0
        {
            line_number += 1;
            let trimmed_line = line.trim_end_matches(['\r', '\n']);
            if !trimmed_line.is_empty() {
                mascot_generic_format_builder
                    .digest_line(trimmed_line)
                    .map_err(|source| MascotError::InputLine {
                        line_number,
                        line: trimmed_line.to_string(),
                        source: Box::new(source),
                    })?;
                if mascot_generic_format_builder.can_build() {
                    mascot_generic_formats.mascot_generic_formats.push(
                        mascot_generic_format_builder.build().map_err(|source| {
                            MascotError::InputLine {
                                line_number,
                                line: trimmed_line.to_string(),
                                source: Box::new(source),
                            }
                        })?,
                    );
                    mascot_generic_format_builder = MascotGenericFormatBuilder::<I, P>::default();
                } else if mascot_generic_format_builder.can_skip_empty_section() {
                    return Err(MascotError::InputLine {
                        line_number,
                        line: trimmed_line.to_string(),
                        source: Box::new(MascotError::EmptyPeakVectors),
                    });
                }
            }
            line.clear();
        }

        Ok(mascot_generic_formats)
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
        let mut mascot_generic_format_builder = MascotGenericFormatBuilder::<I, P>::default();

        for line in iter {
            mascot_generic_format_builder.digest_line(line)?;
            if mascot_generic_format_builder.can_build() {
                mascot_generic_formats
                    .mascot_generic_formats
                    .push(mascot_generic_format_builder.build()?);
                mascot_generic_format_builder = MascotGenericFormatBuilder::<I, P>::default();
            } else if mascot_generic_format_builder.can_skip_empty_section() {
                return Err(MascotError::EmptyPeakVectors);
            }
        }

        Ok(mascot_generic_formats)
    }

    #[cfg(feature = "std")]
    pub(crate) fn from_reader_skipping_invalid_records<R>(mut reader: R) -> Result<(Self, usize)>
    where
        R: BufRead,
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug,
    {
        let mut mascot_generic_formats = Self::default();
        let mut skipped_records = 0;
        let mut block_lines: Vec<String> = Vec::new();
        let mut line = String::new();
        let mut section_open = false;

        while reader
            .read_line(&mut line)
            .map_err(|source| MascotError::InputIo { source })?
            != 0
        {
            let trimmed_line = line.trim_end_matches(['\r', '\n']);
            if trimmed_line.is_empty() {
                line.clear();
                continue;
            }

            if trimmed_line == "BEGIN IONS" {
                if section_open {
                    skipped_records += 1;
                    block_lines.clear();
                }
                section_open = true;
                block_lines.push(trimmed_line.to_string());
                line.clear();
                continue;
            }

            if section_open {
                block_lines.push(trimmed_line.to_string());
                if trimmed_line == "END IONS" {
                    match Self::try_from_iter(block_lines.iter().map(String::as_str)) {
                        Ok(parsed) if parsed.is_empty() => {
                            skipped_records += 1;
                        }
                        Ok(mut parsed) => {
                            mascot_generic_formats
                                .mascot_generic_formats
                                .append(&mut parsed.mascot_generic_formats);
                        }
                        Err(_) => {
                            skipped_records += 1;
                        }
                    }
                    block_lines.clear();
                    section_open = false;
                }
            }

            line.clear();
        }

        if section_open {
            skipped_records += 1;
        }

        Ok((mascot_generic_formats, skipped_records))
    }

    #[cfg(feature = "std")]
    pub(crate) fn from_path_skipping_invalid_records(path: &Path) -> Result<(Self, usize)>
    where
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug,
    {
        Self::from_reader_skipping_invalid_records(Self::reader_from_path(path)?)
    }

    /// Returns an iterator over the MGF records in the collection.
    pub fn iter(&self) -> core::slice::Iter<'_, MascotGenericFormat<I, P>> {
        self.mascot_generic_formats.iter()
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

#[cfg(feature = "std")]
impl<I, P> MGFVec<I, P>
where
    I: Copy + Display,
    P: SpectrumFloat,
{
    /// Writes all records in canonical MGF syntax to a writer.
    ///
    /// Records are separated by one blank line.
    ///
    /// # Errors
    /// Returns an error if the writer cannot be written.
    pub fn write_to<W>(&self, mut writer: W) -> Result<()>
    where
        W: Write,
    {
        for (index, mascot_generic_format) in self.mascot_generic_formats.iter().enumerate() {
            if index > 0 {
                MascotGenericFormat::<I, P>::map_output_io(writeln!(writer))?;
            }
            mascot_generic_format.write_record_to(&mut writer)?;
        }

        Ok(())
    }

    /// Writes all records to a path.
    ///
    /// Files ending in `.zst`, `.zstd`, `.gz`, or `.gzip` are compressed while
    /// they are written.
    ///
    /// # Errors
    /// Returns an error if the file cannot be created, compressed, or written.
    pub fn to_path<PathLike>(&self, path: PathLike) -> Result<()>
    where
        PathLike: AsRef<Path>,
    {
        Self::write_to_path(path, |writer| self.write_to(writer))
    }
}

#[cfg(feature = "std")]
impl<P: SpectrumFloat> MGFVec<usize, P> {
    /// Returns a builder for the GNPS public MGF spectral library.
    #[must_use]
    pub fn gnps() -> GNPSBuilder<P> {
        GNPSBuilder::default()
    }
}

impl<I, P: SpectrumFloat> Default for MGFVec<I, P> {
    fn default() -> Self {
        Self {
            mascot_generic_formats: Vec::new(),
        }
    }
}

impl<I, P: SpectrumFloat> FromIterator<MascotGenericFormat<I, P>> for MGFVec<I, P> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = MascotGenericFormat<I, P>>,
    {
        Self {
            mascot_generic_formats: iter.into_iter().collect(),
        }
    }
}

impl<I, P: SpectrumFloat> IntoIterator for MGFVec<I, P> {
    type IntoIter = alloc::vec::IntoIter<MascotGenericFormat<I, P>>;
    type Item = MascotGenericFormat<I, P>;

    fn into_iter(self) -> Self::IntoIter {
        self.mascot_generic_formats.into_iter()
    }
}

impl<'a, I, P: SpectrumFloat> IntoIterator for &'a MGFVec<I, P> {
    type IntoIter = core::slice::Iter<'a, MascotGenericFormat<I, P>>;
    type Item = &'a MascotGenericFormat<I, P>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<I, P> FromStr for MGFVec<I, P>
where
    I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug,
    P: SpectrumFloat,
{
    type Err = MascotError;

    fn from_str(s: &str) -> Result<Self> {
        let mut mascot_generic_formats = Self {
            mascot_generic_formats: Vec::new(),
        };
        let mut mascot_generic_format_builder = MascotGenericFormatBuilder::<I, P>::default();

        for (line_index, line) in s.lines().enumerate() {
            if line.is_empty() {
                continue;
            }

            let line_number = line_index + 1;
            mascot_generic_format_builder
                .digest_line(line)
                .map_err(|source| MascotError::InputLine {
                    line_number,
                    line: line.to_string(),
                    source: Box::new(source),
                })?;
            if mascot_generic_format_builder.can_build() {
                mascot_generic_formats.mascot_generic_formats.push(
                    mascot_generic_format_builder.build().map_err(|source| {
                        MascotError::InputLine {
                            line_number,
                            line: line.to_string(),
                            source: Box::new(source),
                        }
                    })?,
                );
                mascot_generic_format_builder = MascotGenericFormatBuilder::<I, P>::default();
            } else if mascot_generic_format_builder.can_skip_empty_section() {
                return Err(MascotError::InputLine {
                    line_number,
                    line: line.to_string(),
                    source: Box::new(MascotError::EmptyPeakVectors),
                });
            }
        }

        Ok(mascot_generic_formats)
    }
}

impl<I, P: SpectrumFloat> AsRef<[MascotGenericFormat<I, P>]> for MGFVec<I, P> {
    fn as_ref(&self) -> &[MascotGenericFormat<I, P>] {
        self.mascot_generic_formats.as_slice()
    }
}

impl<I: Copy, P: SpectrumFloat> Spectra for MGFVec<I, P> {
    type Spectrum = MascotGenericFormat<I, P>;
    type SpectraIter<'a>
        = core::slice::Iter<'a, MascotGenericFormat<I, P>>
    where
        Self: 'a;

    fn spectra(&self) -> Self::SpectraIter<'_> {
        self.mascot_generic_formats.iter()
    }

    fn len(&self) -> usize {
        self.mascot_generic_formats.len()
    }
}

impl<I, P: SpectrumFloat> Index<usize> for MGFVec<I, P> {
    type Output = MascotGenericFormat<I, P>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.mascot_generic_formats[index]
    }
}
