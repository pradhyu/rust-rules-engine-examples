# Drools Feature Parity & De Facto Implementation Matrix

> **Core Project Objective**: This repository provides **real-world regulatory examples** (such as Canada Express Entry CRS, Australia GSM 189, and UK Skilled Worker) demonstrating **de facto business rules engine patterns**.
> 
> For enterprise teams evaluating a migration from **Red Hat Drools (JBoss Rules / KIE)**, this matrix explicitly documents what is supported as a **de facto / built-in primitive** versus advanced Drools subsystems where **it can be done, but requires writing custom application logic or extensions**.

---

## 1. Summary Feature Classification Overview

```
+-------------------------------------------------------------------------------------------------------------+
|                                        DROOLS vs RUST ENGINE CLASSIFICATION                                 |
+------------------------------------+-----------------------------+------------------------------------------+
| Drools Subsystem                   | Parity Classification       | Architectural Guidance                   |
+------------------------------------+-----------------------------+------------------------------------------+
| 1. Condition LHS & Pattern Match   | De Facto (Built-in)         | Direct JSON/YAML AST path predicates     |
| 2. Salience & Priority Sorting     | De Facto (Built-in)         | Sort rules by integer priority           |
| 3. Rule Composition & Modular Files| De Facto (Built-in)         | Load & merge rule directories            |
| 4. Logical Quantifiers (Exists/Not)| De Facto (Built-in)         | Boolean trees & presence operators       |
| 5. Action Execution (Points/Flags) | De Facto (Built-in)         | Atomic state actions                     |
| 6. Universal Quantifier (`forall`) | Requires Custom Logic       | Can be done, but you have to write it    |
| 7. Multi-Fact Relational Joins     | Requires Custom Logic       | Can be done, but you have to write it    |
| 8. Temporal CEP / Sliding Windows  | Requires Custom Logic       | Can be done, but you have to write it    |
| 9. Collection Accumulators         | Requires Custom Logic       | Can be done, but you have to write it    |
| 10. Forward Chaining (Working Mem) | Requires Custom Logic       | Can be done, but you have to write it    |
| 11. Truth Maintenance System (TMS) | Requires Custom Logic       | Can be done, but you have to write it    |
| 12. Activation Groups (XOR Mutual) | Requires Custom Logic       | Can be done, but you have to write it    |
| 13. Decision Tables (Spreadsheets) | Requires Custom Logic       | Can be done, but you have to write it    |
| 14. Subcategory Capping & Budgets  | Requires Custom Logic       | Can be done, but you have to write it    |
+------------------------------------+-----------------------------+------------------------------------------+
```

---

## 2. Exhaustive Drools Feature Mapping & Implementation Notes

### A. Pattern Matching, Constraints & Left-Hand-Side (LHS)

