# Rust Rules Engine Examples (`rust-rules-engine-examples`)

A high-performance, deterministic, zero-GC, explainable Business Rules Engine in Rust engineered for complex regulatory and points-based decision systems (such as merit-based immigration frameworks), offering a drop-in replacement for **Red Hat Drools (JBoss Rules)**.

---

## 📚 Key Documents

* **[SPECIFICATION.md](file:///home/pkshrestha/git/rust-rules-engine-examples/SPECIFICATION.md)** — The complete technical specification covering architecture, AST grammar, execution lifecycle, 12 regulatory edge cases, and merit-based immigration program specifications (Canada CRS, Australia GSM 189/190, UK Skilled Worker).
* **[DROOLS_PARITY_MATRIX.md](file:///home/pkshrestha/git/rust-rules-engine-examples/docs/DROOLS_PARITY_MATRIX.md)** — Exhaustive 30+ feature mapping table comparing Drools DRL syntax/mechanisms directly to their Rust Rules Engine AST equivalents, immigration examples, and test IDs.

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
| Decision Tables & DMN Matrix Scoring | `PointsFormula::MatrixLookup` & `PointsFormula::Lookup` |
| 100% Explainability & Event Listeners | Built-in `AuditReport` with full execution DAG & condition traces |
| Deterministic Latency | Native compiled Rust ($\le 50\,\mu\text{s}$, zero GC) |

---

## 🗺️ Project Layout

```
rust-rules-engine-examples/
├── SPECIFICATION.md          # Formal Technical Specification
├── README.md                 # Project Overview & Quickstart
├── docs/
│   └── DROOLS_PARITY_MATRIX.md # Comprehensive Drools Feature Parity Matrix
├── src/
│   ├── core/                 # Core Engine, AST, Evaluator, Context, Audit
│   ├── immigration/          # Merit-Based Immigration Data Models & Programs
│   ├── lib.rs                # Library Root Export
│   └── main.rs               # CLI Tool (evaluate, batch, inspect)
├── rules/                    # Declarative YAML/JSON Rule Programs
│   ├── canada_crs_express_entry.yaml
│   ├── australia_subclass_189.yaml
│   └── uk_skilled_worker_points.yaml
├── applicants/               # Test Applicant Fact Profiles (YAML)
├── examples/                 # Programmatic Rust API Usage Examples
└── tests/                    # 12 Drools Edge Case Verification Tests
```
