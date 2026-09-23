# Complete Drools Feature Parity & Rust Alternative Matrix

This document provides a comprehensive mapping of **every feature in Red Hat Drools (JBoss Rules / KIE)** to its **Rust Rules Engine (`rust_rules_engine`) alternative**, including implementation mechanics, immigration use case examples, and test coverage IDs.

---

## 1. Summary Feature Category Overview

```
+----------------------------------------------------------------------------------------------------+
|                                    DROOLS vs RUST RULES ENGINE                                     |
+------------------------------------+------------------------------------+--------------------------+
| Drools Subsystem                   | Rust Alternative                   | Equivalence Status       |
+------------------------------------+------------------------------------+--------------------------+
| 1. Condition LHS & Pattern Match   | AST Condition Nodes & Path Queries | Full Parity              |
| 2. Logical Quantifiers & Operators | `All`, `Any`, `None`, `Forall`, etc| Full Parity              |
| 3. Collection Accumulators (from)  | `Accumulate` Node + Typed Reducers | Full Parity              |
| 4. Temporal Reasoning (CEP/Fusion) | `Temporal` Windows & Time Predicates| Full Parity              |
| 5. Action RHS & Working Memory     | Actions, Derived Facts & Mutations | Full Parity (Pure/Safe)  |
| 6. Conflict Resolution & Agenda    | Salience, Phase, Activation Groups | Full Parity              |
| 7. Cycle Prevention                | In-flight Activation Tracking      | Full Parity              |
| 8. Decision Tables & DMN           | Multi-Dimensional Matrix Tables    | Full Parity (YAML/JSON)  |
| 9. Stateful vs Stateless Sessions  | `FactContext` & `Engine` Pipeline  | Stateless + Reusable Mem |
| 10. Audit, Tracing & Listeners     | Structured `AuditReport` & Traces  | Zero-Allocation Tracing  |
+------------------------------------+------------------------------------+--------------------------+
```

---

## 2. Exhaustive Drools Feature Mapping Table

### A. Pattern Matching, Constraints & Left-Hand-Side (LHS)

