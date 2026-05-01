use core::marker::PhantomData;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use indicatif::ProgressBar;
use mass_spectrometry::prelude::SpectrumFloat;

use crate::error::{MascotError, Result};
use crate::mascot_generic_format::MGFVec;

/// Current GNPS endpoint for the aggregated public MGF spectral library.
pub const GNPS_ALL_MGF_URL: &str = "https://external.gnps2.org/gnpslibrary/ALL_GNPS.mgf";

const GNPS_ALL_MGF_FILE_NAME: &str = "ALL_GNPS.mgf";

/// Verbosity used while downloading GNPS data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
#[cfg_attr(feature = "mem_size", mem_size(flat))]
pub enum GNPSVerbosity {
    /// Do not emit progress information.
    #[default]
    Quiet,
    /// Use an [`indicatif`] progress bar while downloading.
    Indicatif,
}

/// Builder for downloading and loading the GNPS MGF library.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
pub struct GNPSBuilder<P: SpectrumFloat = f64> {
    config: GNPSBuilderConfig,
    precision: PhantomData<P>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
struct GNPSBuilderConfig {
    url: String,
    target_directory: PathBuf,
    file_name: String,
    verbosity: GNPSVerbosity,
    force_download: bool,
}

impl<P: SpectrumFloat> Default for GNPSBuilder<P> {
    fn default() -> Self {
        Self {
            config: GNPSBuilderConfig {
                url: GNPS_ALL_MGF_URL.to_string(),
                target_directory: std::env::temp_dir().join("mascot-rs-gnps"),
                file_name: GNPS_ALL_MGF_FILE_NAME.to_string(),
                verbosity: GNPSVerbosity::Quiet,
                force_download: false,
            },
            precision: PhantomData,
        }
    }
}

impl<P: SpectrumFloat> GNPSBuilder<P> {
    /// Sets the source URL.
    #[must_use]
    pub fn url<S: Into<String>>(mut self, url: S) -> Self {
        self.config.url = url.into();
        self
    }

    /// Sets the directory where the GNPS MGF file is stored.
    #[must_use]
    pub fn target_directory<PathLike: AsRef<Path>>(mut self, target_directory: PathLike) -> Self {
        self.config.target_directory = target_directory.as_ref().to_path_buf();
        self
    }

    /// Sets the downloaded file name inside the target directory.
    #[must_use]
    pub fn file_name<S: Into<String>>(mut self, file_name: S) -> Self {
        self.config.file_name = file_name.into();
        self
    }

    /// Sets download verbosity.
    #[must_use]
    pub const fn verbosity(mut self, verbosity: GNPSVerbosity) -> Self {
        self.config.verbosity = verbosity;
        self
    }

    /// Sets whether to redownload the library even if the target file exists.
    #[must_use]
    pub const fn force_download(mut self, force_download: bool) -> Self {
        self.config.force_download = force_download;
        self
    }

    /// Returns the configured download path.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.config.target_directory.join(&self.config.file_name)
    }

    /// Downloads the GNPS library if needed and loads the valid MGF records.
    ///
    /// GNPS library exports can contain empty or malformed ion blocks. Those
    /// records are skipped and counted in the returned [`GNPSLoad`].
    ///
    /// # Errors
    /// Returns an error if the download fails, if the target file cannot be
    /// written, or if the downloaded file cannot be read back.
    pub async fn load(self) -> Result<GNPSLoad<P>> {
        std::future::ready(()).await;

        if self.config.file_name.is_empty() {
            return Err(MascotError::EmptyFilename);
        }

        let path = self.path();
        std::fs::create_dir_all(&self.config.target_directory).map_err(|source| {
            MascotError::Io {
                path: self.config.target_directory.display().to_string(),
                source,
            }
        })?;

        let bytes = if !self.config.force_download
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
            self.download_to_path(&path)?
        };

        let (spectra, skipped_records) = Self::load_path(&path)?;

        Ok(GNPSLoad {
            spectra,
            skipped_records,
            path,
            bytes,
        })
    }

    fn download_to_path(&self, path: &Path) -> Result<u64> {
        let mut response =
            ureq::get(&self.config.url)
                .call()
                .map_err(|source| MascotError::Download {
                    url: self.config.url.clone(),
                    source: Box::new(source),
                })?;
        let progress_bar = self.progress_bar(Self::content_length(&response));
        let file = std::fs::File::create(path).map_err(|source| MascotError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let mut writer = BufWriter::new(file);
        let mut reader = response.body_mut().as_reader();
        let mut buffer = vec![0_u8; 1024 * 1024].into_boxed_slice();
        let mut downloaded_bytes = 0_u64;

        loop {
            let read_bytes = reader.read(&mut buffer).map_err(|source| MascotError::Io {
                path: path.display().to_string(),
                source,
            })?;
            if read_bytes == 0 {
                break;
            }
            writer
                .write_all(&buffer[..read_bytes])
                .map_err(|source| MascotError::Io {
                    path: path.display().to_string(),
                    source,
                })?;
            let read_bytes = u64::try_from(read_bytes).map_err(|_| MascotError::Io {
                path: path.display().to_string(),
                source: std::io::Error::other("download chunk length does not fit in u64"),
            })?;
            downloaded_bytes += read_bytes;
            if let Some(progress_bar) = &progress_bar {
                progress_bar.inc(read_bytes);
            }
        }

        writer.flush().map_err(|source| MascotError::Io {
            path: path.display().to_string(),
            source,
        })?;
        if let Some(progress_bar) = progress_bar {
            progress_bar.finish_and_clear();
        }

        Ok(downloaded_bytes)
    }

    fn progress_bar(&self, content_length: Option<u64>) -> Option<ProgressBar> {
        if self.config.verbosity == GNPSVerbosity::Quiet {
            return None;
        }

        let progress_bar = content_length.map_or_else(ProgressBar::new_spinner, ProgressBar::new);
        progress_bar.set_message("Downloading GNPS MGF library");
        Some(progress_bar)
    }

    fn content_length(response: &ureq::http::Response<ureq::Body>) -> Option<u64> {
        response
            .headers()
            .get("content-length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
    }

    fn load_path(path: &Path) -> Result<(MGFVec<usize, P>, usize)> {
        MGFVec::<usize, P>::from_path_skipping_invalid_records(path)
    }
}

/// Result of loading the GNPS MGF library.
#[derive(Debug)]
#[cfg_attr(feature = "mem_size", derive(mem_dbg::MemSize))]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg))]
pub struct GNPSLoad<P: SpectrumFloat = f64> {
    spectra: MGFVec<usize, P>,
    skipped_records: usize,
    path: PathBuf,
    bytes: u64,
}

impl<P: SpectrumFloat> GNPSLoad<P> {
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

    /// Returns the number of GNPS ion blocks skipped during tolerant loading.
    #[must_use]
    pub const fn skipped_records(&self) -> usize {
        self.skipped_records
    }

    /// Returns the local path used for the GNPS MGF file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the size of the local GNPS MGF file in bytes.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }
}

impl<P: SpectrumFloat> AsRef<MGFVec<usize, P>> for GNPSLoad<P> {
    fn as_ref(&self) -> &MGFVec<usize, P> {
        self.spectra()
    }
}
