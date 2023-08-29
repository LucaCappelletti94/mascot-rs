//! Module to handle the possible ion mode as an enumeration.

use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum IonMode {
    Positive,
    Negative,
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
    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        s = s.trim();
        if s.is_empty() {
            return Err("Ion mode cannot be empty.".to_string());
        }
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
