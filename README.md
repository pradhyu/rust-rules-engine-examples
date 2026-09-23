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
* **[DROOLS_PARITY_MATRIX.md](file:///home/pkshrestha/git/rust-rules-engine-examples/docs/DROOLS_PARITY_MATRIX.md)** — Comprehensive 30+ feature matrix classifying de facto built-in capabilities vs. advanced Drools features.
* **[SPRING_BOOT_MIGRATION_CONFIG_DI.md](file:///home/pkshrestha/git/rust-rules-engine-examples/docs/SPRING_BOOT_MIGRATION_CONFIG_DI.md)** — Architectural guide on migrating Spring Boot properties (`@Value`, `@ConfigurationProperties`) and DI (`@Autowired`) to idiomatic Rust.


---

## 🚀 KSD-CO/rust-rule-engine Showcase & Examples

This repository serves as a comprehensive, real-world examples suite for **[`KSD-CO/rust-rule-engine`](https://github.com/KSD-CO/rust-rule-engine)** (`rust-rule-engine = "1.21"`).

All upstream capabilities—**Grule Rule Language (GRL)** parsing, **RETE-UL forward chaining**, **goal-driven backward chaining**, and **streaming CEP time windows**—are demonstrated with fully runnable examples:

| Example | Command | Description | Key Features |
|---|---|---|---|
| **Quickstart** | `cargo run --example rust_rule_engine_quickstart` | Hello World & Working Memory | `RuleEngineBuilder`, `RustRuleEngine`, `Facts`, `Value` |
| **UK Points System** | `cargo run --example rust_rule_engine_uk_immigration` | Statutory UK Skilled Worker policy | GRL ingestion (`.grl`), `salience`, `activation-group` XOR, custom action handlers |
| **Canada CRS System** | `cargo run --example rust_rule_engine_canada_crs` | Express Entry 1200-pt system | Multi-phase forward-chaining, language mastery inference, skill transferability |
| **RETE-UL Network** | `cargo run --example rust_rule_engine_rete` | RETE-UL pattern matching | `IncrementalEngine`, `GrlReteLoader`, `TypedFacts`, alpha/beta memories |
| **Backward Chaining** | `cargo run --example rust_rule_engine_backward_chaining` | Goal-driven proof discovery | `BackwardEngine`, `query`, proof traces, hypothesis rejection |
| **Streaming CEP** | `cargo run --example rust_rule_engine_streaming` | Real-time event window processing | `TimeWindow` (sliding/tumbling), `StreamEvent`, burst fraud alerts |

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

### D. Interactive Terminal REPL Engine
Start an interactive real-time rule evaluation REPL shell:

```bash
cargo run --bin rules-engine-cli -- repl --rules rules/canada_crs/
```

Within the REPL:
- `:load <path|dir>` — Load and compile any rule file or modular directory
- `:eval applicants/tc01_tech_lead_single.json` — Real-time evaluation with colored audit table
- `:whatif applicants/tc01_tech_lead_single.json 485` — Run What-If pathway simulations
- `:stats 10000` — Measure median (p50), 95th (p95), and 99th (p99) latency percentiles
- `:inspect` — View category caps, pass threshold, and rules by phase

---

### E. Run Multi-Protocol Microservices (REST & gRPC)
Launch a unified high-throughput REST (HTTP/JSON on port 8080) and gRPC (HTTP/2 on port 50051) microservice:

```bash
cargo run --bin rules-engine-cli -- serve \
  --rules rules/canada_crs/ \
  --rest-port 8080 \
  --grpc-port 50051
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

## 📖 Programmatic Rust Examples & Client Demos

Run standalone examples and protocol benchmarks:

```bash
# 1. Drools Global Variables & Global Services Demo (Action Handlers, RulePlugin, GlobalsRegistry)
cargo run --example drools_globals_and_services

# 2. Multi-Protocol REST vs gRPC Real-Time Benchmark (1,000 requests)
cargo run --example protocol_benchmark_client

# 3. Real-Time REST JSON Client Demo
cargo run --example realtime_rest_client

# 4. Real-Time gRPC Protobuf Client Demo
cargo run --example realtime_grpc_client

# 5. Real-Time Candidate Advisor & What-If Pathway Simulations
cargo run --example realtime_advisor_demo

# 6. Working Memory Mutation & Forward Chaining Demo
cargo run --example drools_inference_demo
```

### Rust API Usage Example

```rust
use rust_rules_engine::{
    evaluate_facts, json_to_facts, load_knowledge_base_from_path,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load KnowledgeBase from native GRL rule file (via KSD-CO/rust-rule-engine)
    let (kb, categories) = load_knowledge_base_from_path("rules/uk_skilled_worker_points.grl")?;

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
    let facts = json_to_facts(&fact_data)?;

    // 3. Evaluate rules using upstream rust-rule-engine with itemized audit report
    let report = evaluate_facts(&kb, facts, &categories, 70.0)?;

    println!("Total Points: {:.1}", report.total_score);
    println!("Eligible: {}", report.is_eligible());

    // 4. Print beautiful console explanation table
    report.print_audit_table();

    Ok(())
}
```

---

## 🎯 Drools Parity Highlights

| Drools Capability | De Facto Status | KSD-CO/rust-rule-engine Implementation Note |
|---|:---:|---|
| Pattern Matching (LHS) | **Built-in** | Grule Rule Language (`when { ... }`) with full arithmetic and logical expressions |
| Dot-Path Traversal & Safe Nulls | **Built-in** | Non-panicking path resolution returning safe `Value::Null` / `Option` |
| Salience & Rule Priorities | **Built-in** | Upstream `salience <N>` execution ordering |
| Activation Groups (XOR Mutual) | **Built-in** | Upstream `activation-group "<group>"` for exclusive option competition |
| Forward Chaining Reactivity | **Built-in** | RETE-UL pattern matching and working memory mutation |
| Backward Chaining | **Built-in** | Upstream `BackwardEngine` for goal-driven proof discovery |
| Complex Event Processing (CEP) | **Built-in** | Upstream `TimeWindow` (sliding & tumbling windows) for real-time streaming |
| Modular Rule Loading | **Built-in** | `load_knowledge_base_from_path` auto-loads `.grl`, `.yaml`, or `.json` rule files |
| Explainability & Audit Traces | **Built-in** | Zero-allocation `AuditReport` with execution callbacks and itemized rule firing traces |

---

## 🗺️ Project Layout

```
rust-rules-engine-examples/
├── SPECIFICATION.md          # Formal Technical Specification
├── README.md                 # Project Overview & How-To Guide
├── docs/
│   ├── DROOLS_PARITY_MATRIX.md # Exhaustive Drools Feature Parity Matrix
│   └── LLM_RUST_PITFALLS.md    # Lessons Learned & Pitfalls for LLMs
├── proto/
│   └── rules_engine.proto    # gRPC & Protobuf Service Contract Definition
├── src/
│   ├── evaluator.rs          # APPLICATION: Audit Trail, Scoring Engine & Fact Mapper (backed by KSD-CO/rust-rule-engine)
│   ├── server/               # PLATFORM: Multi-Protocol Network Serving Layer
│   │   ├── dto.rs            # Separated Request & Response Data Transfer Objects
│   │   ├── rest.rs           # Axum HTTP/JSON REST API & Router
│   │   ├── grpc.rs           # Tonic HTTP/2 gRPC Service Implementation
│   │   └── mod.rs
│   ├── repl.rs               # PLATFORM: Interactive Terminal REPL Shell (via KSD-CO/rust-rule-engine)
│   ├── lib.rs                # Library Root Export (re-exports KSD-CO/rust-rule-engine)
│   └── main.rs               # CLI Tool (evaluate, batch, inspect, repl, serve)
├── rules/                    # Declarative GRL Rule Programs (parsed by KSD-CO/rust-rule-engine)
│   ├── canada_crs_express_entry.grl    # Canada Express Entry CRS Scoring & Skill Transferability
│   ├── uk_skilled_worker_points.grl    # Statutory UK Skilled Worker Points with XOR Activation Groups
│   ├── drools_globals_and_services.grl # Drools Global Variables & Global Services Demo Rules
│   └── drools_parity_suite.grl         # 1:1 GRL Specification for Advanced Drools Features
├── applicants/               # Test Applicant Fact Profiles (YAML & JSON)
│   ├── tc01_tech_lead_single.yaml
│   ├── tc01_tech_lead_single.json
│   ├── tc04_uk_sponsor_unlicensed.yaml
│   ├── tc05_uk_competing_tradeable.yaml
│   └── tc14_uk_stem_phd_discounted.yaml
├── examples/                 # Programmatic Rust API & Protocol Client/Server Demos
│   ├── drools_globals_and_services.rs        # Drools Global Services (Action Handlers, RulePlugin, GlobalsRegistry)
│   ├── drools_inference_demo.rs              # Working Memory Mutation & Forward Chaining
│   ├── protocol_benchmark_client.rs          # REST vs gRPC Latency & Throughput Benchmark
│   ├── realtime_advisor_demo.rs              # Real-Time Candidate Advisor & What-If Simulations
│   ├── realtime_grpc_client.rs               # gRPC Client (Evaluate, Batch, What-If)
│   ├── realtime_grpc_server.rs               # Standalone Tonic gRPC Server
│   ├── realtime_rest_client.rs               # REST Client (Evaluate, Batch, What-If)
│   ├── realtime_rest_server.rs               # Standalone Axum REST Server
│   ├── rust_rule_engine_quickstart.rs        # Quickstart: Builder & Facts
│   ├── rust_rule_engine_uk_immigration.rs    # UK Skilled Worker Statutory Evaluation
│   ├── rust_rule_engine_canada_crs.rs        # Canada CRS Forward Chaining & Inference
│   ├── rust_rule_engine_rete.rs              # RETE-UL Incremental Pattern Matching Network
│   ├── rust_rule_engine_backward_chaining.rs # Goal-Driven Backward Chaining Discovery
│   └── rust_rule_engine_streaming.rs         # CEP Real-Time Sliding Time Windows
└── tests/                    # Integration & Parity Verification Tests
    ├── drools_parity_edge_cases.rs   # Edge case verification across regulatory policies
    ├── grl_loading_tests.rs          # Verification of GRL parsing and execution
    └── test_upstream_ksd_engine.rs   # Upstream engine parser and execution tests
```

