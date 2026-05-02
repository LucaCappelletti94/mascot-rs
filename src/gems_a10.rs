use core::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::time::Duration;

use indicatif::ProgressBar;
use mass_spectrometry::prelude::SpectrumFloat;
use zenodo_rs::{ArtifactSelector, Auth, RecordId, TransferProgress, ZenodoClient};

use crate::error::{MascotError, Result};
use crate::mascot_generic_format::MGFVec;

/// Zenodo record ID for the converted `GeMS-A10` MGF dataset.
pub const GEMS_A10_ZENODO_RECORD_ID: u64 = 19_980_668;

/// DOI for the converted `GeMS-A10` MGF dataset.
pub const GEMS_A10_ZENODO_DOI: &str = "10.5281/zenodo.19980668";

/// Number of compressed MGF part files in the converted `GeMS-A10` dataset.
pub const GEMS_A10_MGF_PART_COUNT: u8 = 24;

/// Verbosity used while downloading `GeMS-A10` data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
#[cfg_attr(feature = "mem_size", mem_size(flat))]
pub enum GemsA10Verbosity {
    /// Do not emit progress information.
    #[default]
    Quiet,
    /// Use an [`indicatif`] progress bar while downloading.
    Indicatif,
}

/// Builder for downloading and loading the converted `GeMS-A10` MGF dataset.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
pub struct GemsA10Builder<P: SpectrumFloat = f64> {
    config: GemsA10BuilderConfig,
    precision: PhantomData<fn() -> P>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
struct GemsA10BuilderConfig {
    record_id: u64,
    target_directory: PathBuf,
    file_keys: Vec<String>,
    verbosity: GemsA10Verbosity,
    force_download: bool,
    token: Option<String>,
    request_timeout_nanos: Option<u64>,
    connect_timeout_nanos: Option<u64>,
}

impl<P: SpectrumFloat> Default for GemsA10Builder<P> {
    fn default() -> Self {
        Self {
            config: GemsA10BuilderConfig {
                record_id: GEMS_A10_ZENODO_RECORD_ID,
                target_directory: std::env::temp_dir().join("mascot-rs-gems-a10"),
                file_keys: (0..GEMS_A10_MGF_PART_COUNT)
                    .map(Self::part_file_key_unchecked)
                    .collect(),
                verbosity: GemsA10Verbosity::Quiet,
                force_download: false,
                token: None,
                request_timeout_nanos: None,
                connect_timeout_nanos: None,
            },
            precision: PhantomData,
        }
    }
}

impl<P: SpectrumFloat> GemsA10Builder<P> {
    /// Returns the Zenodo file key for a published MGF part.
    ///
    /// # Errors
    /// Returns an error if the part number is outside the published range.
    pub fn part_file_key(part: u8) -> Result<String> {
        Self::validate_part(part)?;
        Ok(Self::part_file_key_unchecked(part))
    }

    /// Sets the Zenodo record ID.
    #[must_use]
    pub const fn zenodo_record_id(mut self, record_id: u64) -> Self {
        self.config.record_id = record_id;
        self
    }

    /// Returns the configured Zenodo record ID.
    #[must_use]
    pub const fn record_id(&self) -> u64 {
        self.config.record_id
    }

    /// Sets the directory where the dataset files are stored.
    #[must_use]
    pub fn target_directory<PathLike: AsRef<Path>>(mut self, target_directory: PathLike) -> Self {
        self.config.target_directory = target_directory.as_ref().to_path_buf();
        self
    }

    /// Selects one Zenodo file key to download and load.
    #[must_use]
    pub fn file_key<S: Into<String>>(mut self, file_key: S) -> Self {
        self.config.file_keys = vec![file_key.into()];
        self
    }

    /// Selects the Zenodo file keys to download and load.
    #[must_use]
    pub fn file_keys<T, S>(mut self, file_keys: T) -> Self
    where
        T: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.config.file_keys = file_keys.into_iter().map(Into::into).collect();
        self
    }

    /// Selects one published MGF part to download and load.
    ///
    /// # Errors
    /// Returns an error if the part number is outside the published range.
    pub fn part(mut self, part: u8) -> Result<Self> {
        self.config.file_keys = vec![Self::part_file_key(part)?];
        Ok(self)
    }

    /// Selects multiple published MGF parts to download and load.
    ///
    /// # Errors
    /// Returns an error if any part number is outside the published range.
    pub fn parts<T>(mut self, parts: T) -> Result<Self>
    where
        T: IntoIterator<Item = u8>,
    {
        self.config.file_keys = parts
            .into_iter()
            .map(Self::part_file_key)
            .collect::<Result<Vec<_>>>()?;
        Ok(self)
    }

    /// Selects all published MGF parts.
    #[must_use]
    pub fn all_parts(mut self) -> Self {
        self.config.file_keys = (0..GEMS_A10_MGF_PART_COUNT)
            .map(Self::part_file_key_unchecked)
            .collect();
        self
    }

