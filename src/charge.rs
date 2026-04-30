use std::{fmt, str::FromStr};

/// Supported precursor charge annotations in an MGF `CHARGE=` line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Charge {
    /// A charge written as `CHARGE=1`.
    One,
    /// A charge written as `CHARGE=1+`.
    OnePlus,
    /// A charge written as `CHARGE=2`.
    Two,
    /// A charge written as `CHARGE=2+`.
    TwoPlus,
    /// A charge written as `CHARGE=3`.
    Three,
    /// A charge written as `CHARGE=3+`.
    ThreePlus,
    /// A charge written as `CHARGE=4`.
    Four,
    /// A charge written as `CHARGE=4+`.
    FourPlus,
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
    /// assert_eq!(Charge::from_str("CHARGE=1").unwrap(), Charge::One);
    /// assert_eq!(Charge::from_str("CHARGE=1+").unwrap(), Charge::OnePlus);
    /// assert_eq!(Charge::from_str("CHARGE=2").unwrap(), Charge::Two);
    /// assert_eq!(Charge::from_str("CHARGE=2+").unwrap(), Charge::TwoPlus);
    /// assert_eq!(Charge::from_str("CHARGE=3").unwrap(), Charge::Three);
    /// assert_eq!(Charge::from_str("CHARGE=3+").unwrap(), Charge::ThreePlus);
    /// assert_eq!(Charge::from_str("CHARGE=4").unwrap(), Charge::Four);
    /// assert_eq!(Charge::from_str("CHARGE=4+").unwrap(), Charge::FourPlus);
    ///
    /// assert!(Charge::from_str("CHARGE=5+").is_err());
    ///
    /// ```
    ///
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CHARGE=1" => Ok(Self::One),
            "CHARGE=1+" => Ok(Self::OnePlus),
            "CHARGE=2" => Ok(Self::Two),
            "CHARGE=2+" => Ok(Self::TwoPlus),
            "CHARGE=3" => Ok(Self::Three),
            "CHARGE=3+" => Ok(Self::ThreePlus),
            "CHARGE=4" => Ok(Self::Four),
            "CHARGE=4+" => Ok(Self::FourPlus),
            _ => Err(format!("Could not parse charge: {s}")),
        }
    }
}

impl fmt::Display for Charge {
    /// Formats a [`Charge`] as its MGF `CHARGE=` line.
    ///
    /// # Examples
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    ///
    /// assert_eq!(Charge::One.to_string(), "CHARGE=1");
    /// assert_eq!(Charge::OnePlus.to_string(), "CHARGE=1+");
    /// assert_eq!(Charge::Two.to_string(), "CHARGE=2");
    /// assert_eq!(Charge::TwoPlus.to_string(), "CHARGE=2+");
    /// assert_eq!(Charge::Three.to_string(), "CHARGE=3");
    /// assert_eq!(Charge::ThreePlus.to_string(), "CHARGE=3+");
    /// assert_eq!(Charge::Four.to_string(), "CHARGE=4");
    /// assert_eq!(Charge::FourPlus.to_string(), "CHARGE=4+");
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::One => "CHARGE=1",
            Self::OnePlus => "CHARGE=1+",
            Self::Two => "CHARGE=2",
            Self::TwoPlus => "CHARGE=2+",
            Self::Three => "CHARGE=3",
            Self::ThreePlus => "CHARGE=3+",
            Self::Four => "CHARGE=4",
            Self::FourPlus => "CHARGE=4+",
        })
    }
}
