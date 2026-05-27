//! **The severe chronic disease (SCD) classification.**
//!
//! [`ScdDiagnosisCodeSet::category_of`] is the operational definition of severe
//! chronic disease used here. It upper-cases the diagnosis code, removes the
//! leading Danish register `D`, and tests the resulting 3-character ICD-10
//! category (`d3`) and 4-character subcategory (`d4`) against the inclusion
//! criteria below. Each criterion is annotated with the clinical condition and
//! ICD-10 range it captures, in ICD-10 chapter order.
//!
//! The criteria implement the published Danish paediatric SCD code list
//! (Kristensen et al. 2012; Jensen et al. 2019, 2022), with a small number of
//! deliberate, clearly-marked modifications — currently the inclusion of `P07`.
//! `REFERENCES.md` records the source publications, the published code list, and
//! every difference between this implementation and that list; each modification
//! is pinned by a dedicated test.
//!
//! The criteria are mutually exclusive by ICD-10 chapter, so every SCD code maps
//! to exactly one [`DiseaseCategory`]; the categories follow the clinical
//! groupings of the published list, aligned to ICD-10 chapters.

use crate::code_patterns::{NormalizedDiagnosis, normalize_diagnosis_code};

/// The clinical disease categories of the SCD definition, following the
/// groupings of the published SCD code list (see `REFERENCES.md`).
///
/// Each category is aligned to an ICD-10 chapter. Two conditions sit in a
/// chapter that differs from their clinical description and are grouped by
/// chapter: hypertensive renal disease (`I12`) is grouped under [`Cardiac`] and
/// perinatal chronic respiratory disease (`P27`) under [`Perinatal`].
///
/// [`Cardiac`]: DiseaseCategory::Cardiac
/// [`Perinatal`]: DiseaseCategory::Perinatal
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiseaseCategory {
    /// Malignant neoplasms — `C00`–`C99`.
    Neoplasms,
    /// Aplastic anaemias and haemophagocytic syndrome — `D610/3/8/9`, `D762`.
    Blood,
    /// Immunodeficiencies — `D80`–`D82`.
    Immunodeficiency,
    /// Endocrine and metabolic disorders, incl. cystic fibrosis — selected `E`.
    Metabolic,
    /// Chronic neurological disorders — selected `G`.
    Neurological,
    /// Chronic heart disease — `I12`, `I27`, `I30`–`I52`.
    Cardiac,
    /// Chronic respiratory disease — `J448`, `J84`.
    Respiratory,
    /// Chronic gastrointestinal and liver disease — `K21`, `K50`/`K51`, `K70`–`K77`, `K90`.
    Gastrointestinal,
    /// Systemic connective-tissue disease — `M30`–`M35`.
    ConnectiveTissue,
    /// Chronic kidney disease and related nephropathy — selected `N`.
    Renal,
    /// Chronic conditions originating in the perinatal period — selected `P`.
    Perinatal,
    /// Congenital malformations and chromosomal abnormalities — selected `Q`.
    Congenital,
}

impl DiseaseCategory {
    /// Every category, in canonical order (also the order of
    /// [`category bits`](crate::ScdResult::disease_categories)).
    pub const ALL: [Self; 12] = [
        Self::Neoplasms,
        Self::Blood,
        Self::Immunodeficiency,
        Self::Metabolic,
        Self::Neurological,
        Self::Cardiac,
        Self::Respiratory,
        Self::Gastrointestinal,
        Self::ConnectiveTissue,
        Self::Renal,
        Self::Perinatal,
        Self::Congenital,
    ];

    /// The stable string key for this category (used in
    /// [`ScdResult::disease_categories`](crate::ScdResult::disease_categories)).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Neoplasms => "neoplasms",
            Self::Blood => "blood_disorders",
            Self::Immunodeficiency => "immunodeficiency",
            Self::Metabolic => "endocrine_metabolic",
            Self::Neurological => "neurological",
            Self::Cardiac => "cardiac",
            Self::Respiratory => "respiratory",
            Self::Gastrointestinal => "gastrointestinal",
            Self::ConnectiveTissue => "connective_tissue",
            Self::Renal => "renal",
            Self::Perinatal => "perinatal",
            Self::Congenital => "congenital",
        }
    }
}