    /// Returns the selected Zenodo file keys.
    #[must_use]
    pub fn selected_file_keys(&self) -> &[String] {
        &self.config.file_keys
    }

    /// Sets download verbosity.
    #[must_use]
    pub const fn verbosity(mut self, verbosity: GemsA10Verbosity) -> Self {
        self.config.verbosity = verbosity;
        self
    }

    /// Sets whether to redownload files even if they already exist locally.
    #[must_use]
    pub const fn force_download(mut self, force_download: bool) -> Self {
        self.config.force_download = force_download;
        self
    }

    /// Sets the Zenodo token used by `zenodo-rs`.
    ///
    /// The published `GeMS-A10` record is public, so the default empty token is
    /// sufficient for public downloads.
    #[must_use]
    pub fn token<S: Into<String>>(mut self, token: S) -> Self {
        self.config.token = Some(token.into());
        self
    }

    /// Sets the overall HTTP request timeout used by `zenodo-rs`.
    #[must_use]
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.config.request_timeout_nanos = Some(Self::duration_to_nanos(timeout));
        self
    }

    /// Sets the TCP connect timeout used by `zenodo-rs`.
    #[must_use]
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout_nanos = Some(Self::duration_to_nanos(timeout));
        self
    }

    /// Returns the configured local path for a Zenodo file key.
    #[must_use]
    pub fn path_for_file_key(&self, file_key: &str) -> PathBuf {
        self.config.target_directory.join(file_key)
    }

    /// Returns the configured local paths for the selected Zenodo file keys.
    #[must_use]
    pub fn paths(&self) -> Vec<PathBuf> {
        self.config
            .file_keys
            .iter()
            .map(|file_key| self.path_for_file_key(file_key))
            .collect()
    }

    /// Downloads the selected `GeMS-A10` files if needed and loads valid MGF records.
    ///
    /// The published dataset is split into compressed MGF part files. All
    /// malformed records are skipped and counted in the returned [`GemsA10Load`].
    ///
    /// # Errors
    /// Returns an error if the selected file list is empty, if a file key is
    /// empty, if a Zenodo operation fails, if a local file cannot be written, or
    /// if a downloaded file cannot be read back.
    pub async fn load(self) -> Result<GemsA10Load<P>> {
        std::future::ready(()).await;

        if self.config.file_keys.is_empty() {
            return Err(MascotError::MissingField {
                builder: "GemsA10Builder",
                field: "file_keys",
            });
        }
        if self.config.file_keys.iter().any(String::is_empty) {
            return Err(MascotError::EmptyFilename);
        }

        std::fs::create_dir_all(&self.config.target_directory).map_err(|source| {
            MascotError::Io {
                path: self.config.target_directory.display().to_string(),
                source,
            }
        })?;

        let mut client = None;
        let mut records = Vec::new();
        let mut files = Vec::with_capacity(self.config.file_keys.len());
        let mut skipped_records = 0_usize;
        let mut bytes = 0_u64;

        for file_key in &self.config.file_keys {
            let path = self.path_for_file_key(file_key);
            let file_bytes = if !self.config.force_download
                && path.try_exists().map_err(|source| MascotError::Io {
                    path: path.display().to_string(),
                    source,
                })? {
                std::fs::metadata(&path)
                    .map_err(|source| MascotError::Io {
                        path: path.display().to_string(),
                        source,
                    })?
                    .len()
            } else {
                if client.is_none() {
                    client = Some(self.client()?);
                }
                self.download_file(
                    client.as_ref().ok_or_else(|| MascotError::MissingField {
                        builder: "GemsA10Builder",
                        field: "zenodo_client",
                    })?,
                    file_key,
                    &path,
                )
                .await?
            };

            let (part_records, part_skipped_records) =
                MGFVec::<usize, P>::from_path_skipping_invalid_records(&path)?;
            records.extend(part_records);
            skipped_records += part_skipped_records;
            bytes += file_bytes;
            files.push(GemsA10FileLoad {
                key: file_key.clone(),
                path,
                bytes: file_bytes,
            });
        }

        Ok(GemsA10Load {
            spectra: records.into_iter().collect(),
            skipped_records,
            files,
            bytes,
        })
    }

    fn client(&self) -> Result<ZenodoClient> {
        let token = self.config.token.clone().unwrap_or_default();
        let mut builder = ZenodoClient::builder(Auth::new(token))
            .user_agent(format!("mascot-rs/{}", env!("CARGO_PKG_VERSION")));
        if let Some(timeout_nanos) = self.config.request_timeout_nanos {
            builder = builder.request_timeout(Duration::from_nanos(timeout_nanos));
        }
        if let Some(timeout_nanos) = self.config.connect_timeout_nanos {
            builder = builder.connect_timeout(Duration::from_nanos(timeout_nanos));
        }
        builder.build().map_err(|source| MascotError::Zenodo {
            operation: "client initialization".to_string(),
            source: Box::new(source),
        })
    }

    async fn download_file(
        &self,
        client: &ZenodoClient,
        file_key: &str,
        path: &Path,
    ) -> Result<u64> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| MascotError::Io {
                path: parent.display().to_string(),
                source,
            })?;
        }

        let selector = ArtifactSelector::file(RecordId(self.config.record_id), file_key);
        let progress = self.progress(file_key);
        let resolved = client
            .download_artifact_with_progress(&selector, path, progress)
            .await
            .map_err(|source| MascotError::Zenodo {
                operation: format!(
                    "download of record {} file {file_key}",
                    self.config.record_id
                ),
                source: Box::new(source),
            })?;
        Ok(resolved.bytes_written)
    }

    fn progress(&self, file_key: &str) -> GemsA10Progress {
        match self.config.verbosity {
            GemsA10Verbosity::Quiet => GemsA10Progress::Quiet,
            GemsA10Verbosity::Indicatif => {
                let progress_bar = ProgressBar::new_spinner();
                progress_bar.set_message(format!("Downloading {file_key}"));
                GemsA10Progress::Indicatif(progress_bar)
            }
        }
    }

    const fn validate_part(part: u8) -> Result<()> {
        if part >= GEMS_A10_MGF_PART_COUNT {
            return Err(MascotError::InvalidGemsA10Part {
                part,
                part_count: GEMS_A10_MGF_PART_COUNT,
            });
        }
        Ok(())
    }

    fn part_file_key_unchecked(part: u8) -> String {
        format!("GeMS_A10.mgf.part-{part:05}.mgf.zst")
    }

    fn duration_to_nanos(timeout: Duration) -> u64 {
        u64::try_from(timeout.as_nanos()).unwrap_or(u64::MAX)
    }
}

