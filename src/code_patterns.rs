//! Diagnosis-code normalization and ICD-10 chapter classification.
//!
//! * [`normalize_diagnosis_code`] canonicalizes a raw register code into the
//!   form the SCD definition is written against (see [`crate::code_sets`]).
//! * [`Icd10Chapter`] classifies a code into one of the 22 ICD-10 chapters; it
//!   is the broad chapter recorded on each [`ScdResult`](crate::ScdResult),
//!   distinct from the finer [`DiseaseCategory`](crate::DiseaseCategory).
//!
//! Both are deterministic and side-effect free.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// A diagnosis code that has been canonicalized by [`normalize_diagnosis_code`].
///
/// Holding a `NormalizedDiagnosis` is a small type-level guarantee that the
/// contained code has already been normalized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedDiagnosis {
    /// The canonicalized, uppercase, alphanumeric code (e.g. `"C500"`), with any
    /// Danish register `D` prefix removed and the ICD chapter letter first.
    pub full_code: String,
}

impl NormalizedDiagnosis {
    /// The 3-character ICD category (chapter letter + two digits), e.g. `"C50"`.
    ///
    /// This is the 3-character ICD-10 category code. Returns a shorter slice if
    /// the code has fewer than three characters.
    #[must_use]
    pub fn category3(&self) -> &str {
        let end = self.full_code.len().min(3);
        &self.full_code[..end]
    }

    /// The 4-character ICD sub-category, e.g. `"C509"`.
    ///
    /// This is the 4-character ICD-10 subcategory code. Returns a shorter slice
    /// if the code has fewer than four characters.
    #[must_use]
    pub fn subcategory4(&self) -> &str {
        let end = self.full_code.len().min(4);
        &self.full_code[..end]
    }
}

/// Canonicalize a raw register diagnosis code.
///
/// The Danish national registers prefix ICD-10 codes with a leading `D`
/// (e.g. `"DJ45"`). Normalization:
///
/// 1. trims and upper-cases the code, removing all whitespace;
/// 2. strips a single leading `D` when it is a register prefix — i.e. when the
///    second character is a letter (`"DJ45"` → `"J45"`), while genuine ICD
///    `D`-codes such as `"D55"` (second character is a digit) are preserved;
/// 3. drops any leading non-letters and all non-alphanumeric characters so the
///    result begins with the ICD chapter letter (e.g. `"J45.0"` → `"J450"`).
///
/// Returns [`None`] for codes that are empty or contain no letter.
#[must_use]
pub fn normalize_diagnosis_code(code: &str) -> Option<NormalizedDiagnosis> {
    let mut text = code.trim().to_uppercase();
    if text.is_empty() {
        return None;
    }
    text.retain(|c| !c.is_whitespace());
    if text.starts_with('D') {
        let mut chars = text.chars();
        let _ = chars.next();
        if let Some(next) = chars.next()
            && next.is_ascii_alphabetic()
        {
            let mut rest = String::new();
            rest.push(next);
            rest.extend(chars);
            text = rest;
        }
    }

    let mut cleaned = String::with_capacity(text.len());
    let mut seen_letter = false;
    for ch in text.chars() {
        if !ch.is_ascii_alphanumeric() {
            continue;
        }
        if !seen_letter {
            if ch.is_ascii_alphabetic() {
                seen_letter = true;
                cleaned.push(ch);
            }
            continue;
        }
        cleaned.push(ch);
    }

    if cleaned.is_empty() {
        return None;
    }
    Some(NormalizedDiagnosis { full_code: cleaned })
}

/// Returns the canonical chapter label for a raw diagnosis code, or [`None`] if
/// the code cannot be classified.
#[must_use]
pub fn icd10_chapter_label(code: &str) -> Option<String> {
    Icd10Chapter::from_diagnosis_code(code).map(|chapter| chapter.as_label().to_string())
}

