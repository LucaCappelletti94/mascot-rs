use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Charge {
    One,
    OnePlus,
    TwoPlus,
    ThreePlus,
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
    /// assert_eq!(Charge::from_str("CHARGE=2+").unwrap(), Charge::TwoPlus);
    /// assert_eq!(Charge::from_str("CHARGE=3+").unwrap(), Charge::ThreePlus);
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
            "CHARGE=2+" => Ok(Self::TwoPlus),
            "CHARGE=3+" => Ok(Self::ThreePlus),
            "CHARGE=4+" => Ok(Self::FourPlus),
            _ => Err(format!("Could not parse charge: {}", s)),
        }
    }
}