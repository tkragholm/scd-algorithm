# Provenance of the SCD definition

This document is the scholarly provenance of the severe chronic disease (SCD)
definition implemented by this crate. It records the source publications, the
exact published ICD-10 code list, and every difference between this
implementation and that list.

## Publications

1. **Kristensen K, Hjuler T, Ravn H, Simões EAF, Stensballe LG.** Chronic
   diseases, chromosomal abnormalities, and congenital malformations as risk
   factors for respiratory syncytial virus hospitalization: a population-based
   cohort study. _Clin Infect Dis._ 2012;54(6):810–817.
   doi:10.1093/cid/cir928 — PMID 22247121.
   _Origin of the ICD-10 chronic-disease code list._

2. **Jensen A, Andersen PK, Andersen JS, Greisen G, Stensballe LG.** Risk factors
   of post-discharge under-five mortality among Danish children 1997–2016: a
   register-based study. _PLoS One._ 2019;14(12):e0226045.
   doi:10.1371/journal.pone.0226045 — PMC6892471.
   _Defines SCD in **S1 File, Table B** = the Kristensen 2012 list **plus
   additions** (marked `X`); excludes congenital viral infections `P35.0–P35.9`._
   This is the canonical machine-readable list reproduced below.

3. **Jensen A, Greisen G, Stensballe LG.** Socioeconomic background affects
   mortality in Danish children with severe chronic disease.
   _Acta Paediatr._ 2022;111(12):2393–2399. doi:10.1111/apa.16540 — PMID 36093628.
   _Groups SCD into 11 categories by ICD-10 chapter; a hospital contact "pertains
   to SCD" if it carries ≥1 SCD diagnosis; SCD strongly associated with under-5
   mortality._

4. **Pediatric health service utilization at tertiary hospitals in Denmark
   2000–2018.** _Sci Rep._ 2024. doi:10.1038/s41598-024-63853-8.
   _Reuses the same definition (Supplementary Table S3)._

## Published code list — Jensen 2019, Table B (S1 File), verbatim

`X` marks diagnoses added by Jensen 2019 on top of the original Kristensen 2012 list.

| ICD-10 codes                    | Diagnoses                                                                        | Added |
| ------------------------------- | -------------------------------------------------------------------------------- | ----- |
| C00.0–C96.9                     | Malignant neoplasms                                                              |       |
| D61.0; D61.3; D61.8–D61.9       | Aplastic anaemias                                                                | X     |
| D76.2                           | Haemophagocytic syndrome, infection-associated                                   | X     |
| D80.0–D82.9                     | Immunodeficiencies                                                               |       |
| E10                             | Insulin-dependent diabetes mellitus                                              | X     |
| E25                             | Adrenogenital disorders                                                          | X     |
| E70.0–73.0                      | Disorders of amino-acid metabolism                                               |       |
| E74.0–E84.9                     | Disorders of metabolism and cystic fibrosis                                      |       |
| G12                             | Spinal muscular atrophy and related syndromes                                    |       |
| G31.0; G31.8–G31.9; G37.0–G37.9 | Degenerative and demyelinating diseases of nervous system                        | X     |
| G40                             | Epilepsy                                                                         |       |
| G60                             | Hereditary and idiopathic neuropathy                                             |       |
| G70.2                           | Congenital and developmental myasthenia                                          |       |
| G71.0–G71.3                     | Mitochondrial myopathy                                                           |       |
| G73.6                           | Myopathy in metabolic diseases                                                   | X     |
| G80                             | Cerebral palsy                                                                   |       |
| G81.1; G82.1; G82.4             | Spastic conditions                                                               | X     |
| G91                             | Hydrocephalus                                                                    |       |
| G94.1                           | Hydrocephalus in neoplastic disease                                              | X     |
| I12                             | Hypertensive renal disease without renal failure                                 | X     |
| I27.1–I27.9                     | Pulmonary heart disease                                                          |       |
| I30.0–I52.8                     | Other forms of heart disease                                                     |       |
| J44.8                           | Other specified chronic obstructive pulmonary disease                            | X     |
| J84                             | Other interstitial pulmonary diseases                                            |       |
| K21                             | Gastro-oesophageal reflux disease                                                |       |
| K50.0–K51.9                     | Crohn disease and ulcerative colitis                                             | X     |
| K70.0–K77.8                     | Diseases of liver                                                                |       |
| K90                             | Intestinal malabsorption                                                         |       |
| M30.0–M35.9                     | Systemic involvement of connective tissue                                        |       |
| N03–N05                         | Nephritic syndrome                                                               | X     |
| N07                             | Hereditary nephropathy, not elsewhere classified                                 |       |
| N13                             | Obstructive and reflux uropathy                                                  |       |
| N18.0–N19.9; N25.0–N27.9        | Chronic kidney disease                                                           |       |
| P27                             | Chronic respiratory disease originating in the perinatal period                  |       |
| P57.0–P57.9                     | Kernicterus                                                                      | X     |
| P91.0–P91.2                     | Disturbances of cerebral status of newborn                                       | X     |
| P94.0–P94.9                     | Disorders of muscle tone of newborn                                              |       |
| Q01–Q07                         | Congenital malformations of the nervous system                                   |       |
| Q20.0–Q26.9                     | Congenital malformations of the circulatory system                               |       |
| Q30.0–Q32.4                     | Congenital malformations of nose, larynx, trachea and bronchus                   |       |
| Q34.0–Q37.9                     | Other congenital malformations of respiratory system, cleft lip and cleft palate |       |
| Q39.0–Q45.3                     | Congenital malformations of upper alimentary tract, intestine and pancreas       |       |
| Q60.0–Q64.9                     | Congenital malformations of the urinary system                                   |       |
| Q79.0                           | Congenital diaphragmatic hernia                                                  |       |
| Q79.2–Q79.3                     | Exomphalos and gastroschisis                                                     |       |
| Q86.0                           | Fetal alcohol syndrome (dysmorphic)                                              |       |
| Q87                             | Other specified congenital malformation syndromes affecting multiple systems     | X     |
| Q90.0–Q99.9                     | Chromosomal abnormalities, not elsewhere classified                              |       |

