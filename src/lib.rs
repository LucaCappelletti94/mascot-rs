#![doc = include_str!("../README.md")]
/// Charge-state parsing and formatting.
pub mod charge;
/// Fragmentation spectrum level parsing.
pub mod fragmentation_spectra_level;
/// Shared line-oriented parser trait.
pub mod line_parser;
/// MGF record and collection types.
pub mod mascot_generic_format;
/// Builder for complete MGF records.
pub mod mascot_generic_format_builder;
/// Fragmentation peak data.
pub mod mascot_generic_format_data;
/// Builder for fragmentation peak data.
pub mod mascot_generic_format_data_builder;
/// MGF metadata.
pub mod mascot_generic_format_metadata;
/// Builder for MGF metadata.
pub mod mascot_generic_format_metadata_builder;
/// Metadata for merged scans.
pub mod merge_scans_metadata;
/// Builder for merged-scan metadata.
pub mod merge_scans_metadata_builder;
/// Floating-point NaN helper trait.
pub mod nan;
/// Strict positivity helper trait.
pub mod strictly_positive;
/// Zero-value helper trait.
pub mod zero;

/// Commonly used crate exports.
pub mod prelude {
    pub use crate::charge::Charge;
    pub use crate::fragmentation_spectra_level::FragmentationSpectraLevel;
    pub use crate::line_parser::LineParser;
    pub use crate::mascot_generic_format::MGFVec;
    pub use crate::mascot_generic_format::MascotGenericFormat;
    pub use crate::mascot_generic_format_builder::MascotGenericFormatBuilder;
    pub use crate::mascot_generic_format_data::MascotGenericFormatData;
    pub use crate::mascot_generic_format_data_builder::MascotGenericFormatDataBuilder;
    pub use crate::mascot_generic_format_metadata::MascotGenericFormatMetadata;
    pub use crate::mascot_generic_format_metadata_builder::MascotGenericFormatMetadataBuilder;
    pub use crate::merge_scans_metadata::MergeScansMetadata;
    pub use crate::merge_scans_metadata_builder::MergeScansMetadataBuilder;
    pub use crate::nan::NaN;
    pub use crate::strictly_positive::StrictlyPositive;
    pub use crate::zero::Zero;
}