/// One of the 22 chapters of ICD-10.
///
/// The [label](Self::as_label) is the canonical string form used when an SCD
/// result is serialized (e.g. `"II Neoplasms"`), and round-trips through
/// [`Self::from_label`] / [`FromStr`]. Serde uses the label form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Icd10Chapter {
    /// I — Certain infectious and parasitic diseases (`A`, `B`).
    Infectious,
    /// II — Neoplasms (`C`, `D00`–`D49`).
    Neoplasms,
    /// III — Diseases of the blood and blood-forming organs and immune disorders (`D50`–`D89`).
    Blood,
    /// IV — Endocrine, nutritional and metabolic diseases (`E`).
    Endocrine,
    /// V — Mental and behavioural disorders (`F`).
    Mental,
    /// VI — Diseases of the nervous system (`G`).
    Nervous,
    /// VII — Diseases of the eye and adnexa (`H00`–`H59`).
    Eye,
    /// VIII — Diseases of the ear and mastoid process (`H60`–`H95`).
    Ear,
    /// IX — Diseases of the circulatory system (`I`).
    Circulatory,
    /// X — Diseases of the respiratory system (`J`).
    Respiratory,
    /// XI — Diseases of the digestive system (`K`).
    Digestive,
    /// XII — Diseases of the skin and subcutaneous tissue (`L`).
    Skin,
    /// XIII — Diseases of the musculoskeletal system and connective tissue (`M`).
    Musculoskeletal,
    /// XIV — Diseases of the genitourinary system (`N`).
    Genitourinary,
    /// XV — Pregnancy, childbirth and the puerperium (`O`).
    Pregnancy,
    /// XVI — Certain conditions originating in the perinatal period (`P`).
    Perinatal,
    /// XVII — Congenital malformations, deformations and chromosomal abnormalities (`Q`).
    Congenital,
    /// XVIII — Symptoms, signs and abnormal clinical and laboratory findings (`R`).
    Symptoms,
    /// XIX — Injury, poisoning and certain other consequences of external causes (`S`, `T`).
    Injury,
    /// XX — External causes of morbidity and mortality (`V`, `W`, `X`, `Y`).
    External,
    /// XXI — Factors influencing health status and contact with health services (`Z`).
    Factors,
    /// XXII — Codes for special purposes (`U`).
    Special,
}

impl Icd10Chapter {
    /// Every chapter in canonical (roman-numeral) order.
    pub const ALL: [Self; 22] = [
        Self::Infectious,
        Self::Neoplasms,
        Self::Blood,
        Self::Endocrine,
        Self::Mental,
        Self::Nervous,
        Self::Eye,
        Self::Ear,
        Self::Circulatory,
        Self::Respiratory,
        Self::Digestive,
        Self::Skin,
        Self::Musculoskeletal,
        Self::Genitourinary,
        Self::Pregnancy,
        Self::Perinatal,
        Self::Congenital,
        Self::Symptoms,
        Self::Injury,
        Self::External,
        Self::Factors,
        Self::Special,
    ];

    /// Classify a raw register diagnosis code into its ICD-10 chapter.
    ///
    /// The code is normalized via [`normalize_diagnosis_code`] first. Returns
    /// [`None`] when the code cannot be normalized or its leading letter is not
    /// a valid ICD-10 chapter letter.
    #[must_use]
    pub fn from_diagnosis_code(code: &str) -> Option<Self> {
        let normalized = normalize_diagnosis_code(code)?;
        Self::from_normalized(&normalized.full_code)
    }

    /// Classify an already-normalized code (uppercase, letter-first).
    #[must_use]
    pub fn from_normalized(code: &str) -> Option<Self> {
        let trimmed = code.trim();
        let letter = trimmed.chars().next()?.to_ascii_uppercase();
        let number = parse_chapter_number(trimmed);

        match letter {
            'A' | 'B' => Some(Self::Infectious),
            'C' => Some(Self::Neoplasms),
            'D' => match number {
                Some(n) if n <= 49 => Some(Self::Neoplasms),
                Some(_) => Some(Self::Blood),
                None => None,
            },
            'E' => Some(Self::Endocrine),
            'F' => Some(Self::Mental),
            'G' => Some(Self::Nervous),
            'H' => match number {
                Some(n) if n <= 59 => Some(Self::Eye),
                Some(n) if n <= 95 => Some(Self::Ear),
                _ => None,
            },
            'I' => Some(Self::Circulatory),
            'J' => Some(Self::Respiratory),
            'K' => Some(Self::Digestive),
            'L' => Some(Self::Skin),
            'M' => Some(Self::Musculoskeletal),
            'N' => Some(Self::Genitourinary),
            'O' => Some(Self::Pregnancy),
            'P' => Some(Self::Perinatal),
            'Q' => Some(Self::Congenital),
            'R' => Some(Self::Symptoms),
            'S' | 'T' => Some(Self::Injury),
            'V' | 'W' | 'X' | 'Y' => Some(Self::External),
            'Z' => Some(Self::Factors),
            'U' => Some(Self::Special),
            _ => None,
        }
    }

