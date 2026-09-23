# Rust Rules Engine Examples (`rust-rules-engine-examples`)

An educational reference project showcasing **idiomatic Rust best practices** for building and consuming business rules engines, demonstrating how to model complex real-world regulatory domains (merit-based immigration policies) and evaluate architectural pathways for **migrating away from Red Hat Drools (JBoss Rules / KIE)**.

---

## 🎯 Why This Project Exists

1. **Learning Idiomatic Rust for Business Logic**: Demonstrates standard Rust layout, strongly-typed Serde fact contexts, decoupled AST evaluation pipelines, modular directory-based rule ingestion, and zero-allocation audit trails.
2. **Evaluating Drools Migration Feasibility**: Provides an honest, side-by-side feature matrix showing what can be expressed natively in Rust vs. what requires custom application logic.
3. **Real-World Domain Complexity**: Replaces toy examples with actual legal frameworks:
   - **Canada Express Entry CRS** (1,200-point multi-phase scoring with subcategory capping and transferability matrices)
   - **Australia GSM Subclass 189** (Hard age gates and points thresholds)
   - **UK Skilled Worker Points** (Mandatory prerequisites + XOR tradeable salary/PhD options)

---

## ⚖️ Moving from Drools to Rust: Key Advantages

| Dimension | Drools (JVM / KIE) | Rust Architecture |
|---|---|---|
| **Execution Latency** | 10ms – 100ms (JIT warm-up + GC pauses) | **$\le 50\,\mu\text{s}$ deterministic** (native machine code, zero GC) |
| **Memory Overhead** | 500MB – 2GB+ JVM heap per worker | **< 10MB** memory footprint |
| **Thread Safety & Scaling** | Synchronized `KieSession` or object pooling | **Immutable `&RuleProgram` (`Send + Sync`)**; parallel evaluation across all CPU cores |
| **Rule Format** | Java-centric `.drl` files requiring compilation | **Declarative YAML/JSON rule folders**; readable by policy analysts, versioned in Git |

---

## 📚 Key Documents

* **[SPECIFICATION.md](file:///home/pkshrestha/git/rust-rules-engine-examples/SPECIFICATION.md)** — The complete technical specification covering AST grammar, multi-phase execution lifecycles, and domain rule designs.
* **[DROOLS_PARITY_MATRIX.md](file:///home/pkshrestha/git/rust-rules-engine-examples/docs/DROOLS_PARITY_MATRIX.md)** — Comprehensive 30+ feature matrix classifying de facto built-in capabilities vs. advanced Drools features requiring custom extensions.


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

The repository includes a clean, production-grade CLI binary `rules-engine-cli` for evaluating profiles, batch rankings, and inspecting rule programs.

### A. Evaluate a Single Applicant Profile against a Rule Folder
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

### B. Run a Batch Selection & Ranking Draw against a Rule Folder
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

### C. Inspect a Rule Folder (Metadata, Caps, and Pipeline Phases)
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

# Real-Time Candidate Advisor & What-If Pathway Simulations
cargo run --example realtime_advisor_demo

# Multi-Format Ingestion (YAML, JSON, GRL) Demo
cargo run --example rule_formats_demo
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

| Drools Capability | De Facto Status | Rust Rules Engine Implementation Note |
|---|:---:|---|
| Pattern Matching (LHS) | **Built-in** | `Condition::Compare` with operators (`eq`, `neq`, `gte`, `lte`, `between`, `in`, `matches_regex`) |
| Dot-Path Traversal & Safe Nulls | **Built-in** | Non-panicking path resolution returning safe `Option` values (three-valued logic) |
| Salience & Rule Priorities | **Built-in** | Rules sorted and executed in descending priority order |
| Modular Rule Folders | **Built-in** | `RuleProgram::from_directory` auto-merges fragmented rules from directories |
| 2D Score Matrix & Lookups | **Built-in** | `PointsFormula::MatrixLookup` & `PointsFormula::Lookup` |
| Explainability & Audit Traces | **Built-in** | Zero-allocation `AuditReport` with itemized rule firing traces |
| Multi-Fact Relational Joins | *Custom* | *Can be done, but you have to write cross-fact join logic* |
| Temporal CEP Sliding Windows | *Custom* | *Can be done, but you have to write timestamp delta validation* |
| Collection Accumulators (`sum`, `fte`) | *Custom* | *Can be done, but you have to write custom reducer actions* |
| Forward Chaining Reactivity | *Custom* | *Can be done, but you have to write an agenda dependency tracker* |
| Activation Groups (XOR Mutual) | *Custom* | *Can be done, but you have to write activation group state tracking* |
| Spreadsheet Decision Tables | *Custom* | *Can be done, but you have to write spreadsheet parser & hit policy evaluators* |

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
│   │   ├── ast.rs            # Rule AST, Action & Condition Nodes
│   │   ├── audit.rs          # Structured Audit & Table Formatter
│   │   ├── context.rs        # FactContext & Path Traversal
│   │   ├── error.rs          # Typed Error Handling
│   │   ├── evaluator.rs      # Engine Pipeline & Formula Evaluators
│   │   └── mod.rs
│   ├── immigration/          # Merit-Based Immigration Domain Models
│   │   ├── models.rs         # Strongly Typed Applicant Profiles
│   │   └── mod.rs
│   ├── lib.rs                # Library Root Export
│   └── main.rs               # CLI Tool (evaluate, batch, inspect, test-suite)
├── rules/                    # Declarative YAML/JSON/GRL Rule Programs
│   ├── canada_crs/           # Modular Rule Folder (Config, Enrichment, Core, Spouse, Transferability, Bonus)
│   ├── canada_crs_express_entry.yaml
│   ├── canada_crs_express_entry.grl
│   ├── australia_subclass_189.yaml
│   ├── uk_skilled_worker_points.yaml
│   ├── uk_skilled_worker_points.json
│   ├── uk_skilled_worker_points.grl
│   └── edge_cases_drools_parity_suite.yaml
├── applicants/               # Test Applicant Fact Profiles (YAML & JSON)
│   ├── tc01_tech_lead_single.yaml
│   └── tc01_tech_lead_single.json
├── examples/                 # Programmatic Rust API Usage Examples
│   ├── canada_crs_demo.rs
│   ├── drools_inference_demo.rs
│   ├── realtime_advisor_demo.rs # Real-Time Candidate Advisor & What-If Simulations
│   └── rule_formats_demo.rs     # Multi-Format Ingestion (YAML, JSON, GRL)
└── tests/                    # Drools Edge Case Verification Integration Tests
    └── drools_parity_edge_cases.rs
```

