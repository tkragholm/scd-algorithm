//! # `scd` — Severe Chronic Disease definition
//!
//! The single, canonical definition of how our research group identifies
//! **severe chronic disease (SCD)** from ICD-10 diagnosis codes. It implements
//! the published Danish paediatric SCD code list (Kristensen et al. 2012; Jensen
//! et al. 2019, 2022; see `REFERENCES.md`), kept small and dependency-light so it
//! can be shared, unchanged, across every project.
//!
//! ## What it does
//!
//! Given a flat list of [`ScdDiagnosis`] records (`person_id`, ICD-10 code,
//! optional date), [`compute_scd_results`] returns one [`ScdResult`] per
//! SCD-positive person: the earliest qualifying diagnosis date and its ICD-10
//! chapter, and which [`DiseaseCategory`] groups they match.
//!
//! ## The definition lives in two places
//!
//! * [`code_sets`] — the SCD predicate ([`ScdDiagnosisCodeSet::category_of`]),
//!   the inclusion criteria themselves. **This is the cohort definition.**
//! * [`code_patterns`] — code normalization and ICD-10 chapter classification.
//!
//! ## Plain input boundary
//!
//! This crate knows nothing about data sources. Callers map their own data
//! (LPR2/LPR3 register records, parquet, CSV, fixtures, …) into [`ScdDiagnosis`]
//! and call [`compute_scd_results`].
//!
//! ## Example
//!
//! Only SCD-positive people appear in the results — a person with no qualifying
//! diagnosis produces no row.
//!
//! ```
//! use chrono::NaiveDate;
//! use scd::{ScdDiagnosis, compute_scd_results};
//!
//! let date = |y, m, d| NaiveDate::from_ymd_opt(y, m, d);
//! let diagnoses = vec![
//!     ScdDiagnosis::new("p1", "DE10", date(2015, 3, 1)),  // type 1 diabetes (SCD)
//!     ScdDiagnosis::new("p1", "DJ45", date(2014, 1, 1)),  // asthma (NOT SCD)
//!     ScdDiagnosis::new("p2", "DZ000", date(2015, 1, 1)), // checkup (NOT SCD)
//! ];
//!
//! let (results, stats) = compute_scd_results(&diagnoses);
//!
//! // p1 qualifies via diabetes; p2 has no SCD diagnosis and is absent.
//! assert_eq!(results.len(), 1);
//! assert_eq!(results[0].person_id, "p1");
//! assert!(results[0].is_scd);
//! assert_eq!(results[0].first_scd_date, date(2015, 3, 1));
//!
//! assert!(results.iter().all(|r| r.person_id != "p2"));
//! assert_eq!(stats.unmapped_codes, 2); // the asthma and the checkup
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod code_patterns;
pub mod code_sets;
pub mod model;

use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

pub use code_patterns::{
    Icd10Chapter, NormalizedDiagnosis, UnknownChapter, icd10_chapter_label,
    normalize_diagnosis_code,
};
pub use code_sets::{DiseaseCategory, ScdDiagnosisCodeSet};
pub use model::{ScdDiagnosis, ScdResult};

/// Counts of records skipped or flagged while computing SCD results.
///
/// These are quality-control signals, not errors: the computation always
/// succeeds, and these counts describe the input.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScdStats {
    /// Diagnosis rows with a blank/whitespace-only code.
    pub missing_diagnoses: usize,
    /// SCD diagnosis rows that had no date.
    pub missing_dates: usize,
    /// Diagnosis rows whose code did not match the SCD definition.
    pub unmapped_codes: usize,
}

/// Aggregated per-person state built up while scanning diagnoses.
#[derive(Default, Clone)]
struct PatientData {
    first_scd_date: Option<NaiveDate>,
    first_scd_chapter: Option<Icd10Chapter>,
    categories_mask: u16,
}