/// The SCD definition: classifies ICD-10 codes against the published code list.
///
/// Classification is pure and stateless; this is a thin handle kept for a stable
/// API and is `Send + Sync`.
///
/// ```
/// use scd::{DiseaseCategory, ScdDiagnosisCodeSet};
///
/// let codes = ScdDiagnosisCodeSet::new();
/// assert_eq!(codes.category_of("DC910"), Some(DiseaseCategory::Neoplasms)); // leukaemia
/// assert_eq!(codes.category_of("DE10"), Some(DiseaseCategory::Metabolic));  // type 1 diabetes
/// assert_eq!(codes.category_of("DJ45"), None);                              // asthma is NOT SCD
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ScdDiagnosisCodeSet;

impl ScdDiagnosisCodeSet {
    /// Construct the SCD code set.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// The category keys in canonical order.
    #[must_use]
    pub fn category_order(&self) -> Vec<String> {
        DiseaseCategory::ALL
            .iter()
            .map(|category| category.as_str().to_string())
            .collect()
    }

    /// Alias of [`category_order`](Self::category_order).
    #[must_use]
    pub fn categories(&self) -> Vec<String> {
        self.category_order()
    }

    /// Returns `true` if `code` meets the SCD definition.
    #[must_use]
    pub fn is_scd_code(&self, code: &str) -> bool {
        self.category_of(code).is_some()
    }

    /// Classify `code` into its [`DiseaseCategory`], or [`None`] if it is not an
    /// SCD code.
    #[must_use]
    pub fn category_of(&self, code: &str) -> Option<DiseaseCategory> {
        normalize_diagnosis_code(code).and_then(|normalized| category_of_normalized(&normalized))
    }
}

/// The SCD inclusion predicate.
///
/// The SCD criteria partition by ICD-10 chapter: the leading code letter selects
/// at most one [`DiseaseCategory`] group, each implemented by a small helper
/// below that returns `Some` when the code meets that group's criteria. `d3` is
/// the 3-character ICD-10 category code and `d4` the 4-character subcategory (see
/// [`NormalizedDiagnosis`]); every group is annotated with the clinical
/// conditions and ICD-10 ranges it captures (see `REFERENCES.md`).
fn category_of_normalized(normalized: &NormalizedDiagnosis) -> Option<DiseaseCategory> {
    let d3 = normalized.category3();
    let d4 = normalized.subcategory4();

    // Dispatch on the ICD-10 chapter letter. `D` covers both the blood and the
    // immune-disorder criteria, which share that chapter.
    match d3.bytes().next()? {
        b'C' => neoplasm(d3),
        b'D' => blood(d4).or_else(|| immunodeficiency(d3)),
        b'E' => metabolic(d3, d4),
        b'G' => neurological(d3, d4),
        b'I' => cardiac(d3),
        b'J' => respiratory(d3, d4),
        b'K' => gastrointestinal(d3),
        b'M' => connective_tissue(d3),
        b'N' => renal(d3),
        b'P' => perinatal(d3, d4),
        b'Q' => congenital(d3, d4),
        _ => None,
    }
}

/// Malignant neoplasms — `C00`–`C99`.
fn neoplasm(d3: &str) -> Option<DiseaseCategory> {
    ("C00"..="C99")
        .contains(&d3)
        .then_some(DiseaseCategory::Neoplasms)
}

/// Aplastic anaemias (`D610/3/8/9`) and infection-associated haemophagocytic
/// syndrome (`D762`).
fn blood(d4: &str) -> Option<DiseaseCategory> {
    let hit = matches!(d4, "D610" | "D613" | "D618" | "D619") || d4 == "D762";
    hit.then_some(DiseaseCategory::Blood)
}

/// Immunodeficiencies — `D80`–`D82`.
fn immunodeficiency(d3: &str) -> Option<DiseaseCategory> {
    matches!(d3, "D80" | "D81" | "D82").then_some(DiseaseCategory::Immunodeficiency)
}

