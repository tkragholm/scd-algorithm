//! The input and output records of the SCD definition.
//!
//! These are the *plain input boundary*: a consumer turns whatever it has —
//! LPR2/LPR3 register records, a parquet extract, a CSV, test fixtures — into
//! [`ScdDiagnosis`] values, and receives [`ScdResult`] values back. The crate
//! itself knows nothing about registers or I/O.

use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::code_patterns::Icd10Chapter;

/// A single diagnosis to be evaluated against the SCD definition.
///
/// One person typically contributes many diagnoses; [`compute_scd_results`]
/// aggregates them per person.
///
/// [`compute_scd_results`]: crate::compute_scd_results
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScdDiagnosis {
    /// Stable person identifier (e.g. a pseudonymized CPR number).
    pub person_id: String,
    /// The raw ICD-10 diagnosis code as recorded (register prefixes and
    /// punctuation are tolerated; see [`normalize_diagnosis_code`]).
    ///
    /// [`normalize_diagnosis_code`]: crate::normalize_diagnosis_code
    pub diagnosis_code: String,
    /// The date the diagnosis applies to (e.g. admission/contact date). May be
    /// absent; absent dates are counted in [`ScdStats::missing_dates`] but do
    /// not exclude the diagnosis.
    ///
    /// [`ScdStats::missing_dates`]: crate::ScdStats::missing_dates
    pub diagnosis_date: Option<NaiveDate>,
}

impl ScdDiagnosis {
    /// Convenience constructor.
    #[must_use]
    pub fn new(
        person_id: impl Into<String>,
        diagnosis_code: impl Into<String>,
        diagnosis_date: Option<NaiveDate>,
    ) -> Self {
        Self {
            person_id: person_id.into(),
            diagnosis_code: diagnosis_code.into(),
            diagnosis_date,
        }
    }
}

/// The per-person outcome of evaluating diagnoses against the SCD definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScdResult {
    /// The person this result describes.
    pub person_id: String,
    /// Whether the person has at least one severe-chronic-disease diagnosis.
    pub is_scd: bool,
    /// The earliest SCD diagnosis date, if any dated SCD diagnosis exists.
    pub first_scd_date: Option<NaiveDate>,
    /// The ICD-10 chapter of the earliest SCD diagnosis.
    ///
    /// Serialized as the chapter [label](Icd10Chapter::as_label)
    /// (e.g. `"II Neoplasms"`).
    pub first_scd_chapter: Option<Icd10Chapter>,
    /// Per-category flags: `category key -> matched`, covering every
    /// [`DiseaseCategory`](crate::DiseaseCategory). Keys come from
    /// [`ScdDiagnosisCodeSet::category_order`](crate::ScdDiagnosisCodeSet::category_order).
    pub disease_categories: HashMap<String, bool>,
}
