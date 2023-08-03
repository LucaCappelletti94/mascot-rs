use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FragmentationSpectraLevel {
    One,
    Two,
}

impl PartialOrd for FragmentationSpectraLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for FragmentationSpectraLevel {}

impl Ord for FragmentationSpectraLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::One, Self::One) => std::cmp::Ordering::Equal,
            (Self::One, Self::Two) => std::cmp::Ordering::Less,
            (Self::Two, Self::One) => std::cmp::Ordering::Greater,
            (Self::Two, Self::Two) => std::cmp::Ordering::Equal,
        }
    }
}

impl FromStr for FragmentationSpectraLevel {
    type Err = String;

    /// Parses a string to a [`FragmentationSpectraLevel`].
    ///
    /// # Arguments
    /// * `s` - The string to parse.
    ///
    /// # Examples
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(FragmentationSpectraLevel::from_str("MSLEVEL=1").unwrap(), FragmentationSpectraLevel::One);
    /// assert_eq!(FragmentationSpectraLevel::from_str("MSLEVEL=2").unwrap(), FragmentationSpectraLevel::Two);
    ///
    /// assert!(FragmentationSpectraLevel::from_str("MSLEVEL=3").is_err());
    ///
    /// ```
    ///
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MSLEVEL=1" => Ok(Self::One),
            "MSLEVEL=2" => Ok(Self::Two),
            _ => Err(format!(
                "Could not parse fragmentation spectra level: {}",
                s
            )),
        }
    }
}