    /// The canonical label, e.g. `"II Neoplasms"`. Stable; changing it changes
    /// the on-disk representation of [`crate::ScdResult`].
    #[must_use]
    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Infectious => "I Infectious and parasitic diseases",
            Self::Neoplasms => "II Neoplasms",
            Self::Blood => "III Blood and immune disorders",
            Self::Endocrine => "IV Endocrine, nutritional and metabolic diseases",
            Self::Mental => "V Mental and behavioural disorders",
            Self::Nervous => "VI Diseases of the nervous system",
            Self::Eye => "VII Diseases of the eye and adnexa",
            Self::Ear => "VIII Diseases of the ear and mastoid process",
            Self::Circulatory => "IX Diseases of the circulatory system",
            Self::Respiratory => "X Diseases of the respiratory system",
            Self::Digestive => "XI Diseases of the digestive system",
            Self::Skin => "XII Diseases of the skin and subcutaneous tissue",
            Self::Musculoskeletal => "XIII Diseases of the musculoskeletal system",
            Self::Genitourinary => "XIV Diseases of the genitourinary system",
            Self::Pregnancy => "XV Pregnancy, childbirth and the puerperium",
            Self::Perinatal => "XVI Certain conditions originating in the perinatal period",
            Self::Congenital => {
                "XVII Congenital malformations, deformations and chromosomal abnormalities"
            }
            Self::Symptoms => "XVIII Symptoms, signs and abnormal findings",
            Self::Injury => "XIX Injury, poisoning and external causes",
            Self::External => "XX External causes of morbidity and mortality",
            Self::Factors => {
                "XXI Factors influencing health status and contact with health services"
            }
            Self::Special => "XXII Codes for special purposes",
        }
    }

    /// The roman-numeral chapter number, e.g. `"II"`.
    #[must_use]
    pub const fn roman(self) -> &'static str {
        match self {
            Self::Infectious => "I",
            Self::Neoplasms => "II",
            Self::Blood => "III",
            Self::Endocrine => "IV",
            Self::Mental => "V",
            Self::Nervous => "VI",
            Self::Eye => "VII",
            Self::Ear => "VIII",
            Self::Circulatory => "IX",
            Self::Respiratory => "X",
            Self::Digestive => "XI",
            Self::Skin => "XII",
            Self::Musculoskeletal => "XIII",
            Self::Genitourinary => "XIV",
            Self::Pregnancy => "XV",
            Self::Perinatal => "XVI",
            Self::Congenital => "XVII",
            Self::Symptoms => "XVIII",
            Self::Injury => "XIX",
            Self::External => "XX",
            Self::Factors => "XXI",
            Self::Special => "XXII",
        }
    }

    /// Parse a chapter from its canonical [label](Self::as_label).
    #[must_use]
    pub fn from_label(label: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|chapter| chapter.as_label() == label)
    }
}

impl fmt::Display for Icd10Chapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_label())
    }
}

impl FromStr for Icd10Chapter {
    type Err = UnknownChapter;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_label(s).ok_or_else(|| UnknownChapter(s.to_string()))
    }
}

/// Error returned when a string cannot be parsed into an [`Icd10Chapter`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownChapter(pub String);

impl fmt::Display for UnknownChapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown ICD-10 chapter label: {:?}", self.0)
    }
}

impl std::error::Error for UnknownChapter {}

impl Serialize for Icd10Chapter {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_label())
    }
}

impl<'de> Deserialize<'de> for Icd10Chapter {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let label = String::deserialize(deserializer)?;
        Self::from_label(&label).ok_or_else(|| de::Error::custom(UnknownChapter(label)))
    }
}

fn parse_chapter_number(code: &str) -> Option<u32> {
    let digits: String = code
        .chars()
        .skip(1)
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_register_prefixed_codes() {
        assert_eq!(normalize_diagnosis_code("DJ45").unwrap().full_code, "J45");
        assert_eq!(
            normalize_diagnosis_code("dj45.0 ").unwrap().full_code,
            "J450"
        );
    }

    #[test]
    fn preserves_genuine_d_codes() {
        // `D55` is a real ICD blood code; the second char is a digit, so the
        // leading `D` must NOT be treated as a register prefix.
        assert_eq!(normalize_diagnosis_code("D55").unwrap().full_code, "D55");
        // Register form of the same code is `DD55`.
        assert_eq!(normalize_diagnosis_code("DD55").unwrap().full_code, "D55");
    }

    #[test]
    fn derives_category_and_subcategory_slices() {
        let n = normalize_diagnosis_code("DC509").unwrap();
        assert_eq!(n.full_code, "C509");
        assert_eq!(n.category3(), "C50");
        assert_eq!(n.subcategory4(), "C509");
    }

    #[test]
    fn rejects_empty_and_letterless_codes() {
        assert!(normalize_diagnosis_code("").is_none());
        assert!(normalize_diagnosis_code("   ").is_none());
        assert!(normalize_diagnosis_code("123").is_none());
    }

    #[test]
    fn classifies_chapters() {
        assert_eq!(
            Icd10Chapter::from_diagnosis_code("DJ45"),
            Some(Icd10Chapter::Respiratory)
        );
        assert_eq!(
            Icd10Chapter::from_diagnosis_code("C50"),
            Some(Icd10Chapter::Neoplasms)
        );
        assert_eq!(
            Icd10Chapter::from_diagnosis_code("D40"),
            Some(Icd10Chapter::Neoplasms)
        );
        assert_eq!(
            Icd10Chapter::from_diagnosis_code("D55"),
            Some(Icd10Chapter::Blood)
        );
    }

    #[test]
    fn label_round_trips_through_serde_and_from_str() {
        for chapter in Icd10Chapter::ALL {
            let json = serde_json::to_string(&chapter).unwrap();
            let back: Icd10Chapter = serde_json::from_str(&json).unwrap();
            assert_eq!(back, chapter);
            assert_eq!(chapter.as_label().parse::<Icd10Chapter>().unwrap(), chapter);
        }
    }
}
