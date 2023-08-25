use std::{fmt::Debug, str::FromStr};

use crate::prelude::*;

#[derive(Debug, Clone)]
/// Struct to hold the data of a single scan in a Mascot Generic Format file.
pub struct MascotGenericFormatDataBuilder<F> {
    level: Option<FragmentationSpectraLevel>,
    mass_divided_by_charge_ratios: Vec<F>,
    fragment_intensities: Vec<F>,
}

impl<F> Default for MascotGenericFormatDataBuilder<F> {
    fn default() -> Self {
        Self {
            level: None,
            mass_divided_by_charge_ratios: Vec::new(),
            fragment_intensities: Vec::new(),
        }
    }
}

impl<F: PartialEq + PartialOrd + Copy + Debug> MascotGenericFormatDataBuilder<F> {
    pub fn build(self) -> Result<MascotGenericFormatData<F>, String> {
        MascotGenericFormatData::new(
            self.level.ok_or_else(|| {
                "Could not build MascotGenericFormatData: level is missing".to_string()
            })?,
            self.mass_divided_by_charge_ratios,
            self.fragment_intensities,
        )
    }

    /// Returns whether the level is equal to two.
    ///
    /// # Raises
    /// Raises an error if the level has not been set.
    pub fn is_level_two(&self) -> Result<bool, String> {
        match self.level {
            Some(FragmentationSpectraLevel::Two) => Ok(true),
            Some(FragmentationSpectraLevel::One) => Ok(false),
            None => Err("Could not determine whether the level is equal to two: the level has not been set.".to_string()),
        }
    }
}

