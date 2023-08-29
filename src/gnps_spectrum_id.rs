//! Module to validate GNPS spectrum IDs as a type.
//!
//! # Examples
//! A GNPS spectrum ID is a string such as:
//!
//! * CCMSLIB00005463540
//! * CCMSLIB00000845086
//! * CCMSLIB00004721996
//! * CCMSLIB00010010599
//! * CCMSLIB00010101988
//! * CCMSLIB00004679916
//! * CCMSLIB00010012246
//! * CCMSLIB00008851197
//! * CCMSLIB00004751435
//! * CCMSLIB00000001547
//! * CCMSLIB00010055263
//! * CCMSLIB00005489309
//! * CCMSLIB00005435444
//! * CCMSLIB00000204741
//! * CCMSLIB00000425029
//! * CCMSLIB00006112665
//! * CCMSLIB00006581625
//! * CCMSLIB00006354301
//! * CCMSLIB00004683236
//! * CCMSLIB00005463540
//! * CCMSLIB00000001547
//! * CCMSLIB00001058235
//! * CCMSLIB00000845086
//! * CCMSLIB00000084736
//! * CCMSLIB00004722107
//! * CCMSLIB00000479320
//! * CCMSLIB00000562749
//! * CCMSLIB00000077994
//! * CCMSLIB00000001547
//! * CCMSLIB00001058175
//! * CCMSLIB00000579358
//! * CCMSLIB00009919545
//! * CCMSLIB00000578447
//! * CCMSLIB00006675755
//! * CCMSLIB00000081017
//! * CCMSLIB00006672953
//! * CCMSLIB00000076959
//! * CCMSLIB00004751209
//! * CCMSLIB00006112554
//! * CCMSLIB00001058205
//! * CCMSLIB00000078899
//! * CCMSLIB00000004221
//! * CCMSLIB00009919030
//! * CCMSLIB00000845012
//! * CCMSLIB00009919300
//! * CCMSLIB00000208752
//! * CCMSLIB00010059112
//! * CCMSLIB00000078679
//! * CCMSLIB00000577491
//! * CCMSLIB00010007697
//! * CCMSLIB00005723212
//! * CCMSLIB00005720323
//! * CCMSLIB00005724063
//! * CCMSLIB00003134487
//! * CCMSLIB00000079350
//! * CCMSLIB00005720908
//!

use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GNPSSpectrumID {
    id: usize,
}

impl GNPSSpectrumID {
    /// Returns the ID of the [`GNPSSpectrumID`].
    ///
    /// # Arguments
    /// * `self` - The [`GNPSSpectrumID`] whose ID to return.
    ///
    /// # Examples
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    ///
    /// let id = 123456789;
    /// let gnps_spectrum_id = GNPSSpectrumID::from(id);
    ///
    /// assert_eq!(gnps_spectrum_id.id(), id);
    /// ```
    pub fn id(&self) -> usize {
        self.id
    }
}

impl From<usize> for GNPSSpectrumID {
    /// Converts a [`usize`] to a [`GNPSSpectrumID`].
    ///
    /// # Arguments
    /// * `id` - The [`usize`] to convert.
    ///
    /// # Examples
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    ///
    /// let id = 123456789;
    /// let gnps_spectrum_id = GNPSSpectrumID::from(id);
    ///
    /// assert_eq!(gnps_spectrum_id.id(), id);
    /// ```
    fn from(id: usize) -> Self {
        Self { id }
    }
}

impl FromStr for GNPSSpectrumID {
    type Err = String;

    /// Parses a string to a [`GNPSSpectrumID`].
    ///
    /// # Arguments
    /// * `s` - The string to parse.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mascot_rs::prelude::*;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(GNPSSpectrumID::from_str("CCMSLIB00005463540").unwrap(), GNPSSpectrumID::from(5463540));
    /// assert_eq!(GNPSSpectrumID::from_str("CCMSLIB00000845086").unwrap(), GNPSSpectrumID::from(845086));
    /// assert_eq!(GNPSSpectrumID::from_str("CCMSLIB00004721996").unwrap(), GNPSSpectrumID::from(4721996));
    /// assert_eq!(GNPSSpectrumID::from_str("CCMSLIB00010010599").unwrap(), GNPSSpectrumID::from(10010599));
    /// assert_eq!(GNPSSpectrumID::from_str("CCMSLIB00010101988").unwrap(), GNPSSpectrumID::from(10101988));
    /// assert_eq!(GNPSSpectrumID::from_str("CCMSLIB00004679916").unwrap(), GNPSSpectrumID::from(4679916));
    /// assert_eq!(GNPSSpectrumID::from_str("CCMSLIB00010012246").unwrap(), GNPSSpectrumID::from(10012246));
    /// assert_eq!(GNPSSpectrumID::from_str("CCMSLIB00008851197").unwrap(), GNPSSpectrumID::from(8851197));
    /// assert_eq!(GNPSSpectrumID::from_str("CCMSLIB00004751435").unwrap(), GNPSSpectrumID::from(4751435));
    /// assert_eq!(GNPSSpectrumID::from_str("CCMSLIB00000001547").unwrap(), GNPSSpectrumID::from(1547));
    /// assert_eq!(GNPSSpectrumID::from_str("CCMSLIB00010055263").unwrap(), GNPSSpectrumID::from(10055263));
    /// assert_eq!(GNPSSpectrumID::from_str("CCMSLIB00005489309").unwrap(), GNPSSpectrumID::from(5489309));
    ///
    /// ```
    ///
    /// # Raises
    /// * If the string does not start with `CCMSLIB`.
    /// * If the remainder of the string is not a valid [`usize`].
    /// * If the overall string is not of length 18.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 18 {
            return Err(format!(
                concat!(
                    "Invalid GNPS spectrum ID: {}. ",
                    "We expect the string to be of length 18, ",
                    "but it is of length {}."
                ),
                s,
                s.len()
            ));
        }

        if !s.starts_with("CCMSLIB") {
            return Err(format!(
                "Invalid GNPS spectrum ID: {}, as it does not start with CCMSLIB.",
                s
            ));
        }

        let id = s[7..].parse::<usize>().map_err(|_| {
            format!(
                "Invalid GNPS spectrum ID: {}. Cannot parse {} to an integer.",
                s,
                &s[7..]
            )
        })?;

        Ok(Self::from(id))
    }
}