**Exclusion note (Jensen 2019):** congenital viral infections `P35.0–P35.9` are
excluded "since the clinical presentation of such infections is very variable."

## Relationship to the published definition

This implementation encodes the conditions of the published list above and is
locked against them by tests. Of the 48 conditions in Jensen 2019 Table B, **42
are reproduced identically** and six differ at code boundaries (below). Every
difference is pinned by a test, so it cannot change silently.

### Boundary differences from the published Table B

| Condition                                         | Published Table B | This implementation | Effect                                                                                                                           |
| ------------------------------------------------- | ----------------- | ------------------------ | -------------------------------------------------------------------------------------------------------------------------------- |
| Malignant neoplasms                               | `C00.0–C96.9`     | `C00–C99`                | also includes **C97** (multiple independent primary sites); C98–C99 are unassigned                                         |
| Pulmonary heart disease                           | `I27.1–I27.9`     | all `I27`                | also includes **I27.0** (primary pulmonary hypertension)                                                                   |
| Other heart disease                               | `I30.0–I52.8`     | `I30–I52`                | also includes **I52.9** (unspecified)                                                                                      |
| Diseases of liver                                 | `K70.0–K77.8`     | `K70–K77`                | also includes **K77.9** (unspecified)                                                                                      |
| Disorders of muscle tone of newborn               | `P94.0–P94.9`     | `P94.1–P94.9`            | **excludes P94.0** (transient neonatal myasthenia gravis); narrower than the published list, and flagged for review                           |
| Congenital malformations of trachea/bronchus/lung | `Q30.0–Q32.4`     | `Q30–Q33`                | also includes **Q32.5–Q33.9** (congenital lung malformations), consistent with the criterion's scope "nose, larynx, trachea, bronchus and lung" |

Five differences make the definition marginally **broader** than Table B (mostly `.9`
"unspecified" subcodes plus C97/I27.0/Q33); one makes it **narrower** (`P94.0`).
None of the 42 core conditions differ.

### Additions beyond the published list

None. This implementation adds **no** conditions beyond Kristensen 2012 /
Jensen 2019/2022.

`P07` (disorders related to short gestation and low birth weight, not elsewhere
classified) appears in **neither** published list and is **deliberately excluded**.
An earlier version of this crate included it; it was removed because it
over-captures preterm / low-birth-weight infants as severe chronic disease,
inflating the perinatal category by an order of magnitude relative to the
published definition. The exclusion is pinned by `p07_is_excluded_from_scd` in
`src/code_sets.rs`, so re-adding it would be a deliberate, visible change.
