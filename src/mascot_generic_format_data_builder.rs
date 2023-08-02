use std::str::FromStr;

use crate::prelude::*;

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

impl<F> MascotGenericFormatDataBuilder<F> {
    pub fn build(self) -> Result<MascotGenericFormatData<F>, String> {
        MascotGenericFormatData::new(
            self.level.ok_or_else(|| "Could not build MascotGenericFormatData: level is missing".to_string())?,
            self.mass_divided_by_charge_ratios,
            self.fragment_intensities,
        )
    }
}

impl<F> LineParser for MascotGenericFormatDataBuilder<F>
where
    F: FromStr,
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
    /// let parser = MascotGenericFormatDataBuilder::<f64>::default();
    ///
    /// assert!(parser.can_parse_line(line));
    ///
    /// let line = "TITLE=File:";
    /// let parser = MascotGenericFormatDataBuilder::<f64>::default();
    ///
    /// assert!(!parser.can_parse_line(line));
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
    ///     let parser = MascotGenericFormatDataBuilder::<f64>::default();
    ///     
    ///     assert!(parser.can_parse_line(line));
    /// }
    ///
    /// ```
    ///
    fn can_parse_line(&self, line: &str) -> bool {
        line.starts_with("MSLEVEL=")
            || line.contains(' ') && line.split(' ').all(|s| s.parse::<F>().is_ok())
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

        let mut split = line.split(' ');

        // We obtain the mass divided by change value:
        let mass_divided_by_charge_ratio = split
            .next()
            .ok_or_else(|| "Could not parse mass divided by charge ratio".to_string())?
            .parse::<F>()
            .map_err(|_| "Could not parse mass divided by charge ratio".to_string())?;

        // We obtain the fragment intensity:
        let fragment_intensity = split
            .next()
            .ok_or_else(|| "Could not parse fragment intensity".to_string())?
            .parse::<F>()
            .map_err(|_| "Could not parse fragment intensity".to_string())?;

        // We add the values to the vectors:
        self.mass_divided_by_charge_ratios.push(mass_divided_by_charge_ratio);
        self.fragment_intensities.push(fragment_intensity);

        Ok(())
    }
}
