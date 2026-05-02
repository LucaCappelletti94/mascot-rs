#[cfg(feature = "std")]
use alloc::string::String;
use alloc::{boxed::Box, string::ToString, vec::Vec};
#[cfg(feature = "std")]
use core::fmt::Display;
use core::{
    fmt::Debug,
    marker::PhantomData,
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
use crate::numeric;

const MZ_FIELD: &str = "mass divided by charge ratio";
const INTENSITY_FIELD: &str = "fragment intensity";

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
        precursor_mz: P,
        mass_divided_by_charge_ratios: Vec<P>,
        fragment_intensities: Vec<P>,
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
            GenericSpectrum::<P>::try_with_capacity(precursor_mz.to_f64(), peak_capacity)?;

        let mut peaks = mass_divided_by_charge_ratios
            .into_iter()
            .zip(fragment_intensities)
            .collect::<Vec<_>>();
        for (mz, intensity) in &peaks {
            let mz = mz.to_f64();
            numeric::validate_positive_f64(mz, MZ_FIELD, &mz.to_string())?;
            let intensity = intensity.to_f64();
            numeric::validate_positive_f64(intensity, INTENSITY_FIELD, &intensity.to_string())?;
        }
        peaks.sort_by(|(left_mz, _), (right_mz, _)| left_mz.to_f64().total_cmp(&right_mz.to_f64()));

        let mut merged_peaks: Vec<(P, P)> = Vec::with_capacity(peaks.len());
        for (mz, intensity) in peaks {
            if let Some((last_mz, last_intensity)) = merged_peaks.last_mut() {
                if last_mz.to_f64().to_bits() == mz.to_f64().to_bits() {
                    *last_intensity = P::from_f64(last_intensity.to_f64() + intensity.to_f64())
                        .ok_or_else(|| MascotError::UnrepresentablePrecisionField {
                            field: "fragment intensity",
                            line: intensity.to_f64().to_string(),
                        })?;
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

    /// Returns mutable metadata for this MGF record.
    pub const fn metadata_mut(&mut self) -> &mut MascotGenericFormatMetadata<I> {
        &mut self.metadata
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
        for (key, value) in self.metadata.arbitrary_metadata() {
            Self::map_output_io(writeln!(writer, "{key}={value}"))?;
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

struct MGFRecordParser<I, P: SpectrumFloat = f64> {
    builder: MascotGenericFormatBuilder<I, P>,
}

impl<I, P: SpectrumFloat> Default for MGFRecordParser<I, P> {
    fn default() -> Self {
        Self {
            builder: MascotGenericFormatBuilder::<I, P>::default(),
        }
    }
}

impl<I, P: SpectrumFloat> MGFRecordParser<I, P> {
    fn reset(&mut self) {
        self.builder = MascotGenericFormatBuilder::<I, P>::default();
    }

    const fn section_open(&self) -> bool {
        self.builder.section_open()
    }
}

impl<I, P> MGFRecordParser<I, P>
where
    I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug,
    P: SpectrumFloat,
{
    fn digest_line(&mut self, line: &str) -> Result<Option<MascotGenericFormat<I, P>>> {
        self.builder.digest_line(line)?;

        if self.builder.can_build() {
            let builder = core::mem::take(&mut self.builder);
            return builder.build().map(Some);
        }

        if self.builder.can_skip_empty_section() {
            return Err(MascotError::EmptyPeakVectors);
        }

        Ok(None)
    }
}

/// A source of MGF document lines for [`MGFIter`].
pub trait MGFLineSource {
    #[doc(hidden)]
    /// Borrowed line type yielded by this source.
    type Line<'line>: AsRef<str>
    where
        Self: 'line;

    #[doc(hidden)]
    /// Returns the next input line.
    fn next_line(&mut self) -> Option<Result<Self::Line<'_>>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MGFIterMode {
    Strict,
    SkipInvalidRecords,
}

/// Borrowed string line source for [`MGFIter`].
#[derive(Debug, Clone)]
pub struct MGFStrLines<'a, T = core::str::Lines<'a>> {
    lines: T,
    lifetime: PhantomData<&'a str>,
}

impl<'a> MGFStrLines<'a> {
    /// Creates a borrowed line source from an MGF document string.
    #[must_use]
    pub fn new(document: &'a str) -> Self {
        Self::from_iterator(document.lines())
    }
}

impl<T> MGFStrLines<'_, T> {
    const fn from_iterator(lines: T) -> Self {
        Self {
            lines,
            lifetime: PhantomData,
        }
    }
}

impl<'a, T> MGFLineSource for MGFStrLines<'a, T>
where
    T: Iterator<Item = &'a str>,
{
    type Line<'line>
        = &'a str
    where
        Self: 'line;

    fn next_line(&mut self) -> Option<Result<Self::Line<'_>>> {
        self.lines.next().map(Ok)
    }
}

#[cfg(feature = "std")]
/// Buffered reader line source for [`MGFIter`].
#[derive(Debug)]
pub struct MGFReader<R> {
    reader: R,
    line: String,
}

#[cfg(feature = "std")]
impl<R> MGFReader<R> {
    /// Creates a line source from a buffered reader.
    #[must_use]
    pub const fn new(reader: R) -> Self {
        Self {
            reader,
            line: String::new(),
        }
    }
}

#[cfg(feature = "std")]
impl<R> MGFLineSource for MGFReader<R>
where
    R: BufRead,
{
    type Line<'line>
        = &'line str
    where
        Self: 'line;

    fn next_line(&mut self) -> Option<Result<Self::Line<'_>>> {
        self.line.clear();
        match self.reader.read_line(&mut self.line) {
            Ok(0) => None,
            Ok(_) => Some(Ok(self.line.trim_end_matches(['\r', '\n']))),
            Err(source) => Some(Err(MascotError::InputIo { source })),
        }
    }
}

/// Streaming iterator over MGF ion blocks.
///
/// Each successful item is one fully parsed [`MascotGenericFormat`]. The
/// iterator reads input incrementally and stops after the first I/O, parse, or
/// build error because the parser state is no longer reliable after that point.
pub struct MGFIter<I, P: SpectrumFloat = f64, S = MGFStrLines<'static>> {
    source: S,
    parser: MGFRecordParser<I, P>,
    line_number: usize,
    skipped_records: usize,
    mode: MGFIterMode,
    discarding_invalid_record: bool,
    finished: bool,
}

#[cfg(feature = "std")]
/// MGF iterator type returned for path-based readers.
pub type MGFPathIter<I, P = f64> = MGFIter<I, P, MGFReader<Box<dyn BufRead>>>;

impl<I, P, S> MGFIter<I, P, S>
where
    P: SpectrumFloat,
    S: MGFLineSource,
{
    /// Creates a streaming MGF iterator from a line source.
    #[must_use]
    pub fn from_line_source(source: S) -> Self {
        Self {
            source,
            parser: MGFRecordParser::<I, P>::default(),
            line_number: 0,
            skipped_records: 0,
            mode: MGFIterMode::Strict,
            discarding_invalid_record: false,
            finished: false,
        }
    }

    /// Skips invalid records instead of stopping at the first parse error.
    ///
    /// I/O errors are still returned. Use [`Self::skipped_records`] after
    /// exhausting the iterator to inspect how many malformed ion blocks were
    /// skipped.
    #[must_use]
    pub const fn skipping_invalid_records(mut self) -> Self {
        self.mode = MGFIterMode::SkipInvalidRecords;
        self
    }

    /// Returns the number of malformed records skipped by this iterator.
    #[must_use]
    pub const fn skipped_records(&self) -> usize {
        self.skipped_records
    }
}

impl<'a, I, P> MGFIter<I, P, MGFStrLines<'a>>
where
    P: SpectrumFloat,
{
    /// Creates a streaming MGF iterator over a borrowed document string.
    #[must_use]
    pub fn from_document(document: &'a str) -> Self {
        Self::from_line_source(MGFStrLines::new(document))
    }
}

#[cfg(feature = "std")]
impl<I, P, R> MGFIter<I, P, MGFReader<R>>
where
    P: SpectrumFloat,
    R: BufRead,
{
    /// Creates a streaming MGF iterator from a buffered reader.
    ///
    /// Use [`Self::from_path`] when loading from a file path with automatic
    /// decompression.
    #[must_use]
    pub fn from_reader(reader: R) -> Self {
        Self::from_line_source(MGFReader::new(reader))
    }
}

#[cfg(feature = "std")]
impl<I, P> MGFIter<I, P, MGFReader<Box<dyn BufRead>>>
where
    P: SpectrumFloat,
{
    /// Creates a streaming MGF iterator from a path.
    ///
    /// Files ending in `.zst`, `.zstd`, `.gz`, or `.gzip` are decompressed
    /// while they are read.
    ///
    /// # Errors
    /// Returns an error if the path cannot be opened or the decompressor cannot
    /// be initialized.
    pub fn from_path<PathLike>(path: PathLike) -> Result<Self>
    where
        PathLike: AsRef<Path>,
    {
        Ok(Self::from_reader(MGFVec::<I, P>::reader_from_path(
            path.as_ref(),
        )?))
    }
}

impl<I, P, S> Iterator for MGFIter<I, P, S>
where
    I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug,
    P: SpectrumFloat,
    S: MGFLineSource,
{
    type Item = Result<MascotGenericFormat<I, P>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        loop {
            let line = match self.source.next_line() {
                Some(Ok(line)) => line,
                Some(Err(source)) => {
                    self.finished = true;
                    return Some(Err(source));
                }
                None => {
                    if self.mode == MGFIterMode::SkipInvalidRecords && self.parser.section_open() {
                        self.skipped_records += 1;
                        self.parser.reset();
                    }
                    self.finished = true;
                    return None;
                }
            };

            self.line_number += 1;
            let line = line.as_ref();
            if line.is_empty() {
                continue;
            }

            if self.mode == MGFIterMode::SkipInvalidRecords {
                if self.discarding_invalid_record {
                    if matches!(line, "BEGIN IONS" | "END IONS") {
                        self.discarding_invalid_record = false;
                    } else {
                        continue;
                    }
                }

                if line != "BEGIN IONS" && !self.parser.section_open() {
                    continue;
                }

                if line == "BEGIN IONS" && self.parser.section_open() {
                    self.skipped_records += 1;
                    self.parser.reset();
                }
            }

            match self.parser.digest_line(line) {
                Ok(Some(record)) => return Some(Ok(record)),
                Ok(None) => {}
                Err(source) => {
                    if self.mode == MGFIterMode::SkipInvalidRecords {
                        self.skipped_records += 1;
                        self.parser.reset();
                        self.discarding_invalid_record = true;
                        if line == "END IONS" {
                            self.discarding_invalid_record = false;
                        }
                        continue;
                    }
                    self.finished = true;
                    return Some(Err(MascotError::InputLine {
                        line_number: self.line_number,
                        line: line.to_string(),
                        source: Box::new(source),
                    }));
                }
            }
        }
    }
}

impl<I, P, S> core::iter::FusedIterator for MGFIter<I, P, S>
where
    I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug,
    P: SpectrumFloat,
    S: MGFLineSource,
{
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MGFPathCompression {
    Zstd,
    Gzip,
    Plain,
}

#[cfg(feature = "std")]
impl MGFPathCompression {
    fn from_path(path: &Path) -> Self {
        if Self::has_extension(path, &["zst", "zstd"]) {
            Self::Zstd
        } else if Self::has_extension(path, &["gz", "gzip"]) {
            Self::Gzip
        } else {
            Self::Plain
        }
    }

    fn has_extension(path: &Path, expected_extensions: &[&str]) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                expected_extensions
                    .iter()
                    .any(|expected_extension| extension.eq_ignore_ascii_case(expected_extension))
            })
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

    /// Creates a streaming MGF iterator from the file at the provided path.
    ///
    /// Files ending in `.zst`, `.zstd`, `.gz`, or `.gzip` are decompressed
    /// while they are read.
    ///
    /// # Errors
    /// Returns an error if the path cannot be opened or the decompressor cannot
    /// be initialized.
    #[cfg(feature = "std")]
    pub fn iter_from_path<PathLike>(path: PathLike) -> Result<MGFPathIter<I, P>>
    where
        PathLike: AsRef<Path>,
    {
        MGFIter::from_path(path)
    }

    #[cfg(feature = "std")]
    fn reader_from_path(path: &Path) -> Result<Box<dyn BufRead>> {
        let file = File::open(path).map_err(|source| MascotError::Io {
            path: path.display().to_string(),
            source,
        })?;

        match MGFPathCompression::from_path(path) {
            MGFPathCompression::Zstd => {
                let decoder =
                    zstd::stream::read::Decoder::new(file).map_err(|source| MascotError::Io {
                        path: path.display().to_string(),
                        source,
                    })?;
                Ok(Box::new(BufReader::new(decoder)))
            }
            MGFPathCompression::Gzip => Ok(Box::new(BufReader::new(
                flate2::read::MultiGzDecoder::new(file),
            ))),
            MGFPathCompression::Plain => Ok(Box::new(BufReader::new(file))),
        }
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

        match MGFPathCompression::from_path(path) {
            MGFPathCompression::Zstd => {
                let writer = BufWriter::new(file);
                let mut encoder =
                    zstd::stream::write::Encoder::new(writer, 0).map_err(|source| {
                        MascotError::Io {
                            path: path.display().to_string(),
                            source,
                        }
                    })?;
                write(&mut encoder)?;
                encoder.finish().map_err(|source| MascotError::Io {
                    path: path.display().to_string(),
                    source,
                })?;
                Ok(())
            }
            MGFPathCompression::Gzip => {
                let writer = BufWriter::new(file);
                let mut encoder =
                    flate2::write::GzEncoder::new(writer, flate2::Compression::default());
                write(&mut encoder)?;
                encoder.finish().map_err(|source| MascotError::Io {
                    path: path.display().to_string(),
                    source,
                })?;
                Ok(())
            }
            MGFPathCompression::Plain => {
                let mut writer = BufWriter::new(file);
                write(&mut writer)?;
                writer.flush().map_err(|source| MascotError::Io {
                    path: path.display().to_string(),
                    source,
                })?;
                Ok(())
            }
        }
    }

    fn collect_mgf_iter<S>(iterator: MGFIter<I, P, S>) -> Result<Self>
    where
        S: MGFLineSource,
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug,
    {
        let mascot_generic_formats =
            iterator.collect::<Result<Vec<MascotGenericFormat<I, P>>>>()?;

        Ok(Self {
            mascot_generic_formats,
        })
    }

    /// Creates a new vector of MGF objects from a buffered line reader.
    ///
    /// # Errors
    /// Returns an error if the reader cannot be read, if any input line cannot
    /// be parsed, or if any MGF section cannot be built.
    #[cfg(feature = "std")]
    pub fn from_reader<R>(reader: R) -> Result<Self>
    where
        R: BufRead,
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug,
    {
        Self::collect_mgf_iter(MGFIter::<I, P, MGFReader<R>>::from_reader(reader))
    }

    /// Creates a streaming MGF iterator over a borrowed document string.
    #[must_use]
    pub fn iter_from_str(document: &str) -> MGFIter<I, P, MGFStrLines<'_>> {
        MGFIter::from_document(document)
    }

    /// Creates a streaming MGF iterator from a buffered line reader.
    ///
    /// # Examples
    /// ```
    /// use mascot_rs::prelude::*;
    ///
    /// let document = "BEGIN IONS\n\
    /// PEPMASS=500.0\n\
    /// CHARGE=1\n\
    /// MSLEVEL=2\n\
    /// 100.0 2.0\n\
    /// SCANS=-1\n\
    /// END IONS\n";
    /// let mut records = MGFVec::<usize>::iter_from_reader(std::io::Cursor::new(document));
    ///
    /// let record = records
    ///     .next()
    ///     .transpose()?
    ///     .ok_or_else(|| std::io::Error::other("missing MGF record"))?;
    ///
    /// assert_eq!(record.precursor_mz().to_bits(), 500.0_f64.to_bits());
    /// assert!(records.next().is_none());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "std")]
    #[must_use]
    pub fn iter_from_reader<R>(reader: R) -> MGFIter<I, P, MGFReader<R>>
    where
        R: BufRead,
    {
        MGFIter::from_reader(reader)
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
        Self::collect_mgf_iter(
            MGFIter::<I, P, MGFStrLines<'a, T::IntoIter>>::from_line_source(
                MGFStrLines::from_iterator(iter.into_iter()),
            ),
        )
    }

    #[cfg(feature = "std")]
    pub(crate) fn from_reader_skipping_invalid_records<R>(reader: R) -> Result<(Self, usize)>
    where
        R: BufRead,
        I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug,
    {
        let mut iterator =
            MGFIter::<I, P, MGFReader<R>>::from_reader(reader).skipping_invalid_records();
        let mut mascot_generic_formats = Vec::new();

        while let Some(record) = iterator.next().transpose()? {
            mascot_generic_formats.push(record);
        }

        let skipped_records = iterator.skipped_records();

        Ok((
            Self {
                mascot_generic_formats,
            },
            skipped_records,
        ))
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

    /// Returns a mutable iterator over the MGF records in the collection.
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, MascotGenericFormat<I, P>> {
        self.mascot_generic_formats.iter_mut()
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

impl<'a, I, P: SpectrumFloat> IntoIterator for &'a mut MGFVec<I, P> {
    type IntoIter = core::slice::IterMut<'a, MascotGenericFormat<I, P>>;
    type Item = &'a mut MascotGenericFormat<I, P>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<I, P> FromStr for MGFVec<I, P>
where
    I: Copy + From<usize> + FromStr + Add<Output = I> + Eq + Debug,
    P: SpectrumFloat,
{
    type Err = MascotError;

    fn from_str(s: &str) -> Result<Self> {
        Self::collect_mgf_iter(Self::iter_from_str(s))
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
