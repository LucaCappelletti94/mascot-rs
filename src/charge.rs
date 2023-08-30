use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// A charge is a signed integer.
/// 
/// # Implementative details
/// Most commonly, charges are a small signed integer.
/// We expect the charges to be at most plus or minus 127.
pub struct Charge {
    charge: i8,
}

impl FromStr for Charge {
    type Err = String;

    /// Parses a string to a [`Charge`].
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
    /// assert_eq!(Charge::from_str("CHARGE=0").unwrap(), 0);
    /// assert_eq!(Charge::from_str("CHARGE=-0").unwrap(), 0);
    /// assert_eq!(Charge::from_str("CHARGE=0-").unwrap(), 0);
    /// assert_eq!(Charge::from_str("CHARGE=-1").unwrap(), -1);
    /// assert_eq!(Charge::from_str("CHARGE=1").unwrap(), 1);
    /// assert_eq!(Charge::from_str("CHARGE=1+").unwrap(), 1);
    /// assert_eq!(Charge::from_str("CHARGE=-2").unwrap(), -2);
    /// assert_eq!(Charge::from_str("CHARGE=2-").unwrap(), -2);
    /// assert_eq!(Charge::from_str("CHARGE=2").unwrap(), 2);
    /// assert_eq!(Charge::from_str("CHARGE=2+").unwrap(), 2);
    /// assert_eq!(Charge::from_str("CHARGE=3").unwrap(), 3);
    /// assert_eq!(Charge::from_str("CHARGE=3+").unwrap(), 3);
    /// assert_eq!(Charge::from_str("CHARGE=4").unwrap(), 4);
    /// assert_eq!(Charge::from_str("CHARGE=4+").unwrap(), 4);
    ///
    /// ```
    ///
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Some(s) = s.strip_prefix("CHARGE=") {
            // If a charge contains a minus sign, we must take it into
            // account when parsing the integer.

            let trimmed_string = s.trim_matches(|c| c == '+' || c == '-');

            let mut charge_value = trimmed_string
                .parse::<i8>()
                .map_err(|e| format!("Invalid charge: {}", e))?;

            if s.contains("-") {
                charge_value = -charge_value;
            }

            Ok(Self {
                charge: charge_value,
            })
        } else {
            Err(format!("Invalid charge: {}", s))
        }
    }
}

impl From<i8> for Charge {
    fn from(charge: i8) -> Self {
        Self { charge }
    }
}

impl PartialEq<i8> for Charge {
    fn eq(&self, other: &i8) -> bool {
        self.charge == *other
    }
}

impl PartialEq<Charge> for i8 {
    fn eq(&self, other: &Charge) -> bool {
        *self == other.charge
    }
}

impl From<Charge> for i8 {
    fn from(charge: Charge) -> Self {
        charge.charge
    }
}