/// Endocrine and metabolic disorders, including cystic fibrosis.
fn metabolic(d3: &str, d4: &str) -> Option<DiseaseCategory> {
    let hit = d3 == "E10" // insulin-dependent diabetes mellitus
        || d3 == "E25" // adrenogenital disorders
        || matches!(d3, "E70" | "E71" | "E72") || d4 == "E730" // amino-acid metabolism
        || ("E74"..="E84").contains(&d3); // metabolism disorders incl. cystic fibrosis
    hit.then_some(DiseaseCategory::Metabolic)
}

/// Chronic neurological disorders (selected chapter VI codes).
fn neurological(d3: &str, d4: &str) -> Option<DiseaseCategory> {
    let hit = d3 == "G12" // spinal muscular atrophy
        || matches!(d4, "G310" | "G318" | "G319") || d3 == "G37" // degenerative/demyelinating
        || d3 == "G40" // epilepsy
        || d3 == "G60" // hereditary/idiopathic neuropathy
        || d4 == "G702" // congenital/developmental myasthenia
        || matches!(d4, "G710" | "G711" | "G712" | "G713") // mitochondrial myopathy
        || d4 == "G736" // myopathy in metabolic diseases
        || d3 == "G80" // cerebral palsy
        || matches!(d4, "G811" | "G821" | "G824") // spastic conditions
        || d3 == "G91" // hydrocephalus
        || d4 == "G941"; // hydrocephalus in neoplastic disease
    hit.then_some(DiseaseCategory::Neurological)
}

/// Chronic heart disease — `I12`, `I27`, `I30`–`I52`.
fn cardiac(d3: &str) -> Option<DiseaseCategory> {
    let hit = d3 == "I12" // hypertensive renal disease
        || d3 == "I27" // pulmonary heart disease
        || ("I30"..="I52").contains(&d3); // other forms of heart disease
    hit.then_some(DiseaseCategory::Cardiac)
}

/// Chronic respiratory disease — `J448`, `J84`.
fn respiratory(d3: &str, d4: &str) -> Option<DiseaseCategory> {
    let hit = d4 == "J448" // other specified COPD
        || d3 == "J84"; // other interstitial pulmonary disease
    hit.then_some(DiseaseCategory::Respiratory)
}

/// Chronic gastrointestinal and liver disease.
fn gastrointestinal(d3: &str) -> Option<DiseaseCategory> {
    let hit = d3 == "K21" // gastro-oesophageal reflux disease
        || matches!(d3, "K50" | "K51") // Crohn disease and ulcerative colitis
        || matches!(d3, "K70" | "K71" | "K72" | "K73" | "K74" | "K75" | "K76" | "K77") // diseases of liver
        || d3 == "K90"; // intestinal malabsorption
    hit.then_some(DiseaseCategory::Gastrointestinal)
}

/// Systemic connective-tissue disease — `M30`–`M35`.
fn connective_tissue(d3: &str) -> Option<DiseaseCategory> {
    matches!(d3, "M30" | "M31" | "M32" | "M33" | "M34" | "M35")
        .then_some(DiseaseCategory::ConnectiveTissue)
}

/// Chronic kidney disease and related nephropathy (selected chapter XIV codes).
fn renal(d3: &str) -> Option<DiseaseCategory> {
    let hit = matches!(d3, "N03" | "N04" | "N05") // nephritic syndrome
        || d3 == "N07" // hereditary nephropathy, not elsewhere classified
        || d3 == "N13" // obstructive and reflux uropathy
        || matches!(d3, "N18" | "N19" | "N25" | "N26" | "N27"); // chronic kidney disease
    hit.then_some(DiseaseCategory::Renal)
}

/// Chronic conditions originating in the perinatal period (selected chapter XVI
/// codes). `P07` is an addition beyond the published list — see `REFERENCES.md`.
fn perinatal(d3: &str, d4: &str) -> Option<DiseaseCategory> {
    let hit = d3 == "P07" // short gestation / low birth weight, NEC (addition)
        || d3 == "P27" // chronic respiratory disease originating perinatally
        || d3 == "P57" // kernicterus
        || matches!(d4, "P910" | "P911" | "P912") // disturbances of cerebral status of newborn
        || ("P941"..="P949").contains(&d4); // disorders of muscle tone of newborns
    hit.then_some(DiseaseCategory::Perinatal)
}

