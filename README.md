# `scd` — Severe Chronic Disease definition

The single, canonical, citable definition of how our research group identifies
**severe chronic disease (SCD)** from ICD-10 diagnosis codes. It implements the
published Danish paediatric SCD code list (Kristensen et al. 2012; Jensen et al.
2019, 2022), with a small number of documented modifications — see
[`REFERENCES.md`](REFERENCES.md) for the full provenance.

This crate is intentionally small and dependency-light (`chrono`, `serde`) so it
can be shared, unchanged, across every project that needs the definition. It
knows nothing about data sources: callers map their own data into `ScdDiagnosis`
records and get `ScdResult`s back.

## Usage

Add it as a git dependency (pin a tag for reproducibility):

```toml
[dependencies]
scd = { git = "https://github.com/tkragholm/scd-algorithm", tag = "v0.1.0" }
chrono = { version = "0.4", features = ["serde"] }
```

```rust
use chrono::NaiveDate;
use scd::{ScdDiagnosis, compute_scd_results};

let date = |y, m, d| NaiveDate::from_ymd_opt(y, m, d);
let diagnoses = vec![
    ScdDiagnosis::new("p1", "DE10", date(2015, 3, 1)),  // type 1 diabetes (SCD)
    ScdDiagnosis::new("p1", "DJ45", date(2014, 1, 1)),  // asthma (NOT SCD)
];

let (results, stats) = compute_scd_results(&diagnoses);
assert!(results[0].is_scd);
assert_eq!(results[0].first_scd_date, date(2015, 3, 1));
```

## The input/output contract

- **Input** — a flat `&[ScdDiagnosis]` of `(person_id, diagnosis_code, optional date)`.
  Raw register codes are accepted as-is: a leading register `D` before a letter
  is stripped (`"DJ45"` → `"J45"`), punctuation and whitespace are removed, and
  everything is upper-cased. Genuine ICD `D`-codes (`"D55"`, register form
  `"DD55"`) are preserved.
- **Output** — one `ScdResult` **per SCD-positive person**. People with no
  qualifying diagnosis produce _no row_, so every returned `ScdResult` has
  `is_scd == true`. Each row carries the earliest qualifying date, that
  diagnosis's broad `Icd10Chapter`, and a `disease_categories` map keyed by the
  twelve `DiseaseCategory` values. Results are sorted by `person_id`.
- **`ScdStats`** — input-quality counters across all rows: `missing_diagnoses`
  (blank codes), `missing_dates` (undated SCD diagnoses), and `unmapped_codes`
  (non-SCD codes). The computation never fails.

## The definition

A diagnosis is SCD when, after normalization, its ICD-10 code matches one of the
conditions below. Codes are shown in **ICD form** (the register prepends a `D`,
which is stripped during normalization). Each SCD code maps to exactly one
category.

| Category (`key`)                        | ICD-10 codes                                                                                                                             |
| --------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Neoplasms (`neoplasms`)                 | `C00`–`C99` (all malignant neoplasms)                                                                                                    |
| Blood (`blood_disorders`)               | aplastic anaemias `D610/3/8/9`; haemophagocytic syndrome `D762`                                                                          |
| Immunodeficiency (`immunodeficiency`)   | `D80`–`D82`                                                                                                                              |
| Metabolic (`endocrine_metabolic`)       | `E10`, `E25`, `E70`–`E72`, `E730`, `E74`–`E84` (incl. cystic fibrosis)                                                                   |
| Neurological (`neurological`)           | `G12`, `G310/8/9`, `G37`, `G40`, `G60`, `G702`, `G710`–`G713`, `G736`, `G80`, `G811/G821/G824`, `G91`, `G941`                            |
| Cardiac (`cardiac`)                     | `I12`, `I27`, `I30`–`I52`                                                                                                                |
| Respiratory (`respiratory`)             | `J448`, `J84`                                                                                                                            |
| Gastrointestinal (`gastrointestinal`)   | `K21`, `K50`/`K51`, `K70`–`K77`, `K90`                                                                                                   |
| Connective tissue (`connective_tissue`) | `M30`–`M35`                                                                                                                              |
| Renal (`renal`)                         | `N03`–`N05`, `N07`, `N13`, `N18`/`N19`, `N25`–`N27`                                                                                      |
| Perinatal (`perinatal`)                 | `P07`&dagger;, `P27`, `P57`, `P910`–`P912`, `P941`–`P949`                                                                                |
| Congenital (`congenital`)               | `Q01`–`Q07`, `Q20`–`Q26`, `Q30`–`Q37`, `Q39`, `Q40`–`Q44`, `Q450`–`Q453`, `Q60`–`Q64`, `Q790`, `Q792`/`Q793`, `Q860`, `Q87`, `Q90`–`Q99` |

The authoritative, executable source is [`src/code_sets.rs`](src/code_sets.rs),
which encodes each condition explicitly; this table is a summary. The exact
published code list and every difference from it are documented in
[`REFERENCES.md`](REFERENCES.md).

> &dagger; `P07` (disorders related to short gestation and low birth weight, not
> elsewhere classified) is a deliberate addition in this implementation, beyond
> the published Jensen 2019 Table B. See [`REFERENCES.md`](REFERENCES.md).

> **Notable exclusions** (deliberately _not_ SCD): asthma `J45`, COPD `J44`
> except `J448`, type 2 diabetes `E11`, sickle-cell/thalassaemia/haemophilia
> (`D55`–`D60`, `D66`, …), and rheumatoid arthritis `M05`/`M06`. Two conditions
> are grouped by ICD chapter rather than clinical wording: hypertensive renal
> disease `I12` → _Cardiac_, perinatal chronic respiratory disease `P27` →
> _Perinatal_.

Changing the definition means editing `src/code_sets.rs` (and recording any
intentional divergence from the published list in `REFERENCES.md`, as done for
`P07`), updating the tests that pin behaviour, and cutting a new tagged release.

## License

Licensed under the [MIT License](LICENSE).