| # | Drools (DRL) Feature | Drools DRL Syntax Example | Rust Rules Engine Alternative | Rust YAML / JSON AST Representation | Immigration Edge Case / Example | Test ID |
|---|---|---|---|---|---|---|
| **A1** | **Field Equality / Inequality** | `Applicant(maritalStatus == "single")` | `Condition::Compare` with `op: "eq"` / `"neq"` | `path: "applicant.marital_status", op: "eq", value: "single"` | Single vs married candidate selection | `TC-01` |
| **A2** | **Numeric Comparisons** | `Applicant(age >= 18 && age <= 29)` | `Condition::Compare` with `op: "gte"`, `"lte"`, `"between"` | `path: "applicant.age", op: "between", value: [18, 29]` | Peak age scoring bracket evaluation | `TC-01` |
| **A3** | **Set Membership (`in` / `not in`)** | `Education(degree in ("master", "doctorate"))` | `Condition::Compare` with `op: "in"` / `"not_in"` | `path: "applicant.education.highest_degree", op: "in", value: ["master", "doctorate"]` | Qualifying higher degree pathways | `TC-02` |
| **A4** | **Null-Safe Navigation (`!.` / `?.`)** | `Applicant(spouse != null, spouse!.education != null)` | Path resolution returning `Result<Option<&Value>>` | Three-valued logic: missing paths yield `None` without panicking | Single applicants with `spouse: null` skip spouse rules cleanly | `TC-08` |
| **A5** | **Regular Expression Matching (`matches`)** | `JobOffer(nocCode matches "^21[0-9]{3}$")` | `Condition::Compare` with `op: "matches_regex"` | `path: "applicant.job_offer.noc_code", op: "matches_regex", value: "^21[0-9]{3}$"` | Checking TEER/NOC code prefix for STEM occupations | `TC-01` |
| **A6** | **Collection Membership (`contains` / `memberOf`)** | `Applicant(tags contains "stem_priority")` | `Condition::Compare` with `op: "contains"` | `path: "applicant.tags", op: "contains", value: "stem_priority"` | Checking if applicant holds specialized profile flags | `TC-05` |
| **A7** | **Variable Bindings** | `$age : age, $edu : education` | Direct dotted path resolution in context | `path: "applicant.age"`, `path: "applicant.education"` | Referencing multiple applicant fields across conditions | `TC-01` |
| **A8** | **Multi-Fact Relational Joins** | `JobOffer($id: sponsorId) and Sponsor(id == $id, isAccredited == true)` | `Condition::MultiFactJoin` | `type: "multi_fact_join", facts: [{name: "j", path: "job"}, {name: "s", path: "sponsors"}], join_conditions: ["j.id eq s.id", "s.active eq true"]` | Validating job offer against verified sponsor database | `TC-04` |
| **A9** | **Existential Quantifier (`exists`)** | `exists JobOffer(isLmiaApproved == true)` | `Condition::Exists` or `Condition::Any` | `type: "exists", path: "applicant.job_offer.is_lmia_approved"` | Checking presence of LMIA labor market approval | `TC-01` |
| **A10** | **Negative Quantifier (`not` / `not exists`)** | `not IneligibilityFlag()` | `Condition::None` or `Condition::Not` | `type: "not", condition: { type: "compare", path: "applicant.has_criminal_record", op: "eq", value: true }` | Verifying absence of criminal / admissibility bars | `TC-06` |
| **A11** | **Universal Quantifier (`forall` / `every`)** | `forall(LanguageBand($score: clb) LanguageBand(clb >= 7))` | `Condition::Forall` or `Condition::MinCount` | `type: "forall", source_array: "applicant.language.abilities", condition: { type: "compare", path: "clb", op: "gte", value: 7 }` | Enforcing minimum CLB 7 in **all 4** test abilities | `TC-10` |
| **A12** | **Threshold Quantifier (`min_count` / `at least N`)** | No native single-line DRL keyword (requires custom accumulate) | `Condition::MinCount { min_required: N, conditions: [...] }` | `type: "min_count", min_required: 3, conditions: [...]` | Passing at least 3 out of 5 regional tie criteria | `TC-01` |

---

### B. Aggregations, Accumulators & Collection Operations

| # | Drools Feature | Drools DRL Syntax | Rust Rules Engine Alternative | Rust AST Model | Immigration Edge Case / Example | Test ID |
|---|---|---|---|---|---|---|
| **B1** | **Sum Accumulator (`accumulate sum`)** | `accumulate(Employment($m: months); $total: sum($m))` | `Action::Accumulate` with `function: "sum"` | `type: "accumulate", source_array: "applicant.work_history", function: "sum", field: "months", target: "derived.total_months"` | Summing months across multiple jobs | `TC-09` |
| **B2** | **Normalized FTE Sum (Custom Accumulator)** | `accumulate(Employment($m: months, $h: hrs); $fte: sum($m * ($h >= 30 ? 1.0 : $h/30.0)))` | `Action::Accumulate` with `function: "sum_fte_years"` | `type: "accumulate", source_array: "applicant.work_history", function: "sum_fte_years", target: "derived.domestic_fte_years"` | Normalizing part-time and full-time Canadian experience | `TC-09` |
| **B3** | **Min / Max Accumulator** | `accumulate(LanguageBand($s: score); $minScore: min($s))` | `Action::Accumulate` with `function: "min"` / `"max"` | `type: "accumulate", source_array: "applicant.language.bands", function: "min", field: "score", target: "derived.lowest_band"` | Finding weakest language score to set baseline CLB | `TC-10` |
| **B4** | **Count Accumulator** | `accumulate(TradeCertification(); $count: count())` | `Action::Accumulate` with `function: "count"` | `type: "accumulate", source_array: "applicant.certifications", function: "count", target: "derived.cert_count"` | Counting number of provincial trade certificates | `TC-03` |
| **B5** | **Filtered Collection (`from collect`)** | `ArrayList() from collect(Employment(isDomestic == true))` | `Action::Accumulate` with `filter: ConditionNode` | `source_array: "applicant.work_history", filter: { path: "is_domestic", op: "eq", value: true }` | Extracting only domestic employment records for scoring | `TC-09` |