| # | Drools (DRL) Feature | Drools DRL Syntax Example | Parity Status | Implementation Note | Example Use Case |
|---|---|---|:---:|---|---|
| **A1** | **Field Equality / Inequality** | `Applicant(maritalStatus == "single")` | **De Facto (Built-in)** | Native path comparison (`op: eq` / `neq`) | Single vs married candidate branch |
| **A2** | **Numeric Comparisons** | `Applicant(age >= 18 && age <= 29)` | **De Facto (Built-in)** | Native numeric range (`op: gte`, `lte`, `between`) | Peak age scoring bracket |
| **A3** | **Set Membership (`in` / `not in`)** | `Education(degree in ("master", "doctorate"))` | **De Facto (Built-in)** | Native array membership (`op: in`, `not_in`) | Qualifying higher degree pathways |
| **A4** | **Null-Safe Navigation (`!.` / `?.`)** | `Applicant(spouse != null, spouse!.education != null)` | **De Facto (Built-in)** | Three-valued logic: non-existent paths return `None` safely | Single applicants with null spouse |
| **A5** | **Regular Expression Matching (`matches`)** | `JobOffer(nocCode matches "^21[0-9]{3}$")` | **De Facto (Built-in)** | Native regex evaluation (`op: matches_regex`) | TEER / NOC STEM code matching |
| **A6** | **Collection Membership (`contains`)** | `Applicant(tags contains "stem_priority")` | **De Facto (Built-in)** | Native collection contains (`op: contains`) | Specialized candidate stream flags |
| **A7** | **Variable Bindings** | `$age : age, $edu : education` | **De Facto (Built-in)** | Dotted path traversal (`applicant.age`) | Multi-field cross-referencing |
| **A8** | **Multi-Fact Relational Joins** | `JobOffer($id: sponsorId) and Sponsor(id == $id, isAccredited == true)` | **Requires Custom Logic** | *Can be done, but you have to write cross-fact join resolution.* | Validating job offer against sponsor DB |
| **A9** | **Existential Quantifier (`exists`)** | `exists JobOffer(isLmiaApproved == true)` | **De Facto (Built-in)** | Condition presence check (`type: exists`) | LMIA labor market approval presence |
| **A10** | **Negative Quantifier (`not` / `not exists`)** | `not IneligibilityFlag()` | **De Facto (Built-in)** | Condition negation (`type: not` / `none`) | Absence of criminality / security bars |
| **A11** | **Universal Quantifier (`forall` / `every`)** | `forall(LanguageBand($score: clb) LanguageBand(clb >= 7))` | **Requires Custom Logic** | *Can be done, but you have to write collection-level predicate iterators.* | Enforcing minimum CLB 7 in all 4 language abilities |
| **A12** | **Threshold Quantifier (`min_count`)** | Custom accumulate count in Drools | **Requires Custom Logic** | *Can be done, but you have to write threshold counting evaluators.* | Passing at least 3 out of 5 regional tie criteria |

---

### B. Aggregations, Accumulators & Collection Operations

| # | Drools Feature | Drools DRL Syntax | Parity Status | Implementation Note | Example Use Case |
|---|---|---|:---:|---|---|
| **B1** | **Sum Accumulator (`accumulate sum`)** | `accumulate(Employment($m: months); $total: sum($m))` | **Requires Custom Logic** | *Can be done, but you have to write collection reducers.* | Summing months across multiple jobs |
| **B2** | **Normalized FTE Sum (Custom Accumulator)** | `accumulate(Employment($m: months, $h: hrs); $fte: sum(...))` | **Requires Custom Logic** | *Can be done, but you have to write custom domain reduction formulas.* | Normalizing part-time Canadian experience to FTE |
| **B3** | **Min / Max Accumulator** | `accumulate(LanguageBand($s: score); $minScore: min($s))` | **Requires Custom Logic** | *Can be done, but you have to write min/max collection reducers.* | Finding lowest language sub-score for baseline CLB |
| **B4** | **Count Accumulator** | `accumulate(TradeCertification(); $count: count())` | **Requires Custom Logic** | *Can be done, but you have to write array length/filter count logic.* | Counting qualifying trade certificates |
| **B5** | **Filtered Collection (`from collect`)** | `ArrayList() from collect(Employment(isDomestic == true))` | **Requires Custom Logic** | *Can be done, but you have to write collection filter predicates.* | Isolating domestic employment records |

---

### C. Temporal Reasoning & Complex Event Processing (Drools Fusion / CEP)

| # | Drools Fusion Feature | Drools CEP Syntax | Parity Status | Implementation Note | Example Use Case |
|---|---|---|:---:|---|---|
| **C1** | **Sliding Recency Window (`within_past`)** | `LanguageTest(this after[0d, 730d] applicationDate)` | **Requires Custom Logic** | *Can be done, but you have to write timestamp delta validation.* | 2-year validity window on IELTS/CELPIP test results |
| **C2** | **Lookback Horizon Window** | `Employment(endDate after[0d, 3650d] applicationDate)` | **Requires Custom Logic** | *Can be done, but you have to write date horizon boundary filters.* | 10-year lookback window for qualifying foreign work |
| **C3** | **Duration / Overlap Checks (`overlaps`, `during`)** | `JobA() and JobB(this overlaps JobA)` | **Requires Custom Logic** | *Can be done, but you have to write interval intersection arithmetic.* | Preventing double-counting of concurrent jobs |
| **C4** | **Point-in-Time Age Freezing** | Java date calculation in DRL | **Requires Custom Logic** | *Can be done, but you have to write date-of-birth locking logic.* | Freezing applicant age at ITA / lock-in date |