enum GemsA10Progress {
    Quiet,
    Indicatif(ProgressBar),
}

impl TransferProgress for GemsA10Progress {
    fn begin(&self, total_bytes: Option<u64>) {
        if let Self::Indicatif(progress_bar) = self {
            progress_bar.set_position(0);
            if let Some(total_bytes) = total_bytes {
                progress_bar.set_length(total_bytes);
            }
        }
    }

    fn advance(&self, delta: u64) {
        if let Self::Indicatif(progress_bar) = self {
            progress_bar.inc(delta);
        }
    }

    fn finish(&self) {
        if let Self::Indicatif(progress_bar) = self {
            progress_bar.finish_and_clear();
        }
    }
}

/// Metadata for a downloaded and loaded `GeMS-A10` file.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
pub struct GemsA10FileLoad {
    key: String,
    path: PathBuf,
    bytes: u64,
}

impl GemsA10FileLoad {
    /// Returns the Zenodo file key.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the local path used for the file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the size of the local file in bytes.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }
}

/// Result of loading selected `GeMS-A10` MGF files.
#[derive(Debug)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
pub struct GemsA10Load<P: SpectrumFloat = f64> {
    spectra: MGFVec<usize, P>,
    skipped_records: usize,
    files: Vec<GemsA10FileLoad>,
    bytes: u64,
}

impl<P: SpectrumFloat> GemsA10Load<P> {
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

    /// Returns the number of malformed ion blocks skipped during tolerant loading.
    #[must_use]
    pub const fn skipped_records(&self) -> usize {
        self.skipped_records
    }

    /// Returns per-file load metadata.
    #[must_use]
    pub fn files(&self) -> &[GemsA10FileLoad] {
        &self.files
    }

    /// Returns the total size of the local dataset files in bytes.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }
}

impl<P: SpectrumFloat> AsRef<MGFVec<usize, P>> for GemsA10Load<P> {
    fn as_ref(&self) -> &MGFVec<usize, P> {
        self.spectra()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_builder_selects_all_published_parts() {
        let builder = GemsA10Builder::<f64>::default();

        assert_eq!(builder.record_id(), GEMS_A10_ZENODO_RECORD_ID);
        assert_eq!(
            builder.selected_file_keys().len(),
            usize::from(GEMS_A10_MGF_PART_COUNT)
        );
        assert_eq!(
            builder.selected_file_keys().first().map(String::as_str),
            Some("GeMS_A10.mgf.part-00000.mgf.zst")
        );
        assert_eq!(
            builder.selected_file_keys().last().map(String::as_str),
            Some("GeMS_A10.mgf.part-00023.mgf.zst")
        );
    }

    #[test]
    fn validates_part_file_keys() -> Result<()> {
        assert_eq!(
            GemsA10Builder::<f64>::part_file_key(7)?,
            "GeMS_A10.mgf.part-00007.mgf.zst"
        );
        assert!(matches!(
            GemsA10Builder::<f64>::part_file_key(GEMS_A10_MGF_PART_COUNT),
            Err(MascotError::InvalidGemsA10Part { .. })
        ));
        Ok(())
    }
}