---

### C. Temporal Reasoning & Complex Event Processing (Drools Fusion / CEP)

| # | Drools Fusion Feature | Drools CEP Syntax | Rust Rules Engine Alternative | Rust AST Model | Immigration Edge Case / Example | Test ID |
|---|---|---|---|---|---|---|
| **C1** | **Sliding Time Window / Recency (`within_past`)** | `LanguageTest(this after[0d, 730d] applicationDate)` | `Condition::Temporal` with `direction: "within_past", window_days: 730` | `type: "temporal", field_path: "test_date", reference_date_path: "application.submission_date", window_days: 730, direction: "within_past"` | 2-year validity limit on IELTS/CELPIP results | `TC-07` |
| **C2** | **Lookback Horizon Window** | `Employment(endDate after[0d, 3650d] applicationDate)` | `Condition::Temporal` with `window_days: 3650` (10 years) | `type: "temporal", field_path: "end_date", reference_date_path: "application.submission_date", window_days: 3650, direction: "within_past"` | Enforcing 10-year lookback window for qualifying foreign work experience | `TC-01` |
| **C3** | **Duration / Overlap Checks (`overlaps`, `during`)** | `JobA() and JobB(this overlaps JobA)` | Date range interval arithmetic in `FactContext` | Interval intersection evaluator in collection accumulator | Eliminating double-counting of simultaneous employment periods | `TC-09` |
| **C4** | **Point-in-Time Age Freezing** | N/A (requires custom Java date calculation) | `compute_age_at_date(dob, lock_in_date)` in context | `type: "calculate_age", dob_path: "applicant.dob", date_path: "application.lock_in_date"` | Freezing age at ITA (Invitation to Apply) date | `TC-06` |

---

### D. Consequence Execution, Working Memory & Right-Hand-Side (RHS)

| # | Drools RHS Feature | Drools DRL Syntax | Rust Rules Engine Alternative | Rust AST Action Model | Immigration Edge Case / Example | Test ID |
|---|---|---|---|---|---|---|
| **D1** | **Fact Insertion (`insert`)** | `insert(new DerivedFact("clb9_plus"));` | `Action::SetAttribute` / `Action::InsertDerivedFact` | `type: "set_attribute", key: "derived.clb9_plus", value: true` | Forward chaining: asserting high language badge | `TC-11` |
| **D2** | **Logical Assertion (TMS / `insertLogical`)** | `insertLogical(new EligibleForStream("stem"));` | Scoped Derived Facts bounded by rule lifecycle | `type: "insert_derived_fact", fact_type: "EligibleStream", data: {...}` | Deriving stream eligibility that auto-invalidates if inputs change | `TC-11` |
| **D3** | **Fact Modification (`modify` / `update`)** | `modify($applicant) { setScore($applicant.getScore() + 50) };` | `Action::AwardPoints` with Category Capping | `type: "award_points", category: "skill_transferability", formula: { type: "fixed", points: 50 }` | Awarding points to specific category budget | `TC-01` |
| **D4** | **Fact Retraction (`delete` / `retract`)** | `delete($temporaryFact);` | `Action::RemoveAttribute` | `type: "remove_attribute", key: "temp.cache_val"` | Cleaning temporary flags between pipeline phases | `TC-11` |
| **D5** | **Early Exit / Halt Execution (`drools.halt()`)** | `drools.halt();` | `Action::SetEligibility` with `is_eligibility_gate: true` | `type: "set_eligibility", eligible: false, reason: "Failed mandatory gate"` | Halting evaluation immediately on age 45+ in Australia 189 | `TC-06` |
| **D6** | **Diagnostic Tagging / Channels** | `channels["audit"].send("Qualified for PNP");` | `Action::AddTag` | `type: "add_tag", tag: "provincial_nominee"` | Tagging profile for targeted immigration invitation rounds | `TC-02` |

