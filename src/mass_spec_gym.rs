use alloc::collections::VecDeque;
use core::marker::PhantomData;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use mass_spectrometry::prelude::SpectrumFloat;

use crate::dataset::{Dataset, DatasetFuture, SingleFileDatasetConfig, SingleFileDatasetDownload};
use crate::error::{MascotError, Result};
use crate::mascot_generic_format::{MGFIter, MGFLineSource, MGFReader, MGFVec};

/// Current Hugging Face endpoint for the `MassSpecGym` benchmark MGF file.
pub const MASS_SPEC_GYM_MGF_URL: &str = "https://huggingface.co/datasets/roman-bushuiev/MassSpecGym/resolve/main/data/auxiliary/MassSpecGym.mgf?download=true";

/// File name used for the `MassSpecGym` benchmark MGF file.
pub const MASS_SPEC_GYM_MGF_FILE_NAME: &str = "MassSpecGym.mgf";

/// Number of spectra reported by the `MassSpecGym` Hugging Face dataset viewer.
pub const MASS_SPEC_GYM_SPECTRA_COUNT: usize = 231_104;

/// Builder for downloading and loading the `MassSpecGym` benchmark MGF dataset.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
pub struct MassSpecGymBuilder<P: SpectrumFloat = f64> {
    config: SingleFileDatasetConfig,
    precision: PhantomData<fn() -> P>,
}

impl<P: SpectrumFloat> Default for MassSpecGymBuilder<P> {
    fn default() -> Self {
        Self {
            config: SingleFileDatasetConfig::new(
                MASS_SPEC_GYM_MGF_URL,
                std::env::temp_dir().join("mascot-rs-mass-spec-gym"),
                MASS_SPEC_GYM_MGF_FILE_NAME,
                "Downloading MassSpecGym MGF dataset",
            ),
            precision: PhantomData,
        }
    }
}

impl<P: SpectrumFloat> MassSpecGymBuilder<P> {
    /// Sets the source URL.
    #[must_use]
    pub fn url<S: Into<String>>(mut self, url: S) -> Self {
        self.config.set_url(url);
        self
    }

    /// Sets the directory where the `MassSpecGym` MGF file is stored.
    #[must_use]
    pub fn target_directory<PathLike: AsRef<Path>>(mut self, target_directory: PathLike) -> Self {
        self.config.set_target_directory(target_directory);
        self
    }

    /// Sets the downloaded file name inside the target directory.
    #[must_use]
    pub fn file_name<S: Into<String>>(mut self, file_name: S) -> Self {
        self.config.set_file_name(file_name);
        self
    }

    /// Enables download progress reporting.
    #[must_use]
    pub const fn verbose(mut self) -> Self {
        self.config.enable_verbose();
        self
    }

    /// Sets whether to redownload the dataset even if the target file exists.
    #[must_use]
    pub const fn force_download(mut self, force_download: bool) -> Self {
        self.config.set_force_download(force_download);
        self
    }

    /// Returns the configured download path.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.config.path()
    }

    /// Downloads the `MassSpecGym` MGF file if needed without loading records.
    ///
    /// # Errors
    /// Returns an error if the configured file name is empty, if the target
    /// directory cannot be created, if the existing local file cannot be
    /// inspected, or if the remote dataset cannot be downloaded.
    pub async fn download(self) -> Result<MassSpecGymDownload> {
        std::future::ready(()).await;
        self.config
            .download()
            .map(MassSpecGymDownload::from_single_file)
    }

    /// Downloads the `MassSpecGym` MGF file if needed and loads valid records.
    ///
    /// The published MGF uses MassSpecGym-specific header keys. Loading
    /// normalizes those keys into the strict MGF parser while preserving the
    /// original keys as arbitrary metadata.
    ///
    /// # Errors
    /// Returns an error if the download fails, if the target file cannot be
    /// written, or if the downloaded file cannot be read back.
    pub async fn load(self) -> Result<MassSpecGymLoad<P>> {
        let download = self.download().await?;
        let (spectra, skipped_records) = Self::load_path(download.path())?;

        Ok(MassSpecGymLoad {
            spectra,
            skipped_records,
            path: download.path,
            bytes: download.bytes,
        })
    }

    fn load_path(path: &Path) -> Result<(MGFVec<usize, P>, usize)> {
        let file = std::fs::File::open(path).map_err(|source| MascotError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let source = MassSpecGymLineSource::new(MGFReader::new(BufReader::new(file)));
        let mut iterator =
            MGFIter::<usize, P, _>::from_line_source(source).skipping_invalid_records();
        let mut records = Vec::new();

        while let Some(record) = iterator.next().transpose()? {
            records.push(record);
        }

        let skipped_records = iterator.skipped_records();
        Ok((records.into_iter().collect(), skipped_records))
    }
}