---

### D. Consequence Execution, Working Memory & Right-Hand-Side (RHS)

| # | Drools RHS Feature | Drools DRL Syntax | Parity Status | Implementation Note | Example Use Case |
|---|---|---|:---:|---|---|
| **D1** | **Fact Insertion (`insert`)** | `insert(new DerivedFact("clb9_plus"));` | **Requires Custom Logic** | *Can be done, but you have to write working memory mutation & agenda reactivity.* | Forward chaining: asserting high language badge |
| **D2** | **Logical Assertion (TMS / `insertLogical`)** | `insertLogical(new EligibleForStream("stem"));` | **Requires Custom Logic** | *Can be done, but you have to write a fact truth maintenance dependency graph.* | Asserting temporary qualifications that auto-retract |
| **D3** | **Fact Modification (`modify` / `update`)** | `modify($applicant) { setScore($applicant.getScore() + 50) };` | **De Facto (Built-in)** | Native score mutation action (`Action::AwardPoints`) | Awarding points to score category budget |
| **D4** | **Fact Retraction (`delete` / `retract`)** | `delete($temporaryFact);` | **Requires Custom Logic** | *Can be done, but you have to write attribute removal in context.* | Clearing temporary flags between evaluation stages |
| **D5** | **Early Exit / Halt (`drools.halt()`)** | `drools.halt();` | **De Facto (Built-in)** | Eligibility gate flag halts pipeline on failure | Disqualifying candidate on hard prerequisite failure |
| **D6** | **Diagnostic Tagging / Channels** | `channels["audit"].send("Qualified for PNP");` | **De Facto (Built-in)** | Tagging action (`Action::AddTag`) | Tagging candidate for specialized category draw rounds |

---

### E. Rule Scheduling, Conflict Resolution & Agenda Control

| # | Drools Agenda Attribute | Drools DRL Attribute | Parity Status | Implementation Note | Example Use Case |
|---|---|---|:---:|---|---|
| **E1** | **Salience (Rule Priority)** | `salience 1000` | **De Facto (Built-in)** | Native integer priority sort (`priority: 1000`) | Hard gates (1000) run before scoring (500) |
| **E2** | **Agenda Groups / Ruleflow Groups** | `agenda-group "validation"` | **Requires Custom Logic** | *Can be done, but you have to write a staged pipeline executor.* | Strict multi-stage evaluation pipeline |
| **E3** | **Activation Groups (XOR Mutual Exclusion)** | `activation-group "age_bracket"` | **Requires Custom Logic** | *Can be done, but you have to write group activation state tracking.* | Discrete age brackets: only 1 rule in group fires |
| **E4** | **Cycle Prevention (`no-loop`)** | `no-loop true` | **Requires Custom Logic** | *Can be done, but you have to write activation history tracking.* | Preventing self-reactivation loops on context mutation |
| **E5** | **Group Cycle Prevention (`lock-on-active`)** | `lock-on-active true` | **Requires Custom Logic** | *Can be done, but you have to write phase-locked activation registries.* | Freezing rule activation within active phase |
| **E6** | **Rule Effective / Expiry Dates** | `date-effective "01-Jan-2026"` | **Requires Custom Logic** | *Can be done, but you have to write date-range rule filters.* | Sunsetting regulatory policy versions |
| **E7** | **Rule Disabling (`enabled`)** | `enabled false` | **De Facto (Built-in)** | Boolean enabled flag (`enabled: false`) | Temporarily pausing specific immigration streams |

---

### F. Decision Tables, DMN & Scorecards