impl PatientData {
    fn update_from_scd(
        &mut self,
        diagnosis_date: Option<NaiveDate>,
        category: DiseaseCategory,
        chapter: Option<Icd10Chapter>,
    ) {
        if let Some(date) = diagnosis_date {
            match self.first_scd_date {
                Some(current_date) if date < current_date => {
                    self.first_scd_date = Some(date);
                    self.first_scd_chapter = chapter;
                }
                Some(current_date) if date == current_date && self.first_scd_chapter.is_none() => {
                    self.first_scd_chapter = chapter;
                }
                None => {
                    self.first_scd_date = Some(date);
                    self.first_scd_chapter = chapter;
                }
                _ => {}
            }
        } else if self.first_scd_chapter.is_none() {
            self.first_scd_chapter = chapter;
        }

        self.categories_mask |= category_bit(category);
    }
}

/// Classify a batch of diagnoses and aggregate them into one [`ScdResult`] per
/// SCD-positive person.
///
/// Only people with at least one qualifying diagnosis appear in the output, so
/// every returned [`ScdResult`] has `is_scd == true`. People whose diagnoses are
/// all non-SCD produce no row. Each result records the earliest qualifying date
/// (and its ICD-10 chapter) and the union of matched disease categories.
/// Results are sorted by `person_id`.
///
/// The accompanying [`ScdStats`] summarize input quality (blank codes, undated
/// SCD diagnoses, and non-SCD codes) across *all* input rows.
#[must_use]
pub fn compute_scd_results(diagnoses: &[ScdDiagnosis]) -> (Vec<ScdResult>, ScdStats) {
    let code_set = ScdDiagnosisCodeSet::new();

    let mut stats = ScdStats::default();
    let mut patients: HashMap<String, PatientData> = HashMap::new();
    // Classification is pure, so memoize it per distinct raw code: a national
    // register repeats the same codes millions of times. `None` means "not SCD".
    let mut classified: HashMap<String, Option<(DiseaseCategory, Option<Icd10Chapter>)>> =
        HashMap::new();

    for diagnosis in diagnoses {
        let code = diagnosis.diagnosis_code.as_str();
        if code.trim().is_empty() {
            stats.missing_diagnoses += 1;
            continue;
        }

        // Look the code up by borrow and only allocate a key string on a cache
        // miss, so a repeated code (the common case at register scale) costs a
        // hash probe rather than a `String` clone on every row.
        let classification = if let Some(&cached) = classified.get(code) {
            cached
        } else {
            let computed = code_set
                .category_of(code)
                .map(|category| (category, Icd10Chapter::from_diagnosis_code(code)));
            classified.insert(code.to_string(), computed);
            computed
        };

        let Some((category, chapter)) = classification else {
            stats.unmapped_codes += 1;
            continue;
        };

        if diagnosis.diagnosis_date.is_none() {
            stats.missing_dates += 1;
        }

        patients
            .entry(diagnosis.person_id.clone())
            .or_default()
            .update_from_scd(diagnosis.diagnosis_date, category, chapter);
    }

    let mut results = patients
        .into_iter()
        .map(|(person_id, data)| ScdResult {
            person_id,
            is_scd: true,
            first_scd_date: data.first_scd_date,
            first_scd_chapter: data.first_scd_chapter,
            disease_categories: build_category_map(data.categories_mask),
        })
        .collect::<Vec<_>>();
    results.sort_by(|a, b| a.person_id.cmp(&b.person_id));

    (results, stats)
}

/// Bit position of a category within [`DiseaseCategory::ALL`].
fn category_bit(category: DiseaseCategory) -> u16 {
    let index = DiseaseCategory::ALL
        .iter()
        .position(|candidate| *candidate == category)
        .expect("every DiseaseCategory is listed in DiseaseCategory::ALL");
    1u16 << index
}

fn build_category_map(categories_mask: u16) -> HashMap<String, bool> {
    DiseaseCategory::ALL
        .iter()
        .enumerate()
        .map(|(index, category)| {
            (
                category.as_str().to_string(),
                categories_mask & (1u16 << index) != 0,
            )
        })
        .collect()
}

