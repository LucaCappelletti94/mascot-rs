use mass_spectrometry::prelude::GenericSpectrumMutationError;
use thiserror::Error;

/// Crate-wide result type.
pub type Result<T> = std::result::Result<T, MascotError>;

/// Errors returned while parsing and validating MGF documents.
#[derive(Debug, Error)]
pub enum MascotError {
    /// A source file could not be read.
    #[error("could not read MGF file \"{path}\": {source}")]
    Io {
        /// Path that could not be read.
        path: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A builder is missing a required field.
    #[error("could not build {builder}: {field} is missing")]
    MissingField {
        /// Builder or object being created.
        builder: &'static str,
        /// Missing field name.
        field: &'static str,
    },
    /// A line could not be parsed into a supported value.
    #[error("could not parse {field} from line \"{line}\"")]
    ParseField {
        /// Field being parsed.
        field: &'static str,
        /// Original input line.
        line: String,
    },
    /// A line provided a non-finite floating-point value.
    #[error("line \"{line}\" contains a non-finite {field}")]
    NonFiniteField {
        /// Field being parsed.
        field: &'static str,
        /// Original input line.
        line: String,
    },
    /// A line provided a zero or negative value for a strictly positive field.
    #[error("line \"{line}\" contains a zero or negative {field}; it must be strictly positive")]
    NonPositiveField {
        /// Field being parsed.
        field: &'static str,
        /// Original input line.
        line: String,
    },
    /// A field appeared more than once with a different value.
    #[error("{field} was already encountered and is now different in line \"{line}\"")]
    ConflictingField {
        /// Field name.
        field: &'static str,
        /// Original input line.
        line: String,
    },
    /// A line is not supported by the current parser.
    #[error("{parser} does not support line \"{line}\"")]
    UnsupportedLine {
        /// Parser name.
        parser: &'static str,
        /// Original input line.
        line: String,
    },
    /// A line appeared before the parser was in a state that can accept it.
    #[error("line \"{line}\" appeared outside an open MGF ion section")]
    LineOutsideIonSection {
        /// Original input line.
        line: String,
    },
    /// A parsed charge value is invalid.
    #[error("invalid charge in line \"{line}\": {reason}")]
    InvalidCharge {
        /// Original input line.
        line: String,
        /// Reason the charge is invalid.
        reason: &'static str,
    },
    /// Charge cannot be zero in validated metadata.
    #[error("charge must be non-zero")]
    ZeroCharge,
    /// `SCANS` does not match the feature id.
    #[error("SCANS is not -1 or equal to FEATURE_ID in line \"{line}\"")]
    ScanFeatureIdMismatch {
        /// Original input line.
        line: String,
    },
    /// Merged scan statistics are internally inconsistent.
    #[error("merged scan statistics do not add up to the total scan count")]
    MergedScanStatisticsMismatch,
    /// Peak m/z and intensity vectors have different lengths.
    #[error("m/z and intensity vectors have different lengths: {mz_len} and {intensity_len}")]
    PeakVectorLengthMismatch {
        /// Number of m/z values.
        mz_len: usize,
        /// Number of intensity values.
        intensity_len: usize,
    },
    /// A peak vector is empty.
    #[error("peak vectors must not be empty")]
    EmptyPeakVectors,
    /// Spectrum validation failed in the shared mass-spectrometry model.
    #[error("could not create spectrum: {0}")]
    SpectrumMutation(#[from] GenericSpectrumMutationError),
    /// First-level data is incompatible with metadata.
    #[error(
        "first-level minimum m/z {first_level_min_mz:?} does not match metadata PEPMASS {parent_ion_mass:?}"
    )]
    FirstLevelParentIonMassMismatch {
        /// Metadata parent ion mass.
        parent_ion_mass: f64,
        /// Minimum first-level m/z.
        first_level_min_mz: f64,
    },
    /// A validated metadata filename is empty.
    #[error("filename must not be empty")]
    EmptyFilename,
}