/// Congenital malformations and chromosomal abnormalities (selected chapter XVII
/// codes).
fn congenital(d3: &str, d4: &str) -> Option<DiseaseCategory> {
    let hit = matches!(d3, "Q01" | "Q02" | "Q03" | "Q04" | "Q05" | "Q06" | "Q07") // nervous system
        || matches!(d3, "Q20" | "Q21" | "Q22" | "Q23" | "Q24" | "Q25" | "Q26") // circulatory system
        || matches!(d3, "Q30" | "Q31" | "Q32" | "Q33") // nose, larynx, trachea, bronchus and lung
        || matches!(d3, "Q34" | "Q35" | "Q36" | "Q37") // other respiratory, cleft lip and palate
        || d3 == "Q39"
        || matches!(d3, "Q40" | "Q41" | "Q42" | "Q43" | "Q44")
        || matches!(d4, "Q450" | "Q451" | "Q452" | "Q453") // upper alimentary tract, intestine, pancreas
        || matches!(d3, "Q60" | "Q61" | "Q62" | "Q63" | "Q64") // urinary system
        || d4 == "Q790" // congenital diaphragmatic hernia
        || matches!(d4, "Q792" | "Q793") // exomphalos and gastroschisis
        || d4 == "Q860" // fetal alcohol syndrome (dysmorphic)
        || d3 == "Q87" // malformation syndromes (multiple systems)
        || matches!(d3, "Q90" | "Q91" | "Q92" | "Q93" | "Q94" | "Q95" | "Q96" | "Q97" | "Q98" | "Q99"); // chromosomal abnormalities
    hit.then_some(DiseaseCategory::Congenital)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A representative positive code for each included condition, with the
    /// category it must classify into. Codes are in raw register form (`D` + ICD).
    const SCD_POSITIVES: &[(&str, DiseaseCategory)] = &[
        ("DC00", DiseaseCategory::Neoplasms),
        ("DC509", DiseaseCategory::Neoplasms),
        ("DC910", DiseaseCategory::Neoplasms),
        ("DD610", DiseaseCategory::Blood),
        ("DD619", DiseaseCategory::Blood),
        ("DD762", DiseaseCategory::Blood),
        ("DD80", DiseaseCategory::Immunodeficiency),
        ("DD82", DiseaseCategory::Immunodeficiency),
        ("DE10", DiseaseCategory::Metabolic),
        ("DE25", DiseaseCategory::Metabolic),
        ("DE70", DiseaseCategory::Metabolic),
        ("DE730", DiseaseCategory::Metabolic),
        ("DE74", DiseaseCategory::Metabolic),
        ("DE84", DiseaseCategory::Metabolic),
        ("DG12", DiseaseCategory::Neurological),
        ("DG310", DiseaseCategory::Neurological),
        ("DG37", DiseaseCategory::Neurological),
        ("DG40", DiseaseCategory::Neurological),
        ("DG60", DiseaseCategory::Neurological),
        ("DG702", DiseaseCategory::Neurological),
        ("DG711", DiseaseCategory::Neurological),
        ("DG736", DiseaseCategory::Neurological),
        ("DG80", DiseaseCategory::Neurological),
        ("DG811", DiseaseCategory::Neurological),
        ("DG91", DiseaseCategory::Neurological),
        ("DG941", DiseaseCategory::Neurological),
        ("DI12", DiseaseCategory::Cardiac),
        ("DI27", DiseaseCategory::Cardiac),
        ("DI30", DiseaseCategory::Cardiac),
        ("DI52", DiseaseCategory::Cardiac),
        ("DJ448", DiseaseCategory::Respiratory),
        ("DJ84", DiseaseCategory::Respiratory),
        ("DK21", DiseaseCategory::Gastrointestinal),
        ("DK50", DiseaseCategory::Gastrointestinal),
        ("DK70", DiseaseCategory::Gastrointestinal),
        ("DK77", DiseaseCategory::Gastrointestinal),
        ("DK90", DiseaseCategory::Gastrointestinal),
        ("DM30", DiseaseCategory::ConnectiveTissue),
        ("DM35", DiseaseCategory::ConnectiveTissue),
        ("DN03", DiseaseCategory::Renal),
        ("DN07", DiseaseCategory::Renal),
        ("DN13", DiseaseCategory::Renal),
        ("DN18", DiseaseCategory::Renal),
        ("DN27", DiseaseCategory::Renal),
        ("DP27", DiseaseCategory::Perinatal),
        ("DP57", DiseaseCategory::Perinatal),
        ("DP910", DiseaseCategory::Perinatal),
        ("DP945", DiseaseCategory::Perinatal),
        ("DQ01", DiseaseCategory::Congenital),
        ("DQ20", DiseaseCategory::Congenital),
        ("DQ30", DiseaseCategory::Congenital),
        ("DQ35", DiseaseCategory::Congenital),
        ("DQ39", DiseaseCategory::Congenital),
        ("DQ452", DiseaseCategory::Congenital),
        ("DQ60", DiseaseCategory::Congenital),
        ("DQ790", DiseaseCategory::Congenital),
        ("DQ792", DiseaseCategory::Congenital),
        ("DQ860", DiseaseCategory::Congenital),
        ("DQ87", DiseaseCategory::Congenital),
        ("DQ90", DiseaseCategory::Congenital),
        ("DQ99", DiseaseCategory::Congenital),
    ];

    /// Codes that must NOT be SCD, including near-miss codes just outside the
    /// included ICD-10 ranges.
    const NON_SCD_CODES: &[&str] = &[
        "DJ45",  // asthma
        "DJ44",  // COPD (only the specific J448 qualifies)
        "DD55",  // sickle-cell / thalassaemia block (not in R)
        "DD66",  // haemophilia (not in R)
        "DM05",  // rheumatoid arthritis (not in R)
        "DE11",  // type 2 diabetes
        "DI83",  // varicose veins
        "DK86",  // pancreas (not in R)
        "DN20",  // calculus of kidney (not in R)
        "DQ89",  // other congenital (just below the Q90 chromosomal block)
        "DZ000", // general examination
        "DR51",  // headache
        "",      // empty
    ];

    #[test]
    fn matches_known_positive_codes() {
        let codes = ScdDiagnosisCodeSet::new();
        for (code, expected) in SCD_POSITIVES {
            assert_eq!(
                codes.category_of(code),
                Some(*expected),
                "{code} should classify as {expected:?}"
            );
        }
    }

    #[test]
    fn rejects_non_scd_codes() {
        let codes = ScdDiagnosisCodeSet::new();
        for code in NON_SCD_CODES {
            assert!(!codes.is_scd_code(code), "{code} should not be SCD");
        }
    }

    #[test]
    fn every_category_is_reachable() {
        let codes = ScdDiagnosisCodeSet::new();
        for category in DiseaseCategory::ALL {
            assert!(
                SCD_POSITIVES
                    .iter()
                    .any(|(code, expected)| *expected == category
                        && codes.category_of(code) == Some(category)),
                "no positive test code reaches {category:?}"
            );
        }
    }

    #[test]
    fn p07_is_perinatal_scd() {
        // P07 (short gestation / low birth weight, NEC) is a deliberate addition
        // beyond the published Jensen 2019 code list; see REFERENCES.md.
        let codes = ScdDiagnosisCodeSet::new();
        assert_eq!(codes.category_of("DP07"), Some(DiseaseCategory::Perinatal));
        assert_eq!(codes.category_of("DP070"), Some(DiseaseCategory::Perinatal));
        assert_eq!(codes.category_of("DP073"), Some(DiseaseCategory::Perinatal));
        // P08 (long gestation / high birth weight) is *not* included.
        assert_eq!(codes.category_of("DP08"), None);
    }

    #[test]
    fn category_boundaries_are_respected() {
        let codes = ScdDiagnosisCodeSet::new();
        // I30–I52 range edges.
        assert_eq!(codes.category_of("DI29"), None);
        assert_eq!(codes.category_of("DI53"), None);
        // Endocrine: only E730 qualifies in the E73 block, and the E74–E84
        // range stops before E85.
        assert_eq!(codes.category_of("DE730"), Some(DiseaseCategory::Metabolic));
        assert_eq!(codes.category_of("DE739"), None);
        assert_eq!(codes.category_of("DE85"), None);
        // Q90–Q99 vs Q89.
        assert_eq!(codes.category_of("DQ89"), None);
        assert_eq!(codes.category_of("DQ90"), Some(DiseaseCategory::Congenital));
    }
}
