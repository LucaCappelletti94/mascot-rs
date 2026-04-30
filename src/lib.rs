#![doc = include_str!("../README.md")]
/// Error types returned by this crate.
pub mod error;
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
    pub use crate::mascot_generic_format::MGFVec;
    pub use crate::mascot_generic_format::MascotGenericFormat;
    pub use crate::mascot_generic_format_metadata::MascotGenericFormatMetadata;
    pub use mass_spectrometry::prelude::{GenericSpectrum, Spectra, Spectrum};
}
