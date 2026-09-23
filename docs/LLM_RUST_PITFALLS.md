# LLM Code Generation: Rust Pitfalls & Rule Format Analysis

> A practical guide for generating Rust code (and rule definitions) with LLMs,
> based on real issues encountered while building the rust-rules-engine-examples project.

---

## Part 1: Why Not DRL/GRL Natively?

### The Architecture Decision

The rust-rules-engine uses **YAML/JSON as its executable format** rather than parsing a
DRL/GRL text grammar. Here's why:

| Approach | Pros | Cons |
|----------|------|------|
| **YAML/JSON AST** (current) | Zero-cost parsing via serde; schema-validated at deserde time; trivially extensible; no grammar maintenance | Verbose; deeply nested; easy to get field names wrong |
| **DRL/GRL parser** | Compact; familiar to Drools users; closer to human intent | Requires a full lexer/parser (pest, nom, or LALRPOP); grammar bugs; harder to extend; parser is a liability |

The engine chose YAML because **serde gives you a parser for free**. Adding a new condition
type is just adding a variant to an enum — no grammar file updates, no parser debugging.

### What About the `.grl` Files?

The `.grl` files in this project are **documentation-only**. They show Drools engineers
"here's what your DRL would look like" as a reference. The engine never reads them.

If you wanted actual DRL parsing, you'd need to add a `RuleProgram::from_drl_str()` that
uses a parser combinator crate (e.g., `pest` or `winnow`) to tokenize DRL syntax and
produce the same `RuleProgram` AST. This is ~500-1000 lines of parser code that would
need ongoing maintenance.

---

## Part 2: YAML vs DRL/GRL — Which is Better for LLM Generation?

### Tokenization Analysis

**YAML (current engine format):**
```yaml
- type: compare
  path: computed.overstay_count
  op: eq
  value: 0
```
- ~20 tokens for a single comparison
- Indentation-sensitive (LLMs frequently mess up YAML indentation)
- Field names must match serde exactly (`op` not `operator`, `path` not `field`)
- Deeply nested structures multiply token count exponentially

**DRL/GRL equivalent:**
```
computed.overstay_count == 0
```
- ~5 tokens for the same comparison
- No indentation sensitivity
- More natural language-like (LLMs handle this better)
- **~4x more token-efficient**

### Real-World LLM Failure Modes with YAML Rules

These are actual errors from this project's LLM-assisted development:

| Error | What Happened | Frequency |
|-------|---------------|-----------|
| `max_points_cap` vs `max_points` | LLM invented a plausible but wrong field name | Very common |
| `is_eligible` vs `eligible` | Wrong field name for SetEligibility | Common |
| `points: 75` + `formula: fixed` | Flat structure instead of nested `formula: { type: fixed, points: 75 }` | Very common |
| YAML indentation errors | Filter condition at wrong nesting level | Common |
| Missing serde tag `type:` field | Omitted `type: compare` discriminator | Occasional |
| `display_name` in category but missing `name` | Partial field population | Common |

### Verdict: DRL Would Be Better for LLM Generation

If the primary rule authorship workflow is "LLM generates rules," then a **DRL/GRL DSL
would be significantly better** than YAML because:

1. **Fewer tokens** = cheaper, faster, less context window consumed
2. **Less structural boilerplate** = fewer places for LLMs to make mistakes
3. **More training data** = DRL/Drools has extensive documentation in LLM training corpora
4. **Natural language proximity** = "when X then Y" maps cleanly to LLM reasoning

However, YAML is better for **programmatic rule generation** (API-driven, UI-driven)
because it's trivially serializable from any language.

### Recommendation

A hybrid approach would be ideal:
- Keep YAML/JSON as the **internal AST format** (what the engine executes)
- Add a **DRL parser** (`RuleProgram::from_drl_str()`) for human/LLM authorship
- Add a **DRL emitter** (`RuleProgram::to_drl_string()`) for human-readable output
- LLMs generate DRL → parser converts to AST → engine evaluates

---

## Part 3: Rust-Specific LLM Code Generation Pitfalls

### Critical: Ownership & Borrowing

LLMs frequently generate code that doesn't compile due to Rust's ownership model.

**Common failure — partial move after borrow:**
```rust
// ❌ LLM generates this:
let eligible = report.is_eligible(); // borrows report
let scores = report.category_scores; // MOVES report — compile error!

// ✅ Correct:
let eligible = report.is_eligible(); // borrow first
let scores = report.category_scores; // move second is fine since is_eligible() returned
```

**Common failure — moved value in loop:**
```rust
// ❌ LLM generates this:
for item in vec {
    process(item);  // moves item
    println!("{}", item.name);  // ERROR: item already moved
}

// ✅ Correct:
for item in &vec {
    process(item);
    println!("{}", item.name);
}
```

### Critical: Serde Tagged Enum Formats

This project uses `#[serde(tag = "type")]` internally-tagged enums. LLMs consistently
get the serialization format wrong.

**The Rust definition:**
```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    AwardPoints { category: String, formula: PointsFormula, reason: String },
    SetEligibility { eligible: bool, reason: String },
}
```

