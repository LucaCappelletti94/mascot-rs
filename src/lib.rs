#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

/// Downloadable dataset traits.
#[cfg(feature = "std")]
pub mod dataset;
/// Error types returned by this crate.
pub mod error;
/// `GeMS-A10` dataset helpers.
#[cfg(feature = "std")]
pub mod gems_a10;
/// GNPS spectral library helpers.
#[cfg(feature = "std")]
pub mod gnps;
/// MGF record and collection types.
pub mod mascot_generic_format;
#[doc(hidden)]
mod mascot_generic_format_builder;
/// MGF metadata.
pub mod mascot_generic_format_metadata;
#[doc(hidden)]
mod mascot_generic_format_metadata_builder;
#[doc(hidden)]
mod numeric;

/// Commonly used crate exports.
pub mod prelude {
    #[cfg(feature = "std")]
    pub use crate::dataset::Dataset;
    #[cfg(feature = "std")]
    pub use crate::dataset::DatasetFuture;
    pub use crate::error::MascotError;
    pub use crate::error::Result;
    #[cfg(feature = "std")]
    pub use crate::gems_a10::GemsA10Builder;
    #[cfg(feature = "std")]
    pub use crate::gems_a10::GemsA10Download;
    #[cfg(feature = "std")]
    pub use crate::gems_a10::GemsA10FileDownload;
    #[cfg(feature = "std")]
    pub use crate::gems_a10::GemsA10FileLoad;
    #[cfg(feature = "std")]
    pub use crate::gems_a10::GemsA10Load;
    #[cfg(feature = "std")]
    pub use crate::gems_a10::GemsA10Verbosity;
    #[cfg(feature = "std")]
    pub use crate::gems_a10::GEMS_A10_MGF_PART_COUNT;
    #[cfg(feature = "std")]
    pub use crate::gems_a10::GEMS_A10_ZENODO_DOI;
    #[cfg(feature = "std")]
    pub use crate::gems_a10::GEMS_A10_ZENODO_RECORD_ID;
    #[cfg(feature = "std")]
    pub use crate::gnps::GNPSBuilder;
    #[cfg(feature = "std")]
    pub use crate::gnps::GNPSDownload;
    #[cfg(feature = "std")]
    pub use crate::gnps::GNPSLoad;
    #[cfg(feature = "std")]
    pub use crate::gnps::GNPSVerbosity;
    #[cfg(feature = "std")]
    pub use crate::gnps::GNPS_ALL_MGF_URL;
    pub use crate::mascot_generic_format::MGFIter;
    pub use crate::mascot_generic_format::MGFVec;
    pub use crate::mascot_generic_format::MascotGenericFormat;
    pub use crate::mascot_generic_format_metadata::Instrument;
    pub use crate::mascot_generic_format_metadata::IonMode;
    pub use crate::mascot_generic_format_metadata::MascotGenericFormatMetadata;
    pub use mass_spectrometry::prelude::{
        GenericSpectrum, Spectra, Spectrum, SpectrumAlloc, SpectrumFloat, SpectrumMut,
    };
    #[cfg(feature = "mem_dbg")]
    pub use mem_dbg::{DbgFlags, MemDbg};
    #[cfg(feature = "mem_size")]
    pub use mem_dbg::{MemSize, SizeFlags};
    pub use smiles_parser::prelude::Smiles;
}