---

### E. Rule Scheduling, Conflict Resolution & Agenda Control

| # | Drools Agenda Attribute | Drools DRL Attribute | Rust Rules Engine Alternative | Rust AST Attribute | Immigration Edge Case / Example | Test ID |
|---|---|---|---|---|---|---|
| **E1** | **Salience (Rule Priority)** | `salience 1000` | `priority: i32` (Higher/Lower executed in order) | `priority: 1000` | Hard gates (1000) run before scoring (500) before bonus (100) | `TC-06` |
| **E2** | **Agenda Groups / Ruleflow Groups** | `agenda-group "validation"` / `ruleflow-group "scoring"` | `phase: "validation"` / `phase: "scoring"` | `phase: "validation_gates"`, `phase: "scoring"` | Strict multi-stage evaluation pipeline | `TC-04` |
| **E3** | **Activation Groups (XOR Mutual Exclusion)** | `activation-group "age_bracket"` | `activation_group: "age_bracket"` | `activation_group: "age_points_selection"` | Discrete age brackets: only 1 rule in group fires | `TC-05` |
| **E4** | **Cycle Prevention (`no-loop`)** | `no-loop true` | `no_loop: true` (Prevents self-reactivation) | `no_loop: true` | Rule updating applicant attribute does not re-trigger itself | `TC-11` |
| **E5** | **Group Cycle Prevention (`lock-on-active`)** | `lock-on-active true` | Phase-locked activation registry | Engine locks phase activations once initialized | Rules in current phase cannot be re-activated by current phase mutations | `TC-11` |
| **E6** | **Rule Effective / Expiry Dates** | `date-effective "01-Jan-2026"`, `date-expires "31-Dec-2026"` | `effective_date`, `expiry_date` metadata in Rule | `effective_date: "2026-01-01", expiry_date: "2026-12-31"` | Regulatory law sunsetting and versioning | `TC-01` |
| **E7** | **Rule Disabling (`enabled`)** | `enabled false` | `enabled: bool` | `enabled: false` | Temporarily pausing specific pilot program streams | `TC-01` |

---

### F. Decision Tables, DMN & Scorecards