| # | Drools / DMN Feature | Drools Equivalent | Parity Status | Implementation Note | Example Use Case |
|---|---|---|:---:|---|---|
| **F1** | **Multi-Column Decision Table** | DMN Decision Table / Drools Table | **Requires Custom Logic** | *Can be done, but you have to write multi-column table evaluation.* | Skill Transferability Matrix (Degree $\times$ Foreign Work $\times$ CLB) |
| **F2** | **CSV / Spreadsheet Decision Tables** | Drools `.xls` / `.csv` Tables | **Requires Custom Logic** | *Can be done, but you have to write spreadsheet parsing & expression matching.* | Rule spreadsheets maintained by policy analysts |
| **F3** | **Hit Policy: Unique (U)** | DMN Unique (`U`) | **Requires Custom Logic** | *Can be done, but you have to write single-match validation.* | Exactly one discrete salary / tier bracket |
| **F4** | **Hit Policy: First (F)** | DMN First (`F`) | **Requires Custom Logic** | *Can be done, but you have to write priority row short-circuiting.* | Highest qualification priority cascade |
| **F5** | **Hit Policy: Collect Sum (C+)** | DMN Collect Sum (`C+`) | **Requires Custom Logic** | *Can be done, but you have to write row value summation.* | Aggregating multiple sub-factor points in table |
| **F6** | **Hit Policy: Collect Max (C>)** | DMN Collect Max (`C>`) | **Requires Custom Logic** | *Can be done, but you have to write maximum output aggregation.* | Best-case points across concurrent pathways |
| **F7** | **Hit Policy: Collect Min (C<)** | DMN Collect Min (`C<`) | **Requires Custom Logic** | *Can be done, but you have to write minimum output aggregation.* | Conservative scoring across concurrent pathways |
| **F8** | **Hit Policy: Collect Count (C#)**| DMN Collect Count (`C#`)| **Requires Custom Logic** | *Can be done, but you have to write matching row count reducers.* | Counting qualifying criteria rows met |
| **F9** | **2D Matrix Lookup** | DMN Boxed Decision Matrix | **De Facto (Built-in)** | Key-value matrix lookup (`PointsFormula::MatrixLookup`) | 2D matrix lookup (Education $\times$ Language Band) |
| **F10** | **Linear Scorecard with Clamps** | Drools PMML Scorecard | **De Facto (Built-in)** | Scaled multiplier with min/max clamp (`PointsFormula::Scaled`) | Overseas work experience (5 pts/yr, max 15 pts) |
| **F11** | **Discrete Map Lookup** | Drools Simple Map Table | **De Facto (Built-in)** | Key-value map lookup (`PointsFormula::Lookup`) | Degree hierarchy scoring table |

---

### G. Auditability, Tracing & Operational Characteristics

| # | Drools Feature | Drools Mechanism | Parity Status | Implementation Note | Architectural Advantage |
|---|---|---|:---:|---|---|
| **G1** | **Agenda Event Listeners** | `AgendaEventListener` callbacks | **Requires Custom Logic** | *Can be done, but you have to write structured audit event collectors.* | Complete itemized legal trace for visa decisions |
| **G2** | **Working Memory Audit** | `WorkingMemoryEventListener` | **Requires Custom Logic** | *Can be done, but you have to write context mutation changelogs.* | Proof of factual derivation history |
| **G3** | **Subcategory Point Caps** | Complex manual accumulator rules in DRL | **Requires Custom Logic** | *Can be done, but you have to write category-level budget capping.* | Guaranteed regulatory compliance with program caps |
| **G4** | **Deterministic Latency** | JVM JIT & GC pauses (10ms – 100ms jitter) | **Native Advantage** | Native compiled Rust ($\le 50\,\mu\text{s}$, zero GC) | Sub-millisecond high-volume batch evaluation |
| **G5** | **Thread Safety & Concurrency** | Synchronized `KieSession` or pooling | **Native Advantage** | Immutable `RuleProgram` (`Send + Sync`) + lock-free parallel execution | Real-time batch visa draw ranking simulations |
