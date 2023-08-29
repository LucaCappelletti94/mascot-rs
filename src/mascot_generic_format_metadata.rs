use std::{fmt::Debug, ops::Add};

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct MascotGenericFormatMetadata<I, F> {
    feature_id: I,
    parent_ion_mass: F,
    retention_time: Option<F>,
    charge: Charge,
    ion_mode: Option<IonMode>,
    sequence: Option<String>,
    source_instrument: Option<String>,
    organism: Option<String>,
    name: Option<String>,
    smiles: Option<String>,
    pubmed_id: Option<PubMedID>,
    merged_scans_metadata: Option<MergeScansMetadata<I>>,
}

impl<I: Copy + Add<Output = I> + Eq + Debug + Copy + Zero, F: StrictlyPositive + Copy>
    MascotGenericFormatMetadata<I, F>
{
    /// Creates a new [`MascotGenericFormatMetadata`].
    ///
    /// # Arguments
    /// * `feature_id` - The feature ID of the metadata.
    /// * `parent_ion_mass` - The parent ion mass of the metadata.
    /// * `retention_time` - The retention time of the metadata.
    /// * `source_instrument` - The source instrument of the metadata.
    /// * `organism` - The organism of the metadata.
    /// * `name` - The name of the metadata.
    /// * `smiles` - The smiles of the metadata.
    /// * `sequence` - The sequence of the metadata.
    /// * `charge` - The charge of the metadata.
    /// * `ion_mode` - The ion mode of the metadata.
    /// * `pubmed_id` - The pubmed ID of the metadata.
    /// * `merged_scans_metadata` - The merged scans metadata of the metadata.
    ///
    /// # Returns
    /// A new [`MascotGenericFormatMetadata`].
    ///
    /// # Errors
    /// * If `parent_ion_mass` is not strictly positive.
    /// * If `retention_time` is not strictly positive.
    ///
    /// # Examples
    ///
    /// ```
    /// use mascot_rs::prelude::*;
    ///
    /// let feature_id = 1;
    /// let parent_ion_mass = 381.0795;
    /// let retention_time = 37.083;
    /// let source_instrument = Some("ESI-QUAD-TOF".to_string());
    /// let name = Some("GNPS-COLLECTIONS-PESTICIDES-POSITIVE".to_string());
    /// let smiles = Some("CC(C)C[C@@H](C(=O)O)NC(=O)[C@H](CC1=CC=CC=C1)N".to_string());
    /// let organism = Some("ORGANISM=GNPS-COLLECTIONS-PESTICIDES-POSITIVE".to_string());
    /// let sequence = Some("K.LLQLELGGQSLPELQK.V".to_string());
    /// let charge = Charge::One;
    /// let pubmed_id = Some(PubMedID::new(15386517, None).unwrap());
    /// let ion_mode = Some(IonMode::Positive);
    ///
    /// let mascot_generic_format_metadata: MascotGenericFormatMetadata<usize, f64> = MascotGenericFormatMetadata::new(
    ///     feature_id,
    ///     parent_ion_mass,
    ///     Some(retention_time),
    ///     source_instrument,
    ///     sequence.clone(),
    ///     organism.clone(),
    ///     name.clone(),
    ///     smiles.clone(),
    ///     charge,
    ///     ion_mode,
    ///     pubmed_id.clone(),
    ///     None,
    /// ).unwrap();
    ///
    /// assert_eq!(mascot_generic_format_metadata.feature_id(), feature_id);
    /// assert_eq!(mascot_generic_format_metadata.parent_ion_mass(), parent_ion_mass);
    /// assert_eq!(mascot_generic_format_metadata.retention_time(), Some(retention_time));
    /// assert_eq!(mascot_generic_format_metadata.charge(), charge);
    ///
    /// assert!(
    ///     MascotGenericFormatMetadata::new(
    ///         feature_id,
    ///         -1.0,
    ///         Some(retention_time),
    ///         None,
    ///         sequence.clone(),
    ///         organism.clone(),
    ///         name.clone(),
    ///         smiles.clone(),
    ///         charge,
    ///         ion_mode,
    ///         pubmed_id.clone(),
    ///         None
    ///     ).is_err()
    /// );
    ///
    /// assert!(
    ///     MascotGenericFormatMetadata::new(
    ///         feature_id,
    ///         parent_ion_mass,
    ///         Some(-1.0),
    ///         None,
    ///         sequence.clone(),
    ///         organism.clone(),
    ///         name.clone(),
    ///         smiles.clone(),
    ///         charge,
    ///         ion_mode,
    ///         pubmed_id.clone(),
    ///         None
    ///     ).is_err()
    /// );
    ///
    /// ```
    ///
    pub fn new(
        feature_id: I,
        parent_ion_mass: F,
        retention_time: Option<F>,
        source_instrument: Option<String>,
        sequence: Option<String>,
        organism: Option<String>,
        name: Option<String>,
        smiles: Option<String>,
        charge: Charge,
        ion_mode: Option<IonMode>,
        pubmed_id: Option<PubMedID>,
        merged_scans_metadata: Option<MergeScansMetadata<I>>,
    ) -> Result<Self, String> {
        if !parent_ion_mass.is_strictly_positive() {
            return Err("Could not create MascotGenericFormatMetadata: parent_ion_mass must be strictly positive".to_string());
        }

        if let Some(retention_time) = retention_time.as_ref() {
            if !retention_time.is_strictly_positive() {
                return Err("Could not create MascotGenericFormatMetadata: retention_time must be strictly positive".to_string());
            }
        }

        // If the source instrument is provided, it cannot be
        // an empty string.
        if let Some(source_instrument) = source_instrument.as_ref() {
            if source_instrument.is_empty() {
                return Err("Could not create MascotGenericFormatMetadata: source_instrument cannot be an empty string".to_string());
            }
        }

        // If the sequence is provided, it cannot be
        // an empty string.
        if let Some(sequence) = sequence.as_ref() {
            if sequence.is_empty() {
                return Err("Could not create MascotGenericFormatMetadata: sequence cannot be an empty string".to_string());
            }
        }

        // If the organism is provided, it cannot be
        // an empty string.
        if let Some(organism) = organism.as_ref() {
            if organism.is_empty() {
                return Err("Could not create MascotGenericFormatMetadata: organism cannot be an empty string".to_string());
            }
        }

        // If the name is provided, it cannot be
        // an empty string.
        if let Some(name) = name.as_ref() {
            if name.is_empty() {
                return Err("Could not create MascotGenericFormatMetadata: name cannot be an empty string".to_string());
            }
        }

        // If the smiles is provided, it cannot be
        // an empty string or "N/A".
        if let Some(smiles) = smiles.as_ref() {
            if smiles.is_empty() || smiles == "N/A" {
                return Err("Could not create MascotGenericFormatMetadata: smiles cannot be an empty string or \"N/A\"".to_string());
            }
        }

        Ok(Self {
            feature_id,
            parent_ion_mass,
            retention_time,
            source_instrument,
            sequence,
            organism,
            charge,
            ion_mode,
            name,
            smiles,
            pubmed_id,
            merged_scans_metadata,
        })
    }

    /// Returns the feature ID of the metadata.
    pub fn feature_id(&self) -> I {
        self.feature_id
    }

    /// Returns the parent ion mass of the metadata.
    pub fn parent_ion_mass(&self) -> F {
        self.parent_ion_mass
    }

    /// Returns the retention time of the metadata.
    pub fn retention_time(&self) -> Option<F> {
        self.retention_time
    }

    /// Returns the charge of the metadata.
    pub fn charge(&self) -> Charge {
        self.charge
    }

    /// Returns the number of scans removed due to low quality.
    pub fn number_of_scans_removed_due_to_low_quality(&self) -> I {
        self.merged_scans_metadata
            .as_ref()
            .map(|m| m.removed_due_to_low_quality())
            .unwrap_or(I::ZERO)
    }
}