/// Tally the diagnosis codes that did **not** match the SCD definition.
///
/// Returns `(code, count)` pairs sorted by descending count, then by code. A
/// diagnostic aid for spotting codes that perhaps *should* be in the definition,
/// or data-quality problems.
#[must_use]
pub fn count_unmapped_scd_codes(diagnoses: &[ScdDiagnosis]) -> Vec<(String, usize)> {
    let code_set = ScdDiagnosisCodeSet::new();
    let mut counts: HashMap<String, usize> = HashMap::new();
    for diagnosis in diagnoses {
        let code = diagnosis.diagnosis_code.trim();
        if code.is_empty() {
            continue;
        }
        if !code_set.is_scd_code(code) {
            *counts.entry(code.to_string()).or_default() += 1;
        }
    }
    let mut entries = counts.into_iter().collect::<Vec<_>>();
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn date(y: i32, m: u32, d: u32) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(y, m, d)
    }

    #[test]
    fn aggregates_per_person_and_picks_earliest_date() {
        let diagnoses = vec![
            ScdDiagnosis::new("p1", "DC509", date(2016, 5, 1)), // neoplasm
            ScdDiagnosis::new("p1", "DE10", date(2014, 2, 3)),  // diabetes, earlier
            ScdDiagnosis::new("p1", "DJ45", date(2013, 1, 1)),  // asthma, NOT SCD
        ];
        let (results, _) = compute_scd_results(&diagnoses);
        assert_eq!(results.len(), 1);
        let p1 = &results[0];
        assert!(p1.is_scd);
        assert_eq!(p1.first_scd_date, date(2014, 2, 3));
        assert_eq!(p1.first_scd_chapter, Some(Icd10Chapter::Endocrine));
        assert!(p1.disease_categories[DiseaseCategory::Metabolic.as_str()]);
        assert!(p1.disease_categories[DiseaseCategory::Neoplasms.as_str()]);
        assert!(!p1.disease_categories[DiseaseCategory::Renal.as_str()]);
    }

    #[test]
    fn non_scd_person_is_reported_negative() {
        let diagnoses = vec![ScdDiagnosis::new("p1", "DZ000", date(2015, 1, 1))];
        let (results, stats) = compute_scd_results(&diagnoses);
        assert_eq!(
            results.len(),
            0,
            "no row is created for non-SCD-only people"
        );
        assert_eq!(stats.unmapped_codes, 1);
    }

    #[test]
    fn stats_count_blank_and_undated_and_unmapped() {
        let diagnoses = vec![
            ScdDiagnosis::new("p1", "   ", date(2015, 1, 1)), // blank
            ScdDiagnosis::new("p1", "DE10", None),            // SCD, undated
            ScdDiagnosis::new("p1", "DJ45", date(2015, 1, 1)), // not SCD
        ];
        let (_, stats) = compute_scd_results(&diagnoses);
        assert_eq!(stats.missing_diagnoses, 1);
        assert_eq!(stats.missing_dates, 1);
        assert_eq!(stats.unmapped_codes, 1);
    }

    #[test]
    fn results_are_sorted_by_person_id() {
        let diagnoses = vec![
            ScdDiagnosis::new("p3", "DE10", date(2015, 1, 1)),
            ScdDiagnosis::new("p1", "DE10", date(2015, 1, 1)),
            ScdDiagnosis::new("p2", "DE10", date(2015, 1, 1)),
        ];
        let (results, _) = compute_scd_results(&diagnoses);
        let ids: Vec<&str> = results.iter().map(|r| r.person_id.as_str()).collect();
        assert_eq!(ids, ["p1", "p2", "p3"]);
    }

    #[test]
    fn scd_result_serde_round_trip() {
        let diagnoses = vec![ScdDiagnosis::new("p1", "DC509", date(2015, 1, 1))];
        let (results, _) = compute_scd_results(&diagnoses);
        let json = serde_json::to_string(&results[0]).unwrap();
        assert!(
            json.contains("II Neoplasms"),
            "chapter serialized as label: {json}"
        );
        let back: ScdResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back, results[0]);
    }

    #[test]
    fn unmapped_codes_are_tallied_by_frequency() {
        let diagnoses = vec![
            ScdDiagnosis::new("p1", "DJ45", date(2015, 1, 1)),
            ScdDiagnosis::new("p2", "DJ45", date(2015, 1, 1)),
            ScdDiagnosis::new("p3", "DZ000", date(2015, 1, 1)),
            ScdDiagnosis::new("p4", "DE10", date(2015, 1, 1)), // SCD, excluded
        ];
        let unmapped = count_unmapped_scd_codes(&diagnoses);
        assert_eq!(
            unmapped,
            vec![("DJ45".to_string(), 2), ("DZ000".to_string(), 1)]
        );
    }
}
