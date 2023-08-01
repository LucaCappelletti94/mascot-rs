use std::{ops::Add, fmt::Debug};

pub struct MergeScansMetadata<I> {
    scans: Vec<I>,
    removed_due_to_low_quality: I,
    removed_due_to_low_cosine: I,
}

impl<I: Add + Eq + Debug> MergeScansMetadata<I> {
    pub fn scans(&self) -> &[I] {
        &self.scans
    }

    pub fn removed_due_to_low_quality(&self) -> &I {
        &self.removed_due_to_low_quality
    }

    pub fn removed_due_to_low_cosine(&self) -> &I {
        &self.removed_due_to_low_cosine
    }

    pub fn new(scans: Vec<I>, removed_due_to_low_quality: I, removed_due_to_low_cosine: I) -> Result<Self, String> {
        if scans.is_empty() {
            return Err(concat!("No scans were provided.",).to_string());
        }

        Ok(Self {
            scans,
            removed_due_to_low_quality,
            removed_due_to_low_cosine,
        })
    }
}
