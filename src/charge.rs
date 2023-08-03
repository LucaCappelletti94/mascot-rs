use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Charge {
    One,
    OnePlus,
    Two,
    TwoPlus,
    Three,
    ThreePlus,
    Four,
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
            _ => Err(format!("Could not parse charge: {}", s)),
        }
    }
}

impl ToString for Charge {
    /// Converts a [`Charge`] to a string.
    /// 
    /// # Arguments
    /// * `charge` - The [`Charge`] to convert.
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
    /// 
    fn to_string(&self) -> String {
        match self {
            Self::One => "CHARGE=1".to_string(),
            Self::OnePlus => "CHARGE=1+".to_string(),
            Self::Two => "CHARGE=2".to_string(),
            Self::TwoPlus => "CHARGE=2+".to_string(),
            Self::Three => "CHARGE=3".to_string(),
            Self::ThreePlus => "CHARGE=3+".to_string(),
            Self::Four => "CHARGE=4".to_string(),
            Self::FourPlus => "CHARGE=4+".to_string(),
        }
    }
}