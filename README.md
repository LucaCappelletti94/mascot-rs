# mascot-rs

[![Build status](https://github.com/lucacappelletti94/mascot-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/lucacappelletti94/mascot-rs/actions)
[![Crates.io](https://img.shields.io/crates/v/mascot-rs.svg)](https://crates.io/crates/mascot-rs)
[![Documentation](https://docs.rs/mascot-rs/badge.svg)](https://docs.rs/mascot-rs)

Parsing utilities for Mascot Generic Format (MGF) spectra. Algorithmic work is delegated to the shared [`mass_spectrometry`](https://github.com/earth-metabolome-initiative/mass-spectrometry-traits) traits and structs exposed through the prelude.

## Feature Flags

Default features enable `std` and `mem_dbg`. Disabling defaults keeps the
string and iterator parser APIs available for `no_std` targets with `alloc`.
File IO, GNPS downloading/loading, and progress reporting require `std`.
Path-based loading supports uncompressed MGF plus `.zst`, `.zstd`, `.gz`, and
`.gzip` files.

## Parsing Documents

Use [`MGFVec`] when parsing a full MGF document.

```rust
use mascot_rs::prelude::*;

# fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
let document = r#"BEGIN IONS
FEATURE_ID=1
PEPMASS=500.0
CHARGE=1
RTINSECONDS=10.0
MSLEVEL=2
100.0 2.0
SCANS=1
END IONS

BEGIN IONS
FEATURE_ID=2
PEPMASS=600.0
CHARGE=1
RTINSECONDS=12.0
MSLEVEL=2
200.0 3.0
SCANS=2
END IONS
"#;

let spectra: MGFVec<usize> = document.parse()?;

assert_eq!(spectra.len(), 2);
assert_eq!(spectra[0].feature_id(), 1);
assert_eq!(spectra[1].precursor_mz().to_bits(), 600.0_f64.to_bits());
# Ok(())
# }
```

Files can be parsed directly from a path, including compressed `.mgf.zst` and
`.mgf.gz` files.

```rust
# #[cfg(feature = "std")]
# fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
use mascot_rs::prelude::*;

let spectra: MGFVec<usize> =
    MGFVec::from_path("tests/data/20220513_PMA_DBGI_01_04_003.mgf")?;

assert_eq!(spectra.len(), 74);
# Ok(())
# }
# #[cfg(not(feature = "std"))]
# fn main() {}
```

## Parsing One Record

Use [`MascotGenericFormat`] when the input must contain exactly one ion block.
Parsing zero or multiple blocks returns [`MascotError::SingleRecordExpected`].

```rust
use mascot_rs::prelude::*;

# fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
let document = r#"BEGIN IONS
FEATURE_ID=1
PEPMASS=500.0
CHARGE=1
RTINSECONDS=10.0
MSLEVEL=2
100.0 2.0
SCANS=1
END IONS
"#;

let record: MascotGenericFormat<usize> = document.parse()?;

assert_eq!(record.feature_id(), 1);
assert_eq!(record.len(), 1);
assert!(matches!(
    "".parse::<MascotGenericFormat<usize>>(),
    Err(MascotError::SingleRecordExpected { found: 0 })
));
# Ok(())
# }
```

## Precision

Spectra use `f64` storage by default. Select another precision with the second
generic parameter.

```rust
use mascot_rs::prelude::*;

# fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
let document = r#"BEGIN IONS
FEATURE_ID=1
PEPMASS=500.0
CHARGE=1
RTINSECONDS=10.0
MSLEVEL=2
100.0 2.0
200.0 3.0
SCANS=1
END IONS
"#;

let spectra: MGFVec<usize, f32> = document.parse()?;
let spectrum: &GenericSpectrum<f32> = spectra[0].as_ref();

assert_eq!(spectra[0].precursor_mz().to_bits(), 500.0_f32.to_bits());
assert_eq!(spectrum.mz_nth(0).to_bits(), 100.0_f32.to_bits());
# Ok(())
# }
```

## GNPS

The GNPS helper is exposed through `MGFVec::<usize, P>::gnps()`. The example
below writes a small local `ALL_GNPS.mgf` file first, so the builder loads an
existing file and does not perform a network request.

```rust
# #[cfg(feature = "std")]
# fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
use mascot_rs::prelude::*;

let target_directory =
    std::env::temp_dir().join(format!("mascot-rs-readme-{}", std::process::id()));
let _ = std::fs::remove_dir_all(&target_directory);
std::fs::create_dir_all(&target_directory)?;

std::fs::write(
    target_directory.join("ALL_GNPS.mgf"),
    r#"BEGIN IONS
PEPMASS=0.0
CHARGE=1
MSLEVEL=2
SCANS=1
100.0 2.0
END IONS
BEGIN IONS
PEPMASS=500.0
CHARGE=1
MSLEVEL=2
SCANS=2
100.0 2.0
END IONS
"#,
)?;

let load = pollster::block_on(
    MGFVec::<usize, f32>::gnps()
        .target_directory(&target_directory)
        .load(),
)?;

std::fs::remove_dir_all(&target_directory)?;

assert_eq!(load.spectra().len(), 1);
assert_eq!(load.skipped_records(), 1);
# Ok(())
# }
# #[cfg(not(feature = "std"))]
# fn main() {}
```

[`MGFVec`]: crate::mascot_generic_format::MGFVec
[`MascotGenericFormat`]: crate::mascot_generic_format::MascotGenericFormat
[`MascotError::SingleRecordExpected`]: crate::error::MascotError::SingleRecordExpected