**What LLMs generate (wrong):**
```yaml
actions:
  - AwardPoints:
      category: compliance
      points: 50
```

**What's actually needed:**
```yaml
actions:
  - type: award_points
    category: compliance
    formula:
      type: fixed
      points: 50
```

Key gotchas:
- The `type` field is the discriminator (from `#[serde(tag = "type")]`)
- `rename_all = "snake_case"` means `AwardPoints` → `award_points`
- Nested tagged enums (like `PointsFormula` inside `Action`) require their own `type` field

### High: Lifetime Annotations

LLMs often omit or incorrectly specify lifetimes:
```rust
// ❌ LLM generates:
fn get_name(report: &AuditReport) -> &str { &report.program_name }

// This works for simple cases but LLMs struggle with:
// ❌
fn find_rule<'a>(rules: &'a [Rule], fired: &[FiredRuleRecord]) -> Option<&'a Rule> {
    // LLMs often put 'a on the wrong parameter
}
```

### High: Async Runtime Conflicts

```rust
// ❌ LLM generates:
#[tokio::main]
async fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap(); // PANIC: nested runtime!
}

// ✅ Correct — don't create a Runtime inside #[tokio::main]:
#[tokio::main]
async fn main() {
    my_async_function().await;
}
```

### Medium: Feature Flag Amnesia

LLMs forget that Rust crates require explicit feature flags:
```toml
# ❌ LLM adds the dependency but forgets features:
tokio = "1"

# ✅ Correct:
tokio = { version = "1", features = ["full"] }

# ❌ LLM uses reqwest::Client but forgets:
reqwest = "0.12"

# ✅ Correct:
reqwest = { version = "0.12", features = ["json"] }
```

### Medium: Edition-Specific Syntax

Rust 2024 edition enables `let-else`, `if-let chains`, and other syntax that LLMs
trained on older Rust may not generate correctly:
```rust
// Rust 2024 allows this:
if let Some(ref group) = rule.activation_group
    && fired_groups.contains(group)
{
    continue;
}

// Older editions need:
if let Some(ref group) = rule.activation_group {
    if fired_groups.contains(group) {
        continue;
    }
}
```

### Medium: Clippy Compliance

LLMs generate code that works but triggers clippy warnings:
```rust
// ❌ Collapsible if:
if condition_a {
    if condition_b {
        do_thing();
    }
}

// ✅ Clippy wants:
if condition_a && condition_b {
    do_thing();
}

// ❌ Manual min/max:
if score > max { score = max; }

// ✅ Clippy wants:
score = score.min(max);
```

### Medium: Type Conversion Traps

```rust
// ❌ LLM assumes implicit conversion:
let count: usize = some_json_value.as_i64().unwrap(); // ERROR: i64 != usize

// ✅ Correct:
let count = some_json_value.as_i64().unwrap() as usize;

// ❌ LLM forgets f64 precision:
let a: f64 = 0.1 + 0.2;
assert_eq!(a, 0.3); // FAILS!

// ✅ Correct:
assert!((a - 0.3).abs() < f64::EPSILON);
```

### Low: Error Handling Style

LLMs inconsistently use `unwrap()`, `expect()`, `?`, and `match`:
```rust
// In library code — never use unwrap():
let program = RuleProgram::from_yaml_str(&content)?;

// In examples/tests — unwrap() with context is fine:
let program = RuleProgram::from_yaml_str(&content)
    .expect("Failed to parse rule program");

// ❌ LLM mixes styles randomly in the same function
```

### Low: Duplicate Method Definitions

LLMs don't track what methods already exist on a type:
```rust
impl RuleProgram {
    pub fn from_path(path: &str) -> Self { ... }  // Already exists!
    // LLM adds another from_path → E0592 duplicate definition
}
```

---

## Part 4: Mitigation Strategies

### For Rule Generation
1. **Provide the exact serde schema** in the LLM prompt (not just examples)
2. **Include the Rust enum definitions** so the LLM can see `#[serde(tag = "type")]`
3. **Use a validator step**: parse the generated YAML immediately and report errors
4. **Consider adding a DRL parser** to let LLMs generate a more natural format

### For Rust Code Generation
1. **Always run `cargo check`** immediately after generation — catch errors early
2. **Run `cargo clippy --all-targets`** to catch style issues
3. **Provide the function signature** — let the LLM fill in the body
4. **Include existing `impl` blocks** in context so the LLM sees existing methods
5. **Pin the Rust edition** in prompts ("use Rust 2024 edition syntax")
6. **Prefer `thiserror`** over manual error types — fewer places to mess up

### For This Project Specifically
1. Keep a reference YAML file as a "golden example" in every LLM prompt
2. The `CategoryConfig` struct requires `name`, `display_name`, `max_points` — document this
3. `AwardPoints` always needs nested `formula: { type: fixed, points: N }` — never flat
4. `SetEligibility` uses `eligible:` not `is_eligible:`
5. Accumulate filter paths are **relative to array items**, not the root fact
