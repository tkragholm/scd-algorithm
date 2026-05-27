//! End-to-end checks of the SCD definition through the public API.
//!
//! These pin the *behaviour* a consumer depends on; they should fail loudly if
//! the definition is ever changed by accident. The definition implements the
//! published Danish paediatric SCD code list (see `REFERENCES.md`).

use chrono::NaiveDate;
use scd::{DiseaseCategory, Icd10Chapter, ScdDiagnosis, ScdDiagnosisCodeSet, compute_scd_results};

const fn date(y: i32, m: u32, d: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(y, m, d)
}

#[test]
fn a_child_with_cancer_then_diabetes_is_classified_from_earliest() {
    let diagnoses = vec![
        ScdDiagnosis::new("child", "DE10", date(2017, 6, 1)), // type 1 diabetes
        ScdDiagnosis::new("child", "DC910", date(2015, 9, 9)), // leukaemia, recorded earlier
        ScdDiagnosis::new("child", "DJ06", date(2014, 1, 1)), // URTI, not SCD
        ScdDiagnosis::new("child", "DJ45", date(2013, 1, 1)), // asthma, NOT SCD under this definition
    ];

    let (results, stats) = compute_scd_results(&diagnoses);

    assert_eq!(results.len(), 1);
    let child = &results[0];
    assert!(child.is_scd);
    assert_eq!(child.first_scd_date, date(2015, 9, 9));
    assert_eq!(child.first_scd_chapter, Some(Icd10Chapter::Neoplasms));
    assert!(child.disease_categories[DiseaseCategory::Neoplasms.as_str()]);
    assert!(child.disease_categories[DiseaseCategory::Metabolic.as_str()]);
    assert_eq!(stats.unmapped_codes, 2); // the URTI and the asthma
}

#[test]
fn every_disease_category_is_reachable() {
    let codes = ScdDiagnosisCodeSet::new();
    let reps = [
        (DiseaseCategory::Neoplasms, "DC509"),
        (DiseaseCategory::Blood, "DD610"),
        (DiseaseCategory::Immunodeficiency, "DD80"),
        (DiseaseCategory::Metabolic, "DE10"),
        (DiseaseCategory::Neurological, "DG40"),
        (DiseaseCategory::Cardiac, "DI27"),
        (DiseaseCategory::Respiratory, "DJ84"),
        (DiseaseCategory::Gastrointestinal, "DK50"),
        (DiseaseCategory::ConnectiveTissue, "DM30"),
        (DiseaseCategory::Renal, "DN18"),
        (DiseaseCategory::Perinatal, "DP27"),
        (DiseaseCategory::Congenital, "DQ20"),
    ];
    for (category, code) in reps {
        assert_eq!(
            codes.category_of(code),
            Some(category),
            "{code} should classify as {category:?}"
        );
    }
}

#[test]
fn asthma_is_not_scd() {
    // Guards against the previously-drifted definition that wrongly counted the
    // whole asthma/COPD respiratory block.
    let codes = ScdDiagnosisCodeSet::new();
    assert!(!codes.is_scd_code("DJ45")); // asthma
    assert!(!codes.is_scd_code("DJ44")); // COPD (only the specific J448 qualifies)
    assert!(codes.is_scd_code("DJ448")); // ... and J448 does
}

/// Pins the six code boundaries where this implementation differs from the
/// published Jensen 2019 Table B. See `REFERENCES.md`. These assert the
/// *current* behaviour, so any future reconciliation with the publication is a
/// deliberate, visible change.
#[test]
fn boundary_cases_differ_from_published_table_b() {
    let codes = ScdDiagnosisCodeSet::new();

    // Broader than the published Table B here:
    assert!(
        codes.is_scd_code("DC97"),
        "C97 included (Table B stops at C96.9)"
    );
    assert!(
        codes.is_scd_code("DI270"),
        "I27.0 included (Table B starts at I27.1)"
    );
    assert!(
        codes.is_scd_code("DI529"),
        "I52.9 included (Table B stops at I52.8)"
    );
    assert!(
        codes.is_scd_code("DK779"),
        "K77.9 included (Table B stops at K77.8)"
    );
    assert!(
        codes.is_scd_code("DQ330"),
        "Q33.x included (Table B stops at Q32.4)"
    );
    assert!(
        codes.is_scd_code("DQ325"),
        "Q32.5 included (Table B stops at Q32.4)"
    );

    // Narrower than the published Table B here:
    assert!(
        !codes.is_scd_code("DP940"),
        "P94.0 omitted (Table B starts at P94.0)"
    );
    assert!(codes.is_scd_code("DP941"), "P94.1 included");
}

#[test]
fn empty_input_yields_empty_results() {
    let (results, stats) = compute_scd_results(&[]);
    assert!(results.is_empty());
    assert_eq!(stats, scd::ScdStats::default());
}
