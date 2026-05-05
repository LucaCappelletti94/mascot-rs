use core::marker::PhantomData;
use std::path::{Path, PathBuf};

use mass_spectrometry::prelude::SpectrumFloat;

use crate::dataset::{Dataset, DatasetFuture, SingleFileDatasetConfig, SingleFileDatasetDownload};
use crate::error::Result;
use crate::mascot_generic_format::MGFVec;

/// Zenodo record ID for the annotated harmonized MS2 MGF dataset.
pub const ANNOTATED_MS2_ZENODO_RECORD_ID: u64 = 20_039_648;

/// DOI for the annotated harmonized MS2 MGF dataset.
pub const ANNOTATED_MS2_ZENODO_DOI: &str = "10.5281/zenodo.20039648";

/// Current Zenodo endpoint for the annotated harmonized MS2 MGF file.
pub const ANNOTATED_MS2_MGF_URL: &str = "https://zenodo.org/api/records/20039648/files/combined-gnps-mass-spec-gym-npc-faithful.harmonized-subset.mgf.zst/content";

/// File name used for the annotated harmonized MS2 MGF file.
pub const ANNOTATED_MS2_MGF_FILE_NAME: &str =
    "combined-gnps-mass-spec-gym-npc-faithful.harmonized-subset.mgf.zst";

/// Number of spectra reported by the Zenodo record.
pub const ANNOTATED_MS2_SPECTRA_COUNT: usize = 439_403;

/// Builder for downloading and loading the annotated harmonized MS2 dataset.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
pub struct AnnotatedMs2Builder<P: SpectrumFloat = f64> {
    config: SingleFileDatasetConfig,
    precision: PhantomData<fn() -> P>,
}

impl<P: SpectrumFloat> Default for AnnotatedMs2Builder<P> {
    fn default() -> Self {
        Self {
            config: SingleFileDatasetConfig::new(
                ANNOTATED_MS2_MGF_URL,
                std::env::temp_dir().join("mascot-rs-annotated-ms2"),
                ANNOTATED_MS2_MGF_FILE_NAME,
                "Downloading annotated MS2 MGF dataset",
            ),
            precision: PhantomData,
        }
    }
}

impl<P: SpectrumFloat> AnnotatedMs2Builder<P> {
    /// Returns the Zenodo record ID for the annotated MS2 dataset.
    #[must_use]
    pub const fn record_id(&self) -> u64 {
        ANNOTATED_MS2_ZENODO_RECORD_ID
    }

    /// Returns the DOI for the annotated MS2 dataset.
    #[must_use]
    pub const fn doi(&self) -> &'static str {
        ANNOTATED_MS2_ZENODO_DOI
    }

    /// Sets the source URL.
    #[must_use]
    pub fn url<S: Into<String>>(mut self, url: S) -> Self {
        self.config.set_url(url);
        self
    }

    /// Sets the directory where the annotated MS2 MGF file is stored.
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

    /// Downloads the annotated MS2 MGF file if needed without loading records.
    ///
    /// # Errors
    /// Returns an error if the configured file name is empty, if the target
    /// directory cannot be created, if the existing local file cannot be
    /// inspected, or if the remote dataset cannot be downloaded.
    pub async fn download(self) -> Result<AnnotatedMs2Download> {
        std::future::ready(()).await;
        self.config
            .download()
            .map(AnnotatedMs2Download::from_single_file)
    }

    /// Downloads the annotated MS2 MGF file if needed and loads valid records.
    ///
    /// The published MGF is already normalized to the crate's structured
    /// headers. Malformed records are skipped and counted in the returned
    /// [`AnnotatedMs2Load`].
    ///
    /// # Errors
    /// Returns an error if the download fails, if the target file cannot be
    /// written, or if the downloaded file cannot be read back.
    pub async fn load(self) -> Result<AnnotatedMs2Load<P>> {
        let download = self.download().await?;
        let (spectra, skipped_records) = Self::load_path(download.path())?;

        Ok(AnnotatedMs2Load {
            spectra,
            skipped_records,
            path: download.path,
            bytes: download.bytes,
        })
    }

    fn load_path(path: &Path) -> Result<(MGFVec<P>, usize)> {
        MGFVec::<P>::from_path_skipping_invalid_records(path)
    }
}

impl<P> Dataset for AnnotatedMs2Builder<P>
where
    P: SpectrumFloat + Send + 'static,
{
    type Download = AnnotatedMs2Download;
    type Load = AnnotatedMs2Load<P>;

    fn download(self) -> DatasetFuture<Self::Download> {
        Box::pin(Self::download(self))
    }

    fn load(self) -> DatasetFuture<Self::Load> {
        Box::pin(Self::load(self))
    }
}

/// Result of downloading the annotated MS2 MGF dataset.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
pub struct AnnotatedMs2Download {
    path: PathBuf,
    bytes: u64,
}

impl AnnotatedMs2Download {
    fn from_single_file(download: SingleFileDatasetDownload) -> Self {
        let (path, bytes) = download.into_parts();
        Self { path, bytes }
    }

    /// Returns the local path used for the annotated MS2 MGF file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the size of the local annotated MS2 MGF file in bytes.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }
}

/// Result of loading the annotated MS2 MGF dataset.
#[derive(Debug)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
pub struct AnnotatedMs2Load<P: SpectrumFloat = f64> {
    spectra: MGFVec<P>,
    skipped_records: usize,
    path: PathBuf,
    bytes: u64,
}

impl<P: SpectrumFloat> AnnotatedMs2Load<P> {
    /// Returns the loaded spectra.
    #[must_use]
    pub const fn spectra(&self) -> &MGFVec<P> {
        &self.spectra
    }

    /// Consumes the load result and returns the loaded spectra.
    #[must_use]
    pub fn into_spectra(self) -> MGFVec<P> {
        self.spectra
    }

    /// Returns the number of ion blocks skipped during tolerant loading.
    #[must_use]
    pub const fn skipped_records(&self) -> usize {
        self.skipped_records
    }

    /// Returns the local path used for the annotated MS2 MGF file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the size of the local annotated MS2 MGF file in bytes.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }
}

impl<P: SpectrumFloat> AsRef<MGFVec<P>> for AnnotatedMs2Load<P> {
    fn as_ref(&self) -> &MGFVec<P> {
        self.spectra()
    }
}