impl<P> Dataset for MassSpecGymBuilder<P>
where
    P: SpectrumFloat + Send + 'static,
{
    type Download = MassSpecGymDownload;
    type Load = MassSpecGymLoad<P>;

    fn download(self) -> DatasetFuture<Self::Download> {
        Box::pin(Self::download(self))
    }

    fn load(self) -> DatasetFuture<Self::Load> {
        Box::pin(Self::load(self))
    }
}

/// Result of downloading the `MassSpecGym` MGF dataset.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
pub struct MassSpecGymDownload {
    path: PathBuf,
    bytes: u64,
}

impl MassSpecGymDownload {
    fn from_single_file(download: SingleFileDatasetDownload) -> Self {
        let (path, bytes) = download.into_parts();
        Self { path, bytes }
    }

    /// Returns the local path used for the `MassSpecGym` MGF file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the size of the local `MassSpecGym` MGF file in bytes.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }
}

/// Result of loading the `MassSpecGym` MGF dataset.
#[derive(Debug)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
pub struct MassSpecGymLoad<P: SpectrumFloat = f64> {
    spectra: MGFVec<usize, P>,
    skipped_records: usize,
    path: PathBuf,
    bytes: u64,
}

impl<P: SpectrumFloat> MassSpecGymLoad<P> {
    /// Returns the loaded spectra.
    #[must_use]
    pub const fn spectra(&self) -> &MGFVec<usize, P> {
        &self.spectra
    }

    /// Consumes the load result and returns the loaded spectra.
    #[must_use]
    pub fn into_spectra(self) -> MGFVec<usize, P> {
        self.spectra
    }

    /// Returns the number of `MassSpecGym` ion blocks skipped during tolerant loading.
    #[must_use]
    pub const fn skipped_records(&self) -> usize {
        self.skipped_records
    }

    /// Returns the local path used for the `MassSpecGym` MGF file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the size of the local `MassSpecGym` MGF file in bytes.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }
}

impl<P: SpectrumFloat> AsRef<MGFVec<usize, P>> for MassSpecGymLoad<P> {
    fn as_ref(&self) -> &MGFVec<usize, P> {
        self.spectra()
    }
}

struct MassSpecGymLineSource<S> {
    source: S,
    queued: VecDeque<String>,
    saw_level: bool,
    saw_charge: bool,
    saw_ion_mode: bool,
}

impl<S> MassSpecGymLineSource<S> {
    const fn new(source: S) -> Self {
        Self {
            source,
            queued: VecDeque::new(),
            saw_level: false,
            saw_charge: false,
            saw_ion_mode: false,
        }
    }

    fn reset_record_state(&mut self) {
        self.queued.clear();
        self.saw_level = false;
        self.saw_charge = false;
        self.saw_ion_mode = false;
    }

