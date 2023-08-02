use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FragmentationSpectraLevel {
    One,
    Two,
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