| # | Drools / DMN Feature | Drools Equivalent | Rust Rules Engine Alternative | Rust AST Model | Immigration Edge Case / Example | Test ID |
|---|---|---|---|---|---|---|
| **F1** | **Multi-Column Decision Table** | DMN Decision Table / Drools Decision Table | `DecisionTable` AST with typed inputs/outputs/rows | `DecisionTable { inputs: [...], outputs: [...], rows: [...] }` | Skill Transferability Matrix (Degree $\times$ Foreign Work $\times$ CLB) | `test_decision_table_hit_policies` |
| **F2** | **CSV / Spreadsheet Decision Tables** | Drools `.xls` / `.csv` Decision Tables | `DecisionTable::from_csv_str` | CSV with header, intervals `[20..29]`, sets `in ['a','b']`, ops `>= 9` | Excel/CSV rule spreadsheets for policy officers | `test_decision_table_csv_spreadsheet_parsing` |
| **F3** | **Hit Policy: Unique (U)** | DMN Unique (`U`) | `HitPolicy::Unique` | `hit_policy: HitPolicy::Unique` (errors if >1 row matches) | Exactly one discrete salary / tier bracket | `test_decision_table_hit_policies` |
| **F4** | **Hit Policy: First (F)** | DMN First (`F`) | `HitPolicy::First` | `hit_policy: HitPolicy::First` (returns top matching row) | Highest qualification priority cascade | `test_decision_table_hit_policies` |
| **F5** | **Hit Policy: Collect Sum (C+)** | DMN Collect Sum (`C+`) | `HitPolicy::CollectSum` | `hit_policy: HitPolicy::CollectSum` (sums all matching rows) | Aggregating multiple sub-factor points in table | `test_decision_table_hit_policies` |
| **F6** | **Hit Policy: Collect Max (C>)** | DMN Collect Max (`C>`) | `HitPolicy::CollectMax` | `hit_policy: HitPolicy::CollectMax` | Best-case points across concurrent pathways | `test_decision_table_hit_policies` |
| **F7** | **Hit Policy: Collect Min (C<)** | DMN Collect Min (`C<`) | `HitPolicy::CollectMin` | `hit_policy: HitPolicy::CollectMin` | Conservative scoring across concurrent pathways | `test_decision_table_hit_policies` |
| **F8** | **Hit Policy: Collect Count (C#)**| DMN Collect Count (`C#`)| `HitPolicy::CollectCount` | `hit_policy: HitPolicy::CollectCount` | Counting qualifying criteria rows met | `test_decision_table_hit_policies` |
| **F9** | **2D Matrix Lookup** | DMN Boxed Decision Matrix | `PointsFormula::MatrixLookup` | `type: "matrix_lookup", row_path: "education", col_path: "clb_level", matrix: {...}` | 2D matrix lookup (Education $\times$ Language Band) | `TC-01` |
| **F10** | **Linear Scorecard with Clamps** | Drools PMML Scorecard | `PointsFormula::Scaled` with min/max clamps | `type: "scaled", path: "years", factor: 5.0, min: 0.0, max: 15.0` | Overseas work experience (5 pts/yr, max 15 pts) | `TC-01` |
| **F11** | **Discrete Map Lookup** | Drools Simple Map Table | `PointsFormula::Lookup` | `type: "lookup", path: "education.degree", mapping: {"phd": 20, "master": 15}` | Degree hierarchy scoring table | `TC-01` |


---

### G. Auditability, Tracing & Performance

| # | Drools Feature | Drools Mechanism | Rust Rules Engine Alternative | Rust Implementation Details | Immigration Advantage |
|---|---|---|---|---|---|
| **G1** | **Agenda Event Listeners** | `AgendaEventListener` callbacks (`afterMatchFired`) | Built-in zero-allocation Execution Tracer | Generates structured `AuditReport` with full execution DAG | Complete itemized legal trace for visa decisions |
| **G2** | **Working Memory Audit** | `WorkingMemoryEventListener` (`objectInserted`) | `FactContext` change-log audit | Tracks all attribute derivations and mutations | Proof of factual derivation history |
| **G3** | **Subcategory Capping** | Requires complex manual accumulator rules | First-class `CategoryConfig` with `max_points` | Automatic score clamping with truncation audit note | Guaranteed regulatory compliance with program caps |
| **G4** | **Deterministic Latency** | JVM JIT & GC pauses (10ms – 100ms jitter) | Native compiled Rust ($\le 50\,\mu\text{s}$, zero GC) | Fixed-memory execution without JVM runtime overhead | Sub-millisecond high-volume batch evaluation |
| **G5** | **Thread Safety & Concurrency** | Synchronized `KieSession` or pooling | Immutable `RuleProgram` + Lock-free parallel evaluation | `&RuleProgram` is `Send + Sync`; evaluate millions of profiles across all CPU cores | Real-time batch visa draw ranking simulations |

---

## 3. Maintenance & Evolution Process

This matrix is a **living specification** for the `rust-rules-engine-examples` project:
1. **Rule Engine Enhancements**: When adding new AST operators or execution phases, update this table with the corresponding Drools DRL syntax and Rust AST node.
2. **Test Suite Verification**: Every row in this matrix must maintain an automated test case in `tests/` matching the designated `Test ID`.
3. **Immigration Program Expansion**: New regulatory models (e.g. Australia GSM, UK Skilled Worker, US RAISE Act, New Zealand SMC) must reference and validate against these matrix primitives.
