//! Diagnosis-code normalization and ICD-10 chapter classification.
//!
//! These are re-exported from [`epistat_register::icd10`], where they are
//! documented.
//!
//! * [`normalize_diagnosis_code`] canonicalizes a raw register code into the
//!   form the SCD definition is written against (see [`crate::code_sets`]).
//! * [`Icd10Chapter`] classifies a code into one of the 22 ICD-10 chapters; it
//!   is the broad chapter recorded on each [`ScdResult`](crate::ScdResult),
//!   distinct from the finer [`DiseaseCategory`](crate::DiseaseCategory).

pub use epistat_register::icd10::{
    Icd10Chapter, NormalizedDiagnosis, UnknownChapter, icd10_chapter_label,
    normalize_diagnosis_code,
};
