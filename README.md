# mascot-rs

[![Build status](https://github.com/lucacappelletti94/mascot-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/lucacappelletti94/mascot-rs/actions)
[![codecov](https://codecov.io/gh/lucacappelletti94/mascot-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/lucacappelletti94/mascot-rs)
[![Crates.io](https://img.shields.io/crates/v/mascot-rs.svg)](https://crates.io/crates/mascot-rs)
[![Documentation](https://docs.rs/mascot-rs/badge.svg)](https://docs.rs/mascot-rs)

Parsing utilities for Mascot Generic Format (MGF) spectra. Algorithmic work is delegated to the shared [`mass_spectrometry`](https://github.com/earth-metabolome-initiative/mass-spectrometry-traits) traits and structs exposed through the prelude.

## Feature Flags

Default features enable `std` and `mem_dbg`. Disabling defaults keeps the
string and iterator parser APIs available for `no_std` targets with `alloc`.
File IO, dataset downloading/loading, and progress reporting require `std`.
Path-based loading supports uncompressed MGF plus `.zst`, `.zstd`, `.gz`, and
`.gzip` files.

## Parsing Documents

Use [`MGFVec`] when parsing a full MGF document. Parsed records can be
filtered, processed, and written back out with the same extension convention as
loading: `.mgf.zst` and `.mgf.gz` files are compressed automatically.

```rust
# #[cfg(feature = "std")]
# fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
use mascot_rs::prelude::*;

let document = r#"BEGIN IONS
FEATURE_ID=1
PEPMASS=500.0
CHARGE=1
RTINSECONDS=10.0
MSLEVEL=2
SMILES=CCO
IONMODE=Positive
SOURCE_INSTRUMENT=LC-ESI-Orbitrap
NAME=Ethanol example
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
assert_eq!(spectra[0].feature_id(), Some(1));
assert_eq!(
    spectra[0].metadata().smiles().map(ToString::to_string).as_deref(),
    Some("CCO")
);
assert_eq!(spectra[0].ion_mode(), Some(IonMode::Positive));
assert_eq!(
    spectra[0].source_instrument(),
    Some(Instrument::Orbitrap)
);
assert_eq!(
    spectra[0].metadata().arbitrary_metadata_value("NAME"),
    Some("Ethanol example")
);
assert_eq!(spectra[1].precursor_mz().to_bits(), 600.0_f64.to_bits());

let mut positive_orbitrap: MGFVec<usize> = spectra
    .into_iter()
    .filter(|record| record.ion_mode() == Some(IonMode::Positive))
    .filter(|record| record.source_instrument() == Some(Instrument::Orbitrap))
    .collect();
positive_orbitrap
    .iter_mut()
    .for_each(|record| {
        let _ = record
            .metadata_mut()
            .insert_arbitrary_metadata("EXPORT_BATCH", "curated");
    });
let total_peaks = positive_orbitrap
    .spectra()
    .map(Spectrum::len)
    .sum::<usize>();

let mut buffer = Vec::new();
positive_orbitrap.write_to(&mut buffer)?;
let reparsed: MGFVec<usize> = std::str::from_utf8(&buffer)?.parse()?;

let path = std::env::temp_dir().join(format!(
    "mascot-rs-parse-write-example-{}.mgf.zst",
    std::process::id()
));
positive_orbitrap.to_path(&path)?;
let from_disk: MGFVec<usize> = MGFVec::from_path(&path)?;
std::fs::remove_file(path)?;

assert_eq!(total_peaks, 1);
assert_eq!(reparsed.len(), 1);
assert_eq!(
    reparsed[0]
        .metadata()
        .arbitrary_metadata_value("EXPORT_BATCH"),
    Some("curated")
);
assert_eq!(from_disk.len(), 1);
# Ok(())
# }
# #[cfg(not(feature = "std"))]
# fn main() {}
```

Files can also be parsed directly from a path, including compressed `.mgf.zst`
and `.mgf.gz` files.

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

## Streaming Records

Use `MGFIter` when records should be read one by one instead of collecting a
whole document into memory. This is the preferred shape for very large MGF
documents and sharded corpora.

```rust
# #[cfg(feature = "std")]
# fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
use mascot_rs::prelude::*;

let document = r#"BEGIN IONS
PEPMASS=500.0
CHARGE=1
MSLEVEL=2
100.0 2.0
SCANS=-1
END IONS

BEGIN IONS
PEPMASS=600.0
CHARGE=1
MSLEVEL=2
200.0 3.0
SCANS=-1
END IONS
"#;

let mut records = MGFVec::<usize>::iter_from_str(document);

let first = records
    .next()
    .transpose()?
    .ok_or_else(|| std::io::Error::other("missing first MGF record"))?;
let second = records
    .next()
    .transpose()?
    .ok_or_else(|| std::io::Error::other("missing second MGF record"))?;

assert_eq!(first.precursor_mz().to_bits(), 500.0_f64.to_bits());
assert_eq!(second.precursor_mz().to_bits(), 600.0_f64.to_bits());
assert!(records.next().is_none());
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

assert_eq!(record.feature_id(), Some(1));
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
below writes a small local `ALL_GNPS.mgf` file first, so the builder downloads
from the local cache, then loads that existing file without performing a network
request. Dataset builders also implement `Dataset`, whose `download()` method
only ensures that the local dataset file exists.

```rust
# #[cfg(feature = "std")]
# fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
use mascot_rs::prelude::*;

let target_directory =
    std::env::temp_dir().join(format!("mascot-rs-readme-{}", std::process::id()));
let cached_path = target_directory.join("ALL_GNPS.mgf");
let _ = std::fs::remove_dir_all(&target_directory);
std::fs::create_dir_all(&target_directory)?;

std::fs::write(
    &cached_path,
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

let download = pollster::block_on(
    MGFVec::<usize, f32>::gnps()
        .target_directory(&target_directory)
        .download(),
)?;
assert_eq!(download.path(), cached_path.as_path());

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

## `MassSpecGym`

The `MassSpecGym` helper is exposed through
`MGFVec::<usize, P>::mass_spec_gym()`. It targets the public Hugging Face
`data/auxiliary/MassSpecGym.mgf` file, which contains 231,104 benchmark spectra.
The loader normalizes `MassSpecGym`-specific headers such as `IDENTIFIER`,
`PRECURSOR_MZ`, `ADDUCT`, and `INSTRUMENT_TYPE` into the strict parser while
preserving the original keys as arbitrary metadata.

```rust
# #[cfg(feature = "std")]
# fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
use mascot_rs::prelude::*;

let target_directory =
    std::env::temp_dir().join(format!("mascot-rs-mass-spec-gym-{}", std::process::id()));
let cached_path = target_directory.join("MassSpecGym.mgf");
let _ = std::fs::remove_dir_all(&target_directory);
std::fs::create_dir_all(&target_directory)?;

std::fs::write(
    &cached_path,
    r#"BEGIN IONS
IDENTIFIER=MassSpecGymID0000001
SMILES=CCO
INCHIKEY=LFQSCWFLJHTTHZ
FORMULA=C2H6O
PRECURSOR_FORMULA=C2H7O
PARENT_MASS=46.041865
PRECURSOR_MZ=47.049141
ADDUCT=[M+H]+
INSTRUMENT_TYPE=Orbitrap
COLLISION_ENERGY=20.0
FOLD=train
SIMULATION_CHALLENGE=True
31.0184 1.0
45.0335 0.5
END IONS
"#,
)?;

let download = pollster::block_on(
    MGFVec::<usize, f32>::mass_spec_gym()
        .target_directory(&target_directory)
        .download(),
)?;
assert_eq!(download.path(), cached_path.as_path());

let load = pollster::block_on(
    MGFVec::<usize, f32>::mass_spec_gym()
        .target_directory(&target_directory)
        .load(),
)?;

std::fs::remove_dir_all(&target_directory)?;

assert_eq!(load.spectra().len(), 1);
assert_eq!(load.spectra()[0].feature_id(), Some(1));
assert_eq!(load.spectra()[0].charge(), 1);
assert_eq!(
    load.spectra()[0]
        .metadata()
        .arbitrary_metadata_value("IDENTIFIER"),
    Some("MassSpecGymID0000001")
);
# Ok(())
# }
# #[cfg(not(feature = "std"))]
# fn main() {}
```

## GeMS-A10

The GeMS-A10 helper is exposed through `MGFVec::<usize, P>::gems_a10()`.
By default it targets Zenodo record `19980668` and the 24 compressed MGF part
files from the top-100 peaks conversion. The top-60 and top-40 peaks
conversions are available with `MGFVec::<usize, P>::gems_a10_top_60_peaks()`
or `MGFVec::<usize, P>::gems_a10_top_40_peaks()`, and with `.top_60_peaks()`
or `.top_40_peaks()` on the builder. They target Zenodo records `20001888`
and `20002962`, respectively. Uncached downloads use `zenodo-rs` and should be
awaited inside a Tokio runtime. The example below writes a small cached file
first, so the builder downloads from the local cache, then loads that local file
without performing a network request.

```rust
# #[cfg(feature = "std")]
# fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
use mascot_rs::prelude::*;

let target_directory =
    std::env::temp_dir().join(format!("mascot-rs-gems-a10-readme-{}", std::process::id()));
let cached_path = target_directory.join("cached-gems-a10.mgf");
let _ = std::fs::remove_dir_all(&target_directory);
std::fs::create_dir_all(&target_directory)?;

std::fs::write(
    &cached_path,
    r"BEGIN IONS
PEPMASS=500.0
CHARGE=1
MSLEVEL=2
FEATURE_ID=1
SCANS=1
100.0 2.0
END IONS
",
)?;

let download = pollster::block_on(
    MGFVec::<usize, f32>::gems_a10()
        .target_directory(&target_directory)
        .file_key("cached-gems-a10.mgf")
        .download(),
)?;
assert_eq!(download.files()[0].path(), cached_path.as_path());

let load = pollster::block_on(
    MGFVec::<usize, f32>::gems_a10()
        .target_directory(&target_directory)
        .file_key("cached-gems-a10.mgf")
        .load(),
)?;

std::fs::remove_dir_all(&target_directory)?;

assert_eq!(load.spectra().len(), 1);
assert_eq!(load.files()[0].key(), "cached-gems-a10.mgf");
# Ok(())
# }
# #[cfg(not(feature = "std"))]
# fn main() {}
```

[`MascotError::SingleRecordExpected`]: crate::error::MascotError::SingleRecordExpected
[`MGFVec`]: crate::mascot_generic_format::MGFVec
[`MascotGenericFormat`]: crate::mascot_generic_format::MascotGenericFormat
