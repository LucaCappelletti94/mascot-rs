//! Module to handle the PubMed ID and possible associated metadata.
//! 
//! # Examples
//! A pubmed ID as provided from the MGF file may look like:
//! 
//! * `PUBMED=123456`
//! * `PUBMED=123456.0`
//! * `123456`
//! * `123456.0`
//! * `PUBMED=PMID: 9873113  doi:10.1016/S0040-4039(96)02163-6`
//! * `PMID: 9873113  doi:10.1016/S0040-4039(96)02163-6`

use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct PubMedID {
    id: usize,
    doi: Option<String>,
}

impl PubMedID {
    /// Returns the pubmed id.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Returns the doi.
    pub fn doi(&self) -> Option<&str> {
        self.doi.as_deref()
    }

    /// Create a new [`PubMedID`].
    /// 
    /// # Arguments
    /// * `id` - The pubmed id.
    /// * `doi` - The doi.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use mascot_rs::prelude::*;
    /// 
    /// let pubmed_id = PubMedID::new(123456, Some("10.1016/S0040-4039(96)02163-6")).unwrap();
    /// 
    /// assert!(pubmed_id.id() == 123456);
    /// assert!(
    ///     pubmed_id.doi().unwrap() == "10.1016/S0040-4039(96)02163-6",
    ///     concat!(
    ///         "We where expecting the doi to be ",
    ///         "10.1016/S0040-4039(96)02163-6, but got ",
    ///         "{} instead."
    ///     ),
    ///     pubmed_id.doi().unwrap()
    /// );
    /// ```
    pub fn new(id: usize, doi: Option<&str>) -> Result<Self, String> {
        if id == 0 {
            return Err("PubMed ID cannot be zero.".to_string());
        }

        if let Some(doi) = doi {
            if doi.is_empty() {
                return Err("DOI cannot be empty.".to_string());
            }
        }

        Ok(Self {
            id,
            doi: doi.map(|s| s.to_string()),
        })
    }
}

impl FromStr for PubMedID {
    type Err = String;

    /// Parses a string to a [`PubMedID`].
    ///
    /// # Arguments
    /// * `s` - The string to parse.
    ///
    /// # Examples
    /// The PubMed ID as provided from the MGF file may look like:
    /// 
    /// * `PUBMED=123456`
    /// * `PUBMED=123456.0`
    /// * `123456`
    /// * `123456.0`
    /// * `PUBMED=PMID: 9873113  doi:10.1016/S0040-4039(96)02163-6`
    /// * `PMID: 9873113  doi:10.1016/S0040-4039(96)02163-6`
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    /// use std::str::FromStr;
    /// 
    /// let pubmed_id = PubMedID::from_str("PUBMED=123456").unwrap();
    /// 
    /// assert!(pubmed_id.id() == 123456);
    /// assert!(pubmed_id.doi().is_none());
    /// 
    /// let pubmed_id = PubMedID::from_str("PUBMED=PMID: 9873113  DOI:10.1016/S0040-4039(96)02163-6").unwrap();
    /// 
    /// assert!(pubmed_id.id() == 9873113);
    /// assert!(
    ///     pubmed_id.doi().unwrap() == "10.1016/S0040-4039(96)02163-6",
    ///     concat!(
    ///         "We where expecting the doi to be ",
    ///         "10.1016/S0040-4039(96)02163-6, but got ",
    ///         "{} instead."
    ///     ),
    ///     pubmed_id.doi().unwrap()
    /// );
    /// 
    /// let pubmed_id = PubMedID::from_str("PMID: 9873113  doi:10.1016/S0040-4039(96)02163-6").unwrap();
    /// 
    /// assert!(pubmed_id.id() == 9873113);
    /// assert!(
    ///     pubmed_id.doi().unwrap() == "10.1016/S0040-4039(96)02163-6",
    ///     concat!(
    ///         "We where expecting the doi to be ",
    ///         "10.1016/S0040-4039(96)02163-6, but got ",
    ///         "{} instead."
    ///     ),
    ///     pubmed_id.doi().unwrap()
    /// );
    /// 
    /// let pubmed_id = PubMedID::from_str("123456").unwrap();
    /// 
    /// assert!(pubmed_id.id() == 123456);
    /// assert!(pubmed_id.doi().is_none());
    /// 
    /// let pubmed_id = PubMedID::from_str("123456.0").unwrap();
    /// 
    /// assert!(pubmed_id.id() == 123456);
    /// assert!(pubmed_id.doi().is_none());
    /// 
    /// let pubmed_id = PubMedID::from_str("PUBMED=123456.0").unwrap();
    /// 
    /// assert!(pubmed_id.id() == 123456);
    /// assert!(pubmed_id.doi().is_none());
    /// 
    /// ```
    ///
    fn from_str(original: &str) -> Result<Self, Self::Err> {
        let s = original.trim();
        if s.is_empty() {
            return Err("PubMed ID cannot be empty.".to_string());
        }

        let s = s.to_uppercase();
        let mut id: usize = 0;
        let mut doi: Option<&str> = None;

        // First, we normalize the string by removing the prefix if it exists.
        let s = if s.starts_with("PUBMED=") {
            &s[7..]
        } else {
            &s
        };
        
        // Secondarily, we remove the PMID prefix if it exists.
        let s = if s.starts_with("PMID:") {
            &s[5..]
        } else {
            &s
        };

        // We trim the string.
        let s = s.trim();

        // We split the string on spaces and handle the first part as the pubmed id
        // and the second part as the doi. There may be multiple white spaces, leading
        // to some empty strings.

        let parts = s.split_whitespace();

        for mut part in parts {
            // It may happen that the provided pubmed ID is not an integer
            // (e.g. "PUBMED=15386517.0"). In this case we parse the integer
            // part of the pubmed ID, by first converting the string to a float
            // and then to an integer. If that is the case, we check that the
            // remainder of the pubmed ID is ".0".

            if let Some(trimmed) = part.strip_suffix(".0"){
                part = trimmed;
            }

            // If the part is wholly numeric, we assume it is the pubmed id.
            if let Ok(part) = part.parse::<usize>() {
                if id == 0 {
                    id = part;
                } else {
                    return Err(format!("Invalid PubMed ID: {}", s));
                }
            }
            // Otherwise, if it starts with doi followed by a column,
            // we assume it is the doi.
            else if let Some(candidate_doi) = part.strip_prefix("DOI:") {
                if candidate_doi.is_empty() {
                    return Err(format!("Invalid PubMed ID: {}", s));
                }
                doi = Some(candidate_doi);
            }
        }
        
        if id == 0 {
            return Err(
                format!(
                    "Invalid PubMed ID: {}",
                    original
                )
            );
        }

        Self::new(id, doi)
    }
}