impl<F> LineParser for MascotGenericFormatDataBuilder<F>
where
    F: FromStr + NaN + StrictlyPositive + PartialOrd + Debug + Copy,
{
    /// Returns whether the line can be parsed by this parser.
    ///
    /// # Arguments
    /// * `line` - The line to parse.
    ///
    /// # Returns
    /// Whether the line can be parsed by this parser.
    ///
    /// # Examples
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    ///
    /// let line = "MSLEVEL=1";
    ///
    /// assert!(MascotGenericFormatDataBuilder::<f64>::can_parse_line(line));
    ///
    /// let line = "SPECTYPE=CORRELATED MS";
    ///
    /// assert!(MascotGenericFormatDataBuilder::<f64>::can_parse_line(line));
    ///
    /// let line = "TITLE=File:";
    ///
    /// assert!(!MascotGenericFormatDataBuilder::<f64>::can_parse_line(line));
    /// 
    /// let line = "SOURCE_INSTRUMENT=ESI-LC-ESI-QFT";
    /// 
    /// assert!(!MascotGenericFormatDataBuilder::<f64>::can_parse_line(line));
    ///
    /// for line in [
    ///     "60.5425 2.4E5",
    ///     "119.0857 3.3E5",
    ///     "72.6217 2.1E4",
    ///     "79.0547 1.6E5",
    ///     "81.0606 1.1E4",
    ///     "81.0704 2.4E6",
    ///     "83.0497 1.7E4"
    /// ] {
    ///     assert!(MascotGenericFormatDataBuilder::<f64>::can_parse_line(line));
    /// }
    ///
    /// ```
    ///
    fn can_parse_line(line: &str) -> bool {
        line.starts_with("MSLEVEL=")
            || line.starts_with("SPECTYPE=CORRELATED MS")
            || line.contains(' ') && line.split(' ').all(|s| s.parse::<F>().is_ok())
    }

    /// Returns whether the builder can be built.
    fn can_build(&self) -> bool {
        self.level.is_some()
            && self.mass_divided_by_charge_ratios.len() == self.fragment_intensities.len()
            && !self.mass_divided_by_charge_ratios.is_empty()
    }

    /// Parses the line and updates the builder.
    ///
    /// # Arguments
    /// * `line` - The line to parse.
    ///
    /// # Examples
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    ///
    /// let line = "TITLE=File:";
    /// let mut parser = MascotGenericFormatDataBuilder::<f64>::default();
    ///
    /// parser.digest_line(line).is_err();
    ///
    /// let line = "MSLEVEL=1";
    /// let mut parser = MascotGenericFormatDataBuilder::<f64>::default();
    ///
    /// parser.digest_line(line).unwrap();
    /// parser.digest_line("SPECTYPE=CORRELATED MS").unwrap();
    ///
    /// let mut parser = MascotGenericFormatDataBuilder::<f64>::default();
    ///
    /// parser.digest_line("MSLEVEL=1");
    /// parser.digest_line("60.5425 2.4E5");
    /// parser.digest_line("119.0857 3.3E5");
    ///
    /// let mascot_generic_format_data = parser.build().unwrap();
    ///
    /// assert_eq!(mascot_generic_format_data.level(), FragmentationSpectraLevel::One);
    /// assert_eq!(mascot_generic_format_data.mass_divided_by_charge_ratios(), &[60.5425, 119.0857]);
    /// assert_eq!(mascot_generic_format_data.fragment_intensities(), &[2.4E5, 3.3E5]);
    ///
    /// ```
    ///
    fn digest_line(&mut self, line: &str) -> Result<(), String> {
        if line.starts_with("MSLEVEL=") {
            self.level = Some(FragmentationSpectraLevel::from_str(line)?);
            return Ok(());
        }

        // If we encounter a SPECTYPE line, the MSLEVEL must have already been parsed
        // and it must be equal to 1:
        if line.starts_with("SPECTYPE=CORRELATED MS") {
            return Ok(());
        }

        let mut split = line.split(' ');

        // We obtain the mass divided by change value:
        let mass_divided_by_charge_ratio = split
            .next()
            .ok_or_else(|| {
                format!(
                    "Could not parse mass divided by charge ratio from line \"{}\".",
                    line
                )
            })?
            .parse::<F>()
            .map_err(|_| {
                format!(
                    "Could not parse mass divided by charge ratio from line \"{}\".",
                    line
                )
            })?;

        // We obtain the fragment intensity:
        let fragment_intensity = split
            .next()
            .ok_or_else(|| "Could not parse fragment intensity".to_string())?
            .parse::<F>()
            .map_err(|_| "Could not parse fragment intensity".to_string())?;

        if mass_divided_by_charge_ratio.is_nan() {
            return Err(format!(
                concat!(
                    "The mass divided by charge ratio provided in the ",
                    "line \"{}\" was interpreted as a NaN."
                ),
                line
            ));
        }

        if !mass_divided_by_charge_ratio.is_strictly_positive() {
            return Err(format!(
                concat!(
                    "The provided line \"{}\" contains a mass divided by charge ratio ",
                    "that has been interpreted as a zero or negative value. ",
                    "The mass divided by charge ratio must be a strictly positive value."
                ),
                line
            ));
        }

        if fragment_intensity.is_nan() {
            return Err(format!(
                concat!(
                    "The fragment intensity provided in the ",
                    "line \"{}\" was interpreted as a NaN."
                ),
                line
            ));
        }

        if !fragment_intensity.is_strictly_positive() {
            return Err(format!(
                concat!(
                    "The provided line \"{}\" contains a fragment intensity ",
                    "that has been interpreted as a zero or negative value. ",
                    "The fragment intensity must be a strictly positive value."
                ),
                line
            ));
        }

        // We check that the value of the mass divided by charge ratio is larger
        // or equal to the previous value:
        if let Some(previous_mass_divided_by_charge_ratio) =
            self.mass_divided_by_charge_ratios.last()
        {
            if self.is_level_two()?
                && *previous_mass_divided_by_charge_ratio > mass_divided_by_charge_ratio
            {
                return Err(format!(
                    concat!(
                        "The mass divided by charge ratio provided in the ",
                        "line \"{}\" was smaller than the previous value. ",
                        "The mass divided by charge ratio must be provided in ",
                        "ascending order. The current value is {:?}, while the ",
                        "previous value was {:?}."
                    ),
                    line, mass_divided_by_charge_ratio, previous_mass_divided_by_charge_ratio
                ));
            }
        }

        // We add the values to the vectors:
        self.mass_divided_by_charge_ratios
            .push(mass_divided_by_charge_ratio);
        self.fragment_intensities.push(fragment_intensity);

        Ok(())
    }
}
