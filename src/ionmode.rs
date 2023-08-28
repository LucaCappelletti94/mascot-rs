//! Module to handle the possible ion mode as an enumeration.

use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum IonMode {
    Positive,
    Negative,
}

impl IonMode {
    /// Returns whether the string provided is a NaN ion mode.
    /// 
    /// # Arguments
    /// * `ionmode` - The string to check.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use mascot_rs::prelude::*;
    /// 
    /// assert!(IonMode::is_nan_ion_mode_from_str("IONMODE=N/A"));
    /// assert!(!IonMode::is_nan_ion_mode_from_str("IONMODE=positive"));
    /// assert!(!IonMode::is_nan_ion_mode_from_str("IONMODE=negative"));
    /// assert!(!IonMode::is_nan_ion_mode_from_str("kfukfuykfjkfue"));
    /// ```
    pub fn is_nan_ion_mode_from_str(ionmode: &str) -> bool {
        ionmode == "IONMODE=N/A" || ionmode == "N/A"
    }
}

impl FromStr for IonMode {
    type Err = String;

    /// Parses a string to a [`IonMode`].
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
    /// assert_eq!(IonMode::from_str("IONMODE=positive").unwrap(), IonMode::Positive);
    /// assert_eq!(IonMode::from_str("positive").unwrap(), IonMode::Positive);
    /// assert_eq!(IonMode::from_str("Positive").unwrap(), IonMode::Positive);
    /// assert_eq!(IonMode::from_str("IONMODE=negative").unwrap(), IonMode::Negative);
    /// assert_eq!(IonMode::from_str("negative").unwrap(), IonMode::Negative);
    /// assert_eq!(IonMode::from_str("Negative").unwrap(), IonMode::Negative);
    ///
    /// ```
    ///
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "IONMODE=positive" => Ok(Self::Positive),
            "positive" => Ok(Self::Positive),
            "Positive" => Ok(Self::Positive),
            "IONMODE=negative" => Ok(Self::Negative),
            "negative" => Ok(Self::Negative),
            "Negative" => Ok(Self::Negative),
            _ => Err(format!("Invalid ion mode: {}", s)),
        }
    }
}
