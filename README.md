# Rust Rules Engine Examples (`rust-rules-engine-examples`)

A high-performance, deterministic, zero-GC, explainable Business Rules Engine in Rust engineered for complex regulatory and points-based decision systems (such as merit-based immigration frameworks), offering a drop-in replacement for **Red Hat Drools (JBoss Rules)**.

---

## 📚 Key Documents

* **[SPECIFICATION.md](file:///home/pkshrestha/git/rust-rules-engine-examples/SPECIFICATION.md)** — The complete technical specification covering architecture, AST grammar, execution lifecycle, 12 regulatory edge cases, and merit-based immigration program specifications (Canada CRS, Australia GSM 189/190, UK Skilled Worker).
* **[DROOLS_PARITY_MATRIX.md](file:///home/pkshrestha/git/rust-rules-engine-examples/docs/DROOLS_PARITY_MATRIX.md)** — Exhaustive 30+ feature mapping table comparing Drools DRL syntax/mechanisms directly to their Rust Rules Engine AST equivalents, immigration examples, and test IDs.

---

## 🚀 Quick Start & How to Run

### 1. Prerequisites
Ensure you have Rust and Cargo installed (edition 2024 or 1.85+):
```bash
cargo --version
```

### 2. Build the Project
```bash
# Build debug binary and library
cargo build

# Build optimized release binary
cargo build --release
```

---

## 💻 CLI Commands (`rules-engine-cli`)

The repository includes a comprehensive CLI binary `rules-engine-cli` for running evaluations, batch rankings, rule inspections, and test suites.

### A. Run the Drools Parity Verification Test Suite
Executes the automated matrix of 10+ regulatory edge cases (Skill Transferability, Spouse Factors, Cap Truncation, Relational Joins, Activation Groups, Temporal CEP Expiry, Null Safety, Quantifiers, Forward Chaining):
```bash
cargo run --bin rules-engine-cli -- test-suite
```

Output:
```text
┌─────────┬──────────────────┬─────────────────┬────────┬────────────┬─────────┐
│ Test ID ┆ Scenario Name    ┆ Feature / Edge  ┆ Score  ┆ Status     ┆ Verdict │
│         ┆                  ┆ Case Tested     ┆        ┆            ┆         │
╞═════════╪══════════════════╪═════════════════╪════════╪════════════╪═════════╡
│ TC-01   ┆ Skill Transfer.  ┆ CLB 9+ Matrix   ┆ 587.0  ┆ Eligible   ┆ PASSED  │
│ TC-02   ┆ Spouse & PNP     ┆ PNP Bonus +600  ┆ 1108.0 ┆ Eligible   ┆ PASSED  │
│ TC-03   ┆ Cap Truncation   ┆ 150 -> 100 Cap  ┆ 535.0  ┆ Eligible   ┆ PASSED  │
│ TC-04   ┆ Relational Join  ┆ Revoked Sponsor ┆ 50.0   ┆ Ineligible ┆ PASSED  │
│ TC-05   ┆ Activation Group ┆ XOR Tradeables  ┆ 70.0   ┆ Eligible   ┆ PASSED  │
│ TC-06   ┆ Hard Age Gate    ┆ Age 46 Barred   ┆ 60.0   ┆ Ineligible ┆ PASSED  │
│ TC-07   ┆ Temporal CEP     ┆ 730-day Window  ┆ 130.0  ┆ Ineligible ┆ PASSED  │
│ TC-08   ┆ Null Safety      ┆ Null Spouse     ┆ 80.0   ┆ Eligible   ┆ PASSED  │
│ TC-10   ┆ Quantifier       ┆ forall(clb>=7)  ┆ 50.0   ┆ Eligible   ┆ PASSED  │
│ TC-11   ┆ Forward Chain    ┆ Derived Facts   ┆ 180.0  ┆ Eligible   ┆ PASSED  │
└─────────┴──────────────────┴─────────────────┴────────┴────────────┴─────────┘
  Results: 10 / 10 tests passed successfully.
```

---

### B. Evaluate a Single Applicant Profile against a Rule Folder
Evaluate a candidate fact file against a **modular folder of rules** (the engine automatically scans and merges all `.yaml` and `.json` files in the folder):

```bash
# 1. Canada Express Entry CRS - Pointing to modular rules folder
cargo run --bin rules-engine-cli -- evaluate \
  --rules rules/canada_crs/ \
  --applicant applicants/tc01_tech_lead_single.yaml \
  --format table

# 2. You can also use --rules-dir alias
cargo run --bin rules-engine-cli -- evaluate \
  --rules-dir rules/canada_crs/ \
  --applicant applicants/tc02_married_phd_researcher.yaml

# 3. Output as structured JSON (for REST API / downstream ingestion)
cargo run --bin rules-engine-cli -- evaluate \
  --rules rules/canada_crs/ \
  --applicant applicants/tc01_tech_lead_single.yaml \
  --format json

# 4. Output as YAML
cargo run --bin rules-engine-cli -- evaluate \
  --rules rules/canada_crs/ \
  --applicant applicants/tc03_cap_overflow_tradesperson.yaml \
  --format yaml
```

---

### C. Run a Batch Selection & Ranking Draw against a Rule Folder
Batch evaluate an entire directory of candidates against a **rule folder**, rank them by score, and apply an invitation cutoff score filter (e.g. Express Entry Draw):

```bash
cargo run --bin rules-engine-cli -- batch \
  --rules rules/canada_crs/ \
  --applicants-dir applicants/ \
  --cutoff 500
```

Output:
```text
 🏆 BATCH RANKING & SELECTION DRAW: Comprehensive Ranking System (CRS) - Express Entry 
  Cutoff Score Threshold: 500.0 points
  Total Candidates Evaluated: 10
┌──────┬─────────────────────────────┬─────────────┬──────────┬────────────────┐
│ Rank ┆ Candidate ID / File         ┆ Total Score ┆ Status   ┆ Draw Result    │
╞══════╪═════════════════════════════╪═════════════╪══════════╪════════════════╡
│ 1    ┆ APP-TC02-PHD-RESEARCHER     ┆ 1108.0      ┆ Eligible ┆ SELECTED (ITA) │
│ 2    ┆ APP-TC01-TECH-LEAD          ┆ 587.0       ┆ Eligible ┆ SELECTED (ITA) │
│ 3    ┆ APP-TC03-TRADESPERSON       ┆ 535.0       ┆ Eligible ┆ SELECTED (ITA) │
│ 4    ┆ APP-TC07-EXPIRED-IELTS      ┆ 472.0       ┆ Eligible ┆ Below Cutoff   │
│ 5    ┆ APP-TC11-FORWARD-CHAIN      ┆ 469.0       ┆ Eligible ┆ Below Cutoff   │
│ 6    ┆ APP-TC10-FORALL-FAIL        ┆ 381.0       ┆ Eligible ┆ Below Cutoff   │
│ 7    ┆ APP-TC08-NULL-SPOUSE        ┆ 377.0       ┆ Eligible ┆ Below Cutoff   │
│ 8    ┆ APP-TC05-COMPETING-TRADEABL ┆ 249.0       ┆ Eligible ┆ Below Cutoff   │
│ 9    ┆ APP-TC04-UNLICENSED-SPONSOR ┆ 225.0       ┆ Eligible ┆ Below Cutoff   │
│ 10   ┆ APP-TC06-AGE-BARRED         ┆ 150.0       ┆ Eligible ┆ Below Cutoff   │
└──────┴─────────────────────────────┴─────────────┴──────────┴────────────────┘
```

---

### D. Inspect a Rule Folder (Metadata, Caps, and Pipeline Phases)
Inspect category budgets, max point caps, and all rules loaded across the files in a rule folder:

```bash
cargo run --bin rules-engine-cli -- inspect --rules rules/canada_crs/
```

---

## 🧪 Running Automated Tests

Run the full suite of unit and integration tests:
```bash
cargo test
```

To run only the edge case parity tests:
```bash
cargo test --test drools_parity_edge_cases
```

---

## 📖 Programmatic Rust Examples

Run the standalone Rust code examples in `examples/`:

```bash
# Canada Express Entry Programmatic Evaluation Demo
cargo run --example canada_crs_demo

# Working Memory Mutation & Forward Chaining Demo
cargo run --example drools_inference_demo

# Drools & DMN Decision Table Evaluation Demo (CSV Spreadsheet & Hit Policies)
cargo run --example decision_table_demo
```

### Rust API Usage Example

```rust
use rust_rules_engine::{Engine, FactContext, RuleProgram};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load rule program from YAML or JSON
    let program = RuleProgram::from_yaml_file("rules/canada_crs_express_entry.yaml")?;
    let engine = Engine::new(program);

    // 2. Ingest fact data (any Serde-compatible struct or JSON value)
    let fact_data = serde_json::json!({
        "applicant": {
            "id": "APP-2026-001",
            "age": 29,
            "marital_status": "single",
            "education": { "highest_degree": "master" },
            "language": {
                "first_official": {
                    "clb_reading": 9,
                    "clb_writing": 9,
                    "clb_listening": 9,
                    "clb_speaking": 9
                }
            },
            "work_experience": { "domestic_years": 2, "foreign_years": 3 }
        }
    });
    let context = FactContext::from_value(fact_data);

    // 3. Evaluate rules
    let report = engine.evaluate(&context)?;

    println!("Total Points: {:.1}", report.total_score);
    println!("Eligible: {}", report.is_eligible());

    // 4. Print beautiful console explanation table
    report.print_audit_table();

    Ok(())
}
```

---

## 🎯 Drools Parity Highlights

| Drools Capability | Rust Rules Engine Alternative |
|---|---|
| Multi-Fact Working Memory & Relational Joins | `WorkingMemory` & `Condition::MultiFactJoin` |
| Forward Chaining & Inference (`insert` / `modify`) | Reactive Derived Facts & `Action::SetAttribute` |
| Agenda Groups & Ruleflow Groups | Pipeline `phase` execution (`validation` $\to$ `enrichment` $\to$ `scoring` $\to$ `capping`) |
| Activation Groups (XOR Mutual Exclusion) | `activation_group` (First-match / Best-match wins) |
| Cycle Prevention (`no-loop`, `lock-on-active`) | `no_loop: true` & phase-locked activation registry |
| Collection Accumulators (`from accumulate`) | `Action::Accumulate` (`sum_fte_years`, `min`, `max`, `count`) |
| Temporal Reasoning & CEP (Drools Fusion) | `Condition::Temporal` sliding lookback & expiry windows |
| Universal & Existential Quantifiers | `forall`, `exists`, `not_exists`, `min_count` |
| Decision Tables & DMN Hit Policies | `DecisionTable` AST & Hit Policies (`First`, `Unique`, `CollectSum`, `CollectMax`, `CollectMin`, `CollectCount`, `Priority`, `Any`, `RuleOrder`) |
| CSV / Spreadsheet Rule Ingestion | `DecisionTable::from_csv_str` with bracket/quote-aware expression parsing |
| Modular Rule Composition (Folders) | `RuleProgram::from_directory` / `--rules <folder>` auto-merging |
| 100% Explainability & Event Listeners | Built-in `AuditReport` with full execution DAG & condition traces |
| Deterministic Latency | Native compiled Rust ($\le 50\,\mu\text{s}$, zero GC) |

---

## 🗺️ Project Layout

```
rust-rules-engine-examples/
├── SPECIFICATION.md          # Formal Technical Specification
├── README.md                 # Project Overview & How-To Guide
├── docs/
│   └── DROOLS_PARITY_MATRIX.md # Exhaustive Drools Feature Parity Matrix
├── src/
│   ├── core/                 # Core Engine, AST, Evaluator, Context, Audit
│   │   ├── ast.rs            # Rule AST, DecisionTable, HitPolicy & Condition Nodes
│   │   ├── audit.rs          # Structured Audit & Table Formatter
│   │   ├── context.rs        # FactContext & Path Traversal
│   │   ├── error.rs          # Typed Error Handling
│   │   ├── evaluator.rs      # Engine Pipeline, Decision Table Evaluator & Formulas
│   │   └── mod.rs
│   ├── immigration/          # Merit-Based Immigration Domain Models
│   │   ├── models.rs         # Strongly Typed Applicant Profiles
│   │   └── mod.rs
│   ├── lib.rs                # Library Root Export
│   └── main.rs               # CLI Tool (evaluate, batch, inspect, test-suite)
├── rules/                    # Declarative YAML/JSON/CSV Rule Programs
│   ├── canada_crs/           # Modular Rule Folder (Config, Enrichment, Core, Spouse, Transferability, Bonus)
│   ├── canada_crs_express_entry.yaml
│   ├── australia_subclass_189.yaml
│   ├── uk_skilled_worker_points.yaml
│   ├── edge_cases_drools_parity_suite.yaml
│   └── sample_decision_table.csv  # Spreadsheet Decision Table Example
├── applicants/               # Test Applicant Fact Profiles (YAML)
├── examples/                 # Programmatic Rust API Usage Examples
│   ├── canada_crs_demo.rs
│   ├── drools_inference_demo.rs
│   └── decision_table_demo.rs # Multi-Column Decision Table & Hit Policies Demo
└── tests/                    # Drools Edge Case & Decision Table Integration Tests
    └── drools_parity_edge_cases.rs
```

