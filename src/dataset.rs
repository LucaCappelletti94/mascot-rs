use std::future::Future;
use std::pin::Pin;

use crate::error::Result;

/// Boxed future returned by dataset retrieval trait methods.
pub type DatasetFuture<T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'static>>;

/// Common interface for downloadable datasets exposed by this crate.
///
/// Implementations provide a `download` step that only ensures local files are
/// present and a `load` step that downloads if needed and parses the dataset.
pub trait Dataset {
    /// Result returned after the dataset files are present locally.
    type Download;

    /// Result returned after the local dataset files are parsed.
    type Load;

    /// Downloads or reuses the local dataset files without parsing them.
    fn download(self) -> DatasetFuture<Self::Download>;

    /// Downloads the dataset if needed and parses it.
    fn load(self) -> DatasetFuture<Self::Load>;
}