    fn normalize_line(&mut self, line: &str) -> String {
        match line {
            "BEGIN IONS" => {
                self.reset_record_state();
                return line.to_string();
            }
            "END IONS" => {
                self.queue_missing_required_metadata();
                if let Some(queued) = self.queued.pop_front() {
                    self.queued.push_back(line.to_string());
                    return queued;
                }
                return line.to_string();
            }
            _ => {}
        }

        if line.starts_with("MSLEVEL=") {
            self.saw_level = true;
        } else if line.starts_with("CHARGE=") {
            self.saw_charge = true;
        } else if line.starts_with("IONMODE=") {
            self.saw_ion_mode = true;
        } else if let Some(identifier) = line.strip_prefix("IDENTIFIER=") {
            if let Some(feature_id) = Self::feature_id_from_identifier(identifier) {
                self.queued.push_back(format!("FEATURE_ID={feature_id}"));
            }
        } else if let Some(precursor_mz) = line.strip_prefix("PRECURSOR_MZ=") {
            self.queued.push_back(format!("PEPMASS={precursor_mz}"));
        } else if let Some(instrument) = line.strip_prefix("INSTRUMENT_TYPE=") {
            self.queued
                .push_back(format!("SOURCE_INSTRUMENT={instrument}"));
        } else if let Some(adduct) = line.strip_prefix("ADDUCT=") {
            self.queue_adduct_metadata(adduct);
        }

        line.to_string()
    }

    fn queue_missing_required_metadata(&mut self) {
        if !self.saw_level {
            self.queued.push_back("MSLEVEL=2".to_string());
            self.saw_level = true;
        }
        if !self.saw_charge {
            self.queued.push_back("CHARGE=0".to_string());
            self.saw_charge = true;
        }
    }

    fn queue_adduct_metadata(&mut self, adduct: &str) {
        let Some(charge) = Self::charge_from_adduct(adduct) else {
            return;
        };

        if !self.saw_charge {
            self.queued.push_back(format!("CHARGE={charge}"));
            self.saw_charge = true;
        }
        if !self.saw_ion_mode {
            let ion_mode = if charge.is_positive() {
                "Positive"
            } else {
                "Negative"
            };
            self.queued.push_back(format!("IONMODE={ion_mode}"));
            self.saw_ion_mode = true;
        }
    }

    fn feature_id_from_identifier(identifier: &str) -> Option<usize> {
        identifier
            .strip_prefix("MassSpecGymID")
            .and_then(|value| value.parse::<usize>().ok())
    }

    fn charge_from_adduct(adduct: &str) -> Option<i8> {
        let adduct = adduct.trim();
        let (without_sign, sign) = if let Some(without_sign) = adduct.strip_suffix('+') {
            (without_sign, 1_i8)
        } else {
            (adduct.strip_suffix('-')?, -1_i8)
        };

        let magnitude = without_sign
            .char_indices()
            .rev()
            .find_map(|(index, character)| {
                (!character.is_ascii_digit())
                    .then_some(&without_sign[index + character.len_utf8()..])
            })
            .filter(|digits| !digits.is_empty())
            .map_or(Some(1_i8), |digits| digits.parse::<i8>().ok())?;

        Some(sign.saturating_mul(magnitude))
    }
}

impl<S> MGFLineSource for MassSpecGymLineSource<S>
where
    S: MGFLineSource,
{
    type Line<'line>
        = String
    where
        Self: 'line;

    fn next_line(&mut self) -> Option<Result<Self::Line<'_>>> {
        if let Some(queued) = self.queued.pop_front() {
            return Some(Ok(queued));
        }

        let line = match self.source.next_line()? {
            Ok(line) => line.as_ref().to_string(),
            Err(source) => return Some(Err(source)),
        };
        Some(Ok(self.normalize_line(&line)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_charge_from_adduct() {
        assert_eq!(
            MassSpecGymLineSource::<MGFReader<std::io::Cursor<&str>>>::charge_from_adduct("[M+H]+"),
            Some(1)
        );
        assert_eq!(
            MassSpecGymLineSource::<MGFReader<std::io::Cursor<&str>>>::charge_from_adduct(
                "[M+2H]2+"
            ),
            Some(2)
        );
        assert_eq!(
            MassSpecGymLineSource::<MGFReader<std::io::Cursor<&str>>>::charge_from_adduct("[M-H]-"),
            Some(-1)
        );
        assert_eq!(
            MassSpecGymLineSource::<MGFReader<std::io::Cursor<&str>>>::charge_from_adduct(
                "[M-2H]2-"
            ),
            Some(-2)
        );
    }
}
