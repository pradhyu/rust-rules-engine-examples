# Enterprise Specification: High-Performance Rust Rules Engine
## *Architectural Equivalence with Drools (JBoss Rules) & Comprehensive Merit-Based Immigration Test Suite*

**Document Version:** 2.0.0-enterprise-draft  
**Authors:** DeepMind Pair Programming / Pradhyumna Shrestha  
**Project:** `rust-rules-engine-examples`  
**Target:** Production-Grade Drop-In Replacement for Drools in High-Throughput / Complex Regulatory Decision Systems  

---

## 1. Executive Summary & Drools Parity Objectives

Enterprise Business Rule Management Systems (BRMS) like **Red Hat Drools (JBoss Rules)** are widely deployed for mission-critical decisioning in finance, insurance, and public administration. However, JVM-based Drools engines introduce significant latency overhead, heavy memory footprints, complex classloaders, and Garbage Collection pauses that hinder modern low-latency, cloud-native, or embedded environments.

This specification defines a **high-performance, deterministic, zero-GC, explainable Rules Engine in Rust** (`rust_rules_engine`) engineered to support the full spectrum of enterprise rule capabilities provided by Drools, while maintaining 100% auditability and legal explainability.

### 1.1. Core Architectural Benchmarks & Parity Matrix

| Drools (JBoss Rules) Feature | Rust Rules Engine Primitive | Enterprise Immigration Use Case |
|---|---|---|
| **Multi-Fact Working Memory & Cross-Fact Joins** | `WorkingMemory` & `FactQuery` joins across entity sets | Joining `Applicant`, multiple `EmploymentPeriod`s, `LanguageTest`, `JobOffer`, and `Sponsor` |
| **Forward Chaining & Inference (`insert` / `modify`)** | Derived Facts & Reactive Working Memory Mutation | Rule derives `applicant.is_stem_specialist = true`, triggering a downstream priority category rule |
| **Salience / Priority Ordering** | `priority` (signed integer) | Hard eligibility gate rules (priority 1000) fire before core scoring (500) before bonus aggregations (100) |
| **Agenda Groups & Ruleflow Groups** | `phase` / `agenda_group` execution stages | Strict pipeline execution: `validation_gate` $\to$ `fact_enrichment` $\to$ `scoring` $\to$ `capping` $\to$ `verdict` |
| **Activation Groups / XOR Mutual Exclusion** | `activation_group` (First-match / Best-match wins) | Age bracket rules: only the single matching bracket rule fires, canceling other bracket evaluations |
| **Cycle Prevention (`no-loop`, `lock-on-active`)** | In-flight rule activation tracking | Modifying applicant fact attributes does not re-trigger the modifying rule into an infinite loop |
| **Quantifiers (`exists`, `not exists`, `forall`)** | `exists`, `not_exists`, `forall`, `min_count` | "All 4 language abilities $\ge$ CLB 7" (`forall`), "Has at least 1 job offer with approved LMIA" (`exists`) |
| **Accumulators & Aggregations (`from accumulate`)** | `accumulate` (`sum`, `count`, `min`, `max`, `collect`) | Summing total continuous qualifying work months across distinct jobs, calculating minimum CLB score |
| **Temporal Reasoning & Sliding Windows (CEP / Fusion)** | `temporal` operators (`within_last`, `before`, `after`) | "Language test must be taken within 24 months of application date", "10-year lookback for work experience" |
| **Three-Valued Logic & Null-Safe Navigation (`?.`)** | `null_safe` path resolution & `Option<Value>` | Gracefully handling missing spouse, omitted job offers, or null test scores without panics |
| **Decision Tables & Hit Policies (DMN / Drools DT)** | Multi-Dimensional Matrix Rules (`HitPolicy`) | Multi-variable cross-factor lookup: Education $\times$ First Language CLB $\times$ Domestic Experience |
| **Audit Trail & Agenda Event Listeners** | `AuditReport` with full execution DAG & condition trace | Comprehensive itemized justification showing passed/failed conditions, actual vs expected values, and points |
> [!NOTE]
> For the complete, exhaustive 30+ feature reference mapping (including DRL syntax examples, AST representations, and migration guidelines), see **[DROOLS_PARITY_MATRIX.md](file:///home/pkshrestha/git/rust-rules-engine-examples/docs/DROOLS_PARITY_MATRIX.md)**.

---


## 2. System Architecture & Working Memory Model

```
+----------------------------------------------------------------------------------------------------+
|                                      WORKING MEMORY (FACT STORE)                                   |
|                                                                                                    |
|  +---------------------+   +---------------------+   +---------------------+   +----------------+  |
|  |   Applicant Fact    |   | Employment Facts[N] |   | LanguageTest Facts  |   | JobOffer Fact  |  |
|  | (Age, Status, Educ) |   | (NOC, Dates, Hours) |   | (IELTS/TEF Bands)   |   | (Salary, LMIA) |  |
|  +----------+----------+   +----------+----------+   +----------+----------+   +-------+--------+  |
+-------------|-------------------------|-------------------------|----------------------|-----------+
              |                         |                         |                      |
              +-------------------------+------------+------------+----------------------+
                                                     |
                                                     v
+----------------------------------------------------------------------------------------------------+
|                                    RULE ENGINE EXECUTION PIPELINE                                  |
|                                                                                                    |
|   PHASE 1: VALIDATION & ELIGIBILITY GATES (Salience 1000+)                                         |
|   - Hard constraints (Age ceiling, Identity verification, Minimum language floor)                 |
|   - Rejection / Ineligibility early flags                                                          |
|                                                    |                                               |
|                                                    v                                               |
|   PHASE 2: FACT ENRICHMENT & INFERENCE (Salience 800 - 500)                                        |
|   - Working Memory Mutations (`insert_derived_fact`, `set_attribute`)                              |
|   - Temporal normalization (calculate exact age at application date, continuous work months)       |
|   - Forward chaining triggers subsequent rule activations                                          |
|                                                    |                                               |
|                                                    v                                               |
|   PHASE 3: SCORING & DECISION MATRICES (Salience 400 - 100)                                        |
|   - Core Human Capital, Spouse Factors, Skill Transferability                                      |
|   - Multi-fact joins & Accumulations (`forall`, `accumulate.sum`)                                  |
|   - Activation Groups (XOR mutual exclusion for discrete tiers)                                    |
|                                                    |                                               |
|                                                    v                                               |
|   PHASE 4: CATEGORY CAPPING & AGGREGATION (Salience 50 - 10)                                       |
|   - Subcategory caps (e.g. max 100 on skill transferability)                                       |
|   - Global points cap (e.g. max 1200 on CRS)                                                       |
|                                                    |                                               |
|                                                    v                                               |
|   PHASE 5: AUDIT TRAIL GENERATION & VERDICT (Salience 0)                                           |
|   - Generate complete execution trace DAG & explainability report                                  |
+----------------------------------------------------|-----------------------------------------------+
                                                     |
                                                     v
+----------------------------------------------------------------------------------------------------+
|                                           AUDIT & EXPLAINABILITY                                   |
|                                                                                                    |
|  +-------------------------+   +-------------------------------+   +----------------------------+  |
|  | Total Score & Pass Gate |   | Subcategory Breakdown & Caps  |   | Itemized Fired Rules Trace |  |
|  +-------------------------+   +-------------------------------+   +----------------------------+  |
+----------------------------------------------------------------------------------------------------+
```

---

## 3. Comprehensive Edge Cases & Test Scenarios

To ensure `rust_rules_engine` completely covers enterprise Drools capabilities, the engine is specified against **12 Complex Regulatory Edge Cases**:

---

### Edge Case 1: Multi-Fact Joins & Relational Pattern Matching
* **Drools Pattern:** `JobOffer($sponsorId: sponsorId, salary >= 38700) and Sponsor(id == $sponsorId, isAccredited == true, status == "active")`
* **Regulatory Context (UK Skilled Worker / Canada LMIA):** A job offer only awards points if the associated employer sponsor fact exists in working memory, has an active accreditation license, and the offered salary meets or exceeds the specific occupation threshold.
* **Failure Modes Tested:**
  * Job offer exists, but sponsor fact is missing in working memory $\to$ Condition Fails.
  * Sponsor exists but `is_accredited = false` $\to$ Condition Fails.
  * Sponsor is valid and salary qualifies $\to$ Rule Fires.

---

### Edge Case 2: Multi-Row Collection Accumulation & Overlap Resolution
* **Drools Pattern:** `accumulate(EmploymentPeriod(isDomestic == true, $hrs: weeklyHours, $months: durationMonths); $totalYears: sum($months * ($hrs >= 30 ? 1.0 : $hrs / 30.0) / 12.0))`
* **Regulatory Context (Canada Canadian Experience Class / Express Entry):** Work experience consists of a list of separate employment records. Part-time work must be normalized to full-time equivalence (30 hrs/week = 1.0 FTE). Overlapping employment dates during the same calendar month cannot be double-counted.
* **Failure Modes Tested:**
  * 3 distinct jobs of 6 months each at 15 hrs/week (0.5 FTE) = $3 \times 6 \times 0.5 = 9$ FTE months (qualifies for $< 1$ year $\to$ 0 pts).
  * 2 concurrent jobs of 30 hrs/week in the same 12-month period capped at 12 full-time months (cannot count as 2 years).
  * Minimum 12 full-time continuous months required for baseline qualification.

---

### Edge Case 3: Temporal Reasoning & Sliding Lookback Windows (CEP)
* **Drools Fusion Pattern:** `LanguageTest(this after[0d, 730d] applicationDate)` and `EmploymentPeriod(endDate after[0d, 3650d] applicationDate)`
* **Regulatory Context (General Immigration Standards):**
  * Language test results (IELTS/CELPIP) expire strictly **2 years (730 days)** after the test date.
  * Work experience is only valid if performed within the **10-year lookback window** preceding the lock-in submission date.
* **Failure Modes Tested:**
  * Language test taken 731 days before application date $\to$ Flagged as expired, 0 points, fails mandatory language gate.
  * Work experience completed 11 years ago $\to$ Filtered out by temporal window accumulator.

---

### Edge Case 4: Activation Groups & XOR Mutual Exclusion
* **Drools Pattern:** `rule "Age_Bracket_20_29" activation-group "age-points" ... rule "Age_Bracket_30_34" activation-group "age-points" ...`
* **Regulatory Context (Age Points & Competing Pathways):** In a points system, an applicant can only match exactly **one** discrete age bracket. Furthermore, if an applicant qualifies under both the *Trade Certificate Stream* (50 pts) and the *Higher Education Stream* (50 pts), but the program stipulates mutual exclusivity, only the highest priority activation must fire.
* **Failure Modes Tested:**
  * Multiple age condition rules matching $\to$ Exactly one rule executes; others in the activation group are canceled and recorded as `deactivated_by_activation_group`.

---

### Edge Case 5: Forward Chaining, Inference & Attribute Enrichment
* **Drools Pattern:** 
  * *Rule 1 (Inference):* When `LanguageTest(clbReading >= 9, clbWriting >= 9, clbListening >= 9, clbSpeaking >= 9)` $\to$ `insert(new HighLanguageProficiencyFact(applicantId))`
  * *Rule 2 (Downstream):* When `PostSecondaryEducation(degree == "master")` and `exists HighLanguageProficiencyFact()` $\to$ `awardPoints(50, "Skill Transferability")`
* **Regulatory Context (Skill Transferability):** Derived facts or enriched attributes dynamically trigger secondary downstream rules without requiring repetitive compound conditions.
* **Failure Modes Tested:**
  * Rule 1 fires and inserts derived fact $\to$ Rule 2 successfully matches the newly asserted fact in the same execution cycle.
  * Cycle prevention (`no-loop`): Rule 1 does not re-fire upon its own fact insertion.

---

### Edge Case 6: Three-Valued Logic & Null-Safe Traversal
* **Drools Pattern:** `Applicant(spouse == null || spouse.languageTest?.clbReading >= 5)`
* **Regulatory Context (Single vs Married Applicants):**
  * Single applicants have `spouse: null`.
  * Married applicants have `spouse: { ... }`, but spouse language tests or education credentials may be optional/null.
* **Failure Modes Tested:**
  * Single applicant evaluated against spouse rules $\to$ Handled gracefully without `NullPointerException` / panic, evaluating to `false` or skipping.
  * Dotted lookup on deeply nested missing attributes (e.g., `applicant.spouse.education.is_eca_verified`) yields `FactNotFound` or `null` gracefully.

---

### Edge Case 7: Universal Quantifiers (`forall` / `every`) & Threshold Bands
* **Drools Pattern:** `forall(LanguageAbility($score: clbScore) LanguageAbility(clbScore >= 7))`
* **Regulatory Context (CLB Threshold Gates):** A program mandates that an applicant must achieve at least CLB 7 in **all 4 abilities** (Reading, Writing, Listening, Speaking). A single score of CLB 6 with three CLB 10s fails the universal quantifier.
* **Failure Modes Tested:**
  * Scores: R: 10, W: 10, L: 10, S: 6 $\to$ `forall(clb >= 7)` evaluates to `false`.
  * Scores: R: 7, W: 7, L: 7, S: 7 $\to$ `forall(clb >= 7)` evaluates to `true`.

---

### Edge Case 8: Multi-Dimensional Matrix Decision Tables with Discontinuous Scales
* **Drools Decision Table (DMN Boxed Expression):**
  | Education Level | First Official Language CLB | Canadian Work Exp | Points Awarded |
  |---|---|---|---|
  | Master / PhD | CLB 9+ in all 4 | 0 years | 50 |
  | Master / PhD | CLB 7 or 8 in all 4 | 0 years | 25 |
  | Bachelor | CLB 9+ in all 4 | 0 years | 25 |
  | Master / PhD | N/A | 1+ years Canadian | 50 |
* **Regulatory Context (Canada CRS Matrix):** Points are evaluated non-linearly against a 2D/3D matrix table with fallback default scores.
* **Failure Modes Tested:**
  * Exact coordinate hit $\to$ Correct score returned.
  * Partial coordinate hit or missing band $\to$ Evaluates to matrix fallback/default.

---

### Edge Case 9: Cascading Subcategory Capping & Global Boundary Limits
* **Regulatory Context:**
  * Category A (Human Capital): Capped at 500.
  * Category B (Spouse): Capped at 40.
  * Category C (Skill Transferability): Composed of 5 sub-rules each awarding 50 points (theoretical max 250), but strictly capped at **100 points**.
  * Total Score: Capped at **1,200 points**.
* **Failure Modes Tested:**
  * Applicant earns 50 pts on Sub-rule C1, 50 pts on C2, and 50 pts on C3 (Raw total: 150) $\to$ Category C capped at 100 points, 50 points truncated, audit log reflects exact truncation.

---

### Edge Case 10: Hard Mandatory Eligibility Gates vs Soft Scoring Penalties
* **Regulatory Context:** 
  * Age 45+ in Australia Subclass 189 is an **absolute hard gate rejection** (`eligible = false`).
  * In Canada CRS, Age 45+ awards 0 points for age, but the applicant remains eligible if other criteria compensate.
* **Failure Modes Tested:**
  * Hard gate failure flags applicant as `eligible = false`, records legal blocking reason, and optionally halts further scoring to save compute.
  * Soft gate awards 0 points but maintains `eligible = true`.

---

### Edge Case 11: Truth Maintenance System (TMS) & Fact Invalidation
* **Drools Pattern:** Logical assertions (`insertLogical`) where derived facts retract if the condition ceases to hold.
* **Regulatory Context:** In dynamic simulations or interactive recalculations (e.g. changing an IELTS score from 8 to 5), all derived facts (`is_high_language_proficient`) and dependent rule activations are automatically invalidated and recalculated.

---

### Edge Case 12: Deterministic Agenda Salience & Phase Execution Pipeline
* **Regulatory Context:** Rules must execute in strict deterministic order across multi-threaded or distributed workers.
* **Failure Modes Tested:**
  * Rules within the same phase execute strictly in descending `priority` order.
  * Ties in priority are resolved deterministically using unique `rule_id` lexicographical sorting.

---

## 4. Extended Rule Language Grammar & AST

To satisfy all enterprise edge cases, the declarative AST is extended with primitives for multi-fact joins, accumulators, temporal windows, and agenda management:

```
RuleProgram
 ├── ID, Name, Version, Description
 ├── Categories: Map<CategoryID, CategoryConfig (max_points, cap_policy)>
 ├── GlobalCaps: TotalPointsCap, PassMarkThreshold
 └── Rules: List<RuleDefinition>
      ├── ID: String
      ├── Name: String
      ├── Phase / AgendaGroup: String ("validation" | "enrichment" | "scoring" | "capping")
      ├── Priority / Salience: i32
      ├── ActivationGroup: Option<String> (XOR mutual exclusion)
      ├── NoLoop: bool (Cycle prevention)
      ├── IsEligibilityGate: bool (Hard gate flag)
      ├── Condition: ConditionNode
      │    ├── Compare (Path, Operator, Value)
      │    ├── Logical (All, Any, None, Not, MinCount)
      │    ├── MultiFactJoin (FactTypes, JoinConditions)
      │    ├── Quantifier (Forall, Exists, NotExists)
      │    ├── Accumulate (SourceArray, FilterCondition, Function, TargetVar)
      │    ├── Temporal (FieldPath, ReferenceDatePath, WindowDuration, Direction)
      │    └── MatrixLookup (RowPath, ColPath, TableID, Fallback)
      └── Actions: List<ActionNode>
           ├── AwardPoints (Category, Formula, Reason)
           ├── SetEligibility (Eligible: bool, Reason: String)
           ├── InsertDerivedFact (FactType, Data)
           ├── MutateContext (Path, Value)
           └── AddAuditTag (Tag: String)
```

---

## 5. Formal YAML Declarative Specification Examples

### 5.1. Canada CRS Express Entry (Full Enterprise Spec)

```yaml
id: "canada_crs_express_entry"
name: "Comprehensive Ranking System (CRS) - Express Entry"
version: "2026.2"
total_points_cap: 1200
pass_mark_threshold: null

categories:
  core_human_capital:
    name: "core_human_capital"
    display_name: "Core / Human Capital Factors"
    max_points: 500
  spouse_factors:
    name: "spouse_factors"
    display_name: "Spouse / Common-Law Partner Factors"
    max_points: 40
  skill_transferability:
    name: "skill_transferability"
    display_name: "Skill Transferability Factors"
    max_points: 100
  additional_points:
    name: "additional_points"
    display_name: "Additional Factors"
    max_points: 600

rules:
  # =========================================================================
  # PHASE 1: FACT ENRICHMENT & INFERENCE (Salience 800)
  # =========================================================================
  - id: "infer_high_language_status"
    name: "Infer High First Language Status (CLB 9+ in all 4)"
    phase: "enrichment"
    priority: 800
    no_loop: true
    condition:
      type: "all"
      conditions:
        - type: "compare"
          path: "applicant.language.first_official.clb_reading"
          op: "gte"
          value: 9
        - type: "compare"
          path: "applicant.language.first_official.clb_writing"
          op: "gte"
          value: 9
        - type: "compare"
          path: "applicant.language.first_official.clb_listening"
          op: "gte"
          value: 9
        - type: "compare"
          path: "applicant.language.first_official.clb_speaking"
          op: "gte"
          value: 9
    actions:
      - type: "set_attribute"
        key: "derived.has_initial_clb_9_plus"
        value: true
      - type: "add_tag"
        tag: "high_language_proficient"

  - id: "accumulate_continuous_canadian_experience"
    name: "Accumulate Full-Time Equivalent Canadian Experience"
    phase: "enrichment"
    priority: 750
    condition:
      type: "exists"
      path: "applicant.employment_history"
    actions:
      - type: "accumulate"
        source_array: "applicant.employment_history"
        filter:
          type: "all"
          conditions:
            - type: "compare"
              path: "is_domestic_canadian"
              op: "eq"
              value: true
            - type: "temporal"
              field_path: "end_date"
              reference_date_path: "application.submission_date"
              window_days: 3650 # 10-year lookback
              direction: "within_past"
        function: "sum_fte_years"
        target_attribute: "derived.calculated_domestic_years"

  # =========================================================================
  # PHASE 2: SCORING - CORE HUMAN CAPITAL (Salience 500)
  # =========================================================================
  - id: "crs_core_age_single"
    name: "Age Points (Single Applicant)"
    phase: "scoring"
    priority: 500
    activation_group: "age_scoring"
    condition:
      type: "all"
      conditions:
        - type: "compare"
          path: "applicant.marital_status"
          op: "eq"
          value: "single"
    actions:
      - type: "award_points"
        category: "core_human_capital"
        formula:
          type: "lookup"
          path: "applicant.age"
          mapping:
            "18": 99
            "19": 105
            "20": 110
            "21": 110
            "22": 110
            "23": 110
            "24": 110
            "25": 110
            "26": 110
            "27": 110
            "28": 110
            "29": 110
            "30": 105
            "31": 99
            "32": 94
            "33": 88
            "34": 83
            "35": 77
            "36": 72
            "37": 66
            "38": 61
            "39": 55
            "40": 50
            "41": 39
            "42": 28
            "43": 17
            "44": 6
          default: 0
        reason: "Core Human Capital age points for single candidate."

  - id: "crs_core_education_single"
    name: "Education Points (Single Applicant)"
    phase: "scoring"
    priority: 500
    condition:
      type: "all"
      conditions:
        - type: "compare"
          path: "applicant.marital_status"
          op: "eq"
          value: "single"
    actions:
      - type: "award_points"
        category: "core_human_capital"
        formula:
          type: "lookup"
          path: "applicant.education.highest_degree"
          mapping:
            "doctorate": 150
            "master": 135
            "two_or_more": 128
            "bachelor": 120
            "three_year_post": 112
            "two_year_post": 98
            "one_year_post": 90
            "secondary": 30
          default: 0
        reason: "Core Human Capital education level points."

  - id: "crs_first_language_single"
    name: "First Official Language Points (Single Applicant)"
    phase: "scoring"
    priority: 500
    condition:
      type: "all"
      conditions:
        - type: "compare"
          path: "applicant.marital_status"
          op: "eq"
          value: "single"
    actions:
      - type: "award_points"
        category: "core_human_capital"
        formula:
          type: "language_clb_band"
          reading_path: "applicant.language.first_official.clb_reading"
          writing_path: "applicant.language.first_official.clb_writing"
          listening_path: "applicant.language.first_official.clb_listening"
          speaking_path: "applicant.language.first_official.clb_speaking"
          band_scores:
            "10": 34
            "9": 31
            "8": 23
            "7": 17
            "6": 9
            "5": 6
            "4": 6
        reason: "Per-ability first language CLB scores."

  # =========================================================================
  # PHASE 3: SCORING - SKILL TRANSFERABILITY COMBINATIONS (Salience 400)
  # =========================================================================
  - id: "crs_transferability_education_clb9"
    name: "Skill Transferability: Education + High Language (CLB 9+)"
    phase: "scoring"
    priority: 400
    condition:
      type: "all"
      conditions:
        - type: "compare"
          path: "applicant.education.highest_degree"
          op: "in"
          value: ["two_or_more", "master", "doctorate"]
        - type: "compare"
          path: "computed.has_initial_clb_9_plus"
          op: "eq"
          value: true
    actions:
      - type: "award_points"
        category: "skill_transferability"
        formula:
          type: "fixed"
          points: 50
        reason: "Post-secondary degree combined with CLB 9+ across all language abilities."

  - id: "crs_transferability_foreign_work_clb9"
    name: "Skill Transferability: Foreign Work Experience + High Language (CLB 9+)"
    phase: "scoring"
    priority: 400
    condition:
      type: "all"
      conditions:
        - type: "compare"
          path: "applicant.work_experience.foreign_years"
          op: "gte"
          value: 3
        - type: "compare"
          path: "computed.has_initial_clb_9_plus"
          op: "eq"
          value: true
    actions:
      - type: "award_points"
        category: "skill_transferability"
        formula:
          type: "fixed"
          points: 50
        reason: "3+ years foreign work experience combined with CLB 9+."

  # =========================================================================
  # PHASE 4: ADDITIONAL BONUS FACTORS (Salience 300)
  # =========================================================================
  - id: "crs_bonus_provincial_nomination"
    name: "Provincial Nomination Bonus (+600)"
    phase: "scoring"
    priority: 300
    condition:
      type: "compare"
      path: "applicant.additional_factors.provincial_nomination"
      op: "eq"
      value: true
    actions:
      - type: "award_points"
        category: "additional_points"
        formula:
          type: "fixed"
          points: 600
        reason: "Valid provincial nomination certificate."

  - id: "crs_bonus_french_bilingual"
    name: "French Language Bilingual Bonus (+50)"
    phase: "scoring"
    priority: 300
    condition:
      type: "all"
      conditions:
        - type: "compare"
          path: "applicant.language.second_official.clb_reading"
          op: "gte"
          value: 7
        - type: "compare"
          path: "applicant.language.second_official.clb_writing"
          op: "gte"
          value: 7
        - type: "compare"
          path: "applicant.language.second_official.clb_listening"
          op: "gte"
          value: 7
        - type: "compare"
          path: "applicant.language.second_official.clb_speaking"
          op: "gte"
          value: 7
        - type: "compare"
          path: "applicant.language.first_official.clb_reading"
          op: "gte"
          value: 5
    actions:
      - type: "award_points"
        category: "additional_points"
        formula:
          type: "fixed"
          points: 50
        reason: "NCLC 7+ in French and CLB 5+ in English."
```

---

### 5.2. UK Skilled Worker Points System (Relational Joins & Threshold Gates)

```yaml
id: "uk_skilled_worker_points"
name: "UK Skilled Worker Points-Based System"
version: "2026.1"
pass_mark_threshold: 70

categories:
  mandatory_criteria:
    name: "mandatory_criteria"
    display_name: "Mandatory Non-Tradeable Criteria"
    max_points: 50
  tradeable_criteria:
    name: "tradeable_criteria"
    display_name: "Tradeable Points Options"
    max_points: 20

rules:
  # =========================================================================
  # PHASE 1: MANDATORY CRITERIA (50 Points Required)
  # =========================================================================
  - id: "uk_sponsor_job_offer_join"
    name: "Job Offer from Licensed Sponsor"
    phase: "scoring"
    priority: 1000
    is_eligibility_gate: true
    condition:
      type: "multi_fact_join"
      facts:
        - name: "job"
          path: "applicant.job_offer"
        - name: "sponsor"
          path: "reference_data.sponsors"
      join_conditions:
        - "job.sponsor_license_number eq sponsor.license_number"
        - "sponsor.is_active eq true"
        - "sponsor.rating eq 'A_rated'"
    actions:
      - type: "award_points"
        category: "mandatory_criteria"
        formula: { type: "fixed", points: 20 }
        reason: "Valid job offer from active A-rated licensed Home Office sponsor."

  - id: "uk_skill_level_rqf3"
    name: "Job at Appropriate Skill Level (RQF 3 or higher)"
    phase: "scoring"
    priority: 950
    is_eligibility_gate: true
    condition:
      type: "compare"
      path: "applicant.job_offer.rqf_skill_level"
      op: "gte"
      value: 3
    actions:
      - type: "award_points"
        category: "mandatory_criteria"
        formula: { type: "fixed", points: 20 }
        reason: "Job meets minimum RQF 3 skill requirement."

  - id: "uk_english_b1_gate"
    name: "English Language Proficiency at CEFR Level B1"
    phase: "scoring"
    priority: 900
    is_eligibility_gate: true
    condition:
      type: "compare"
      path: "applicant.language.cefr_level"
      op: "in"
      value: ["B1", "B2", "C1", "C2"]
    actions:
      - type: "award_points"
        category: "mandatory_criteria"
        formula: { type: "fixed", points: 10 }
        reason: "Demonstrated English proficiency at CEFR B1 or higher."

  # =========================================================================
  # PHASE 2: TRADEABLE OPTIONS (20 Points Required - Activation Group)
  # =========================================================================
  - id: "uk_tradeable_option_a_salary"
    name: "Tradeable Option A: Salary >= £38,700 and Going Rate"
    phase: "scoring"
    priority: 500
    activation_group: "uk_tradeable_points"
    condition:
      type: "all"
      conditions:
        - type: "compare"
          path: "applicant.job_offer.annual_salary"
          op: "gte"
          value: 38700.00
        - type: "compare"
          path: "applicant.job_offer.meets_occupation_going_rate"
          op: "eq"
          value: true
    actions:
      - type: "award_points"
        category: "tradeable_criteria"
        formula: { type: "fixed", points: 20 }
        reason: "Option A: Salary exceeds £38,700 and meets occupation going rate."

  - id: "uk_tradeable_option_b_stem_phd"
    name: "Tradeable Option B: Relevant STEM PhD and Salary >= £30,960"
    phase: "scoring"
    priority: 450
    activation_group: "uk_tradeable_points"
    condition:
      type: "all"
      conditions:
        - type: "compare"
          path: "applicant.education.highest_degree"
          op: "eq"
          value: "doctorate"
        - type: "compare"
          path: "applicant.education.is_stem"
          op: "eq"
          value: true
        - type: "compare"
          path: "applicant.job_offer.annual_salary"
          op: "gte"
          value: 30960.00
    actions:
      - type: "award_points"
        category: "tradeable_criteria"
        formula: { type: "fixed", points: 20 }
        reason: "Option B: STEM PhD holder meeting reduced £30,960 threshold."

  - id: "uk_tradeable_option_c_shortage_occupation"
    name: "Tradeable Option C: Immigration Salary List Job and Salary >= £30,960"
    phase: "scoring"
    priority: 400
    activation_group: "uk_tradeable_points"
    condition:
      type: "all"
      conditions:
        - type: "compare"
          path: "applicant.job_offer.is_on_immigration_salary_list"
          op: "eq"
          value: true
        - type: "compare"
          path: "applicant.job_offer.annual_salary"
          op: "gte"
          value: 30960.00
    actions:
      - type: "award_points"
        category: "tradeable_criteria"
        formula: { type: "fixed", points: 20 }
        reason: "Option C: Occupation listed on Immigration Salary List."
```

---

## 6. Verification Test Suite & Edge Case Matrix

The test suite validates engine correctness across all 12 edge cases against expected outputs:

| Test Case ID | Edge Case Tested | Input Scenario Summary | Expected Verdict | Expected Score | Key Audit Assertion |
|---|---|---|---|---|---|
| `TC-01-CRS-TECH` | Skill Transferability & Language Bonus | Single Tech Lead (Age 29, Master's, CLB 9, 2yr CA Exp, 4yr Foreign Exp, Job Offer) | **Eligible** | **584 / 1200** | Transferability capped at 100/100; Job offer $+50$; Age $+110$. |
| `TC-02-CRS-MARRIED-PHD` | Spouse Factors & Provincial Nomination | Married PhD Candidate (Age 32, PhD, CLB 10, Spouse Bachelor's + CLB 8, PNP nomination) | **Eligible** | **1044 / 1200** | PNP $+600$; Spouse education $+10$; Spouse language $+20$. |
| `TC-03-CRS-EXP-CAP` | Subcategory Cap Overflow | Tradesperson with 5 transferability sub-rules qualifying (Raw score 150) | **Eligible** | **470 / 1200** | Raw transferability 150 clamped strictly to 100 max points. |
| `TC-04-UK-SPONSOR-JOIN` | Multi-Fact Relational Join | Job offer provided, but employer license number missing from A-rated register | **Ineligible** | **0 / 70** | Hard gate fails on `uk_sponsor_job_offer_join`; 0 mandatory points. |
| `TC-05-UK-STEM-TRADEABLE` | Activation Group Selection | Applicant meets both Option A (£42k salary) and Option B (STEM PhD) | **Eligible** | **70 / 70** | Option A fires (Priority 500); Option B canceled by activation group. |
| `TC-06-AUS-AGE-GATE` | Hard Gate vs Soft Gate | Applicant aged 46 submitting Australia GSM 189 Expression of Interest | **Ineligible** | **0 / 65** | Immediate rejection on Australian 45+ age eligibility gate. |
| `TC-07-TEMPORAL-EXPIRATION` | Temporal CEP Window | Candidate with perfect IELTS scores, but test date was 25 months ago | **Ineligible** | **0** | Language test rejected by `temporal(window_days: 730)` lookback constraint. |
| `TC-08-NULL-SAFE-SPOUSE` | Three-Valued Logic | Single candidate profile evaluated against rules referencing `spouse.*` paths | **Eligible** | Normal score | Evaluator returns `FactNotFound` safely without panic; skips spouse rules. |
| `TC-09-FTE-ACCUMULATOR` | Collection Accumulation | Candidate with 3 concurrent part-time jobs summing to 0.9 FTE years | **Eligible / No Exp Pts** | Normal score | Accumulator yields 0.9 yrs ($< 1.0$ yr cutoff); correctly awards 0 points. |
| `TC-10-FORALL-QUANTIFIER` | Universal Quantifier | Candidate has CLB 10, 10, 10, but 6 in Speaking for CLB 7+ requirement | **Fails Rule** | 0 bonus pts | `forall` evaluates to `false` due to single sub-threshold score. |
| `TC-11-FORWARD-CHAINING` | Derived Fact Inference | Rule 1 sets `has_initial_clb_9_plus = true`, enabling Rule 2 transferability | **Eligible** | $+50$ pts awarded | Working memory mutation in phase 1 triggers rule in phase 3. |
| `TC-12-MUTUAL-EXCLUSION` | Priority Ordering | Competing scoring paths with same salience resolved via lexicographical ID | **Eligible** | Deterministic | Identical score and execution trace across 10,000 parallel test iterations. |

---

## 7. Performance & Resource Guarantees

1. **Zero Garbage Collection (Deterministic Latency)**: Sub-millisecond evaluation latency ($\le 50\,\mu\text{s}$ per applicant profile across 100+ rules).
2. **Concurrent Safety (`Send + Sync`)**: RulePrograms and FactContexts are immutable during evaluation runs, allowing lock-free parallel batch evaluation across thousands of applicant threads.
3. **Memory Footprint**: Minimal heap allocation during evaluation, suitable for embedded or serverless AWS Lambda / Cloud Run execution.
4. **Serialization Parity**: 100% JSON Schema and YAML serialization support for integration with front-end rule visualizers and enterprise workflows.
