#![doc = include_str!("../README.md")]
/// Error types returned by this crate.
pub mod error;
/// GNPS spectral library helpers.
pub mod gnps;
/// MGF record and collection types.
pub mod mascot_generic_format;
#[doc(hidden)]
mod mascot_generic_format_builder;
/// MGF metadata.
pub mod mascot_generic_format_metadata;
#[doc(hidden)]
mod mascot_generic_format_metadata_builder;

/// Commonly used crate exports.
pub mod prelude {
    pub use crate::error::MascotError;
    pub use crate::error::Result;
    pub use crate::gnps::GNPSBuilder;
    pub use crate::gnps::GNPSLoad;
    pub use crate::gnps::GNPSVerbosity;
    pub use crate::gnps::GNPS_ALL_MGF_URL;
    pub use crate::mascot_generic_format::MGFVec;
    pub use crate::mascot_generic_format::MascotGenericFormat;
    pub use crate::mascot_generic_format_metadata::MascotGenericFormatMetadata;
    pub use mass_spectrometry::prelude::{GenericSpectrum, Spectra, Spectrum, SpectrumFloat};
    pub use mem_dbg::{DbgFlags, MemDbg, MemSize, SizeFlags};
}
