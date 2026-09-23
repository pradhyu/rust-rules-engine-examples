# External Properties & Dependency Injection in Rust (Spring Boot Migration Guide)

This guide documents how enterprise configuration management and Dependency Injection (DI) paradigms from **Spring Boot (Java)** map to **idiomatic Rust**, specifically for low-latency microservices and rules engines.

---

## 1. Architectural Philosophy: Spring Boot vs. Rust

| Dimension | Spring Boot (Java) | Idiomatic Rust |
|---|---|---|
| **Discovery Mechanism** | Runtime classpath reflection (`@ComponentScan`) | Compile-time modules (`pub mod`, `use`) |
| **Configuration Files** | `application.yml`, `application-{profile}.yml` | `config/default.yaml`, `config/{env}.yaml` |
| **Layered Overrides** | CLI args > System props > Env vars > YAML | `config-rs` or `figment` layered provider builder |
| **Environment Variables** | `SPRING_APPLICATION_JSON`, `SERVER_PORT` | `APP__SERVER__PORT` (Serde nested separator) |
| **Type Binding** | `@ConfigurationProperties(prefix = "app")` | `#[derive(Deserialize)]` on Rust structs |
| **Default Fallbacks** | `@Value("${app.timeout:5000}")` | `#[serde(default = "default_timeout")]` |
| **Dependency Injection** | `@Autowired` into Spring Beans | `axum::extract::State`, Tonic service fields, or constructor injection |
| **Global Access** | Spring `ApplicationContext.getBean(...)` | `std::sync::OnceLock<AppConfig>` static singleton |
| **Memory Cost** | Heavy JVM reflection metadata + CGLIB proxies | Zero-cost abstraction, stack/heap inline allocations |

---

## 2. Layered Configuration with `config-rs`

[`config-rs`](https://crates.io/crates/config) is the de facto standard in Rust that mirrors Spring Boot's multi-layered configuration resolution.

### Strong-Typed Config Structs

```rust
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub rules: RulesConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RulesConfig {
    pub default_rules_path: String,
    pub pass_mark_threshold: f64,
    pub max_cycles: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("RUN_ENV").unwrap_or_else(|_| "development".into());

        let builder = Config::builder()
            // 1. Base default configuration (application.yml)
            .add_source(File::with_name("config/default").required(true))
            // 2. Profile-specific override (e.g. config/production.yaml)
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // 3. Local developer overrides (e.g. config/local.yaml, ignored by git)
            .add_source(File::with_name("config/local").required(false))
            // 4. Environment variable overrides (e.g. APP__SERVER__PORT=9090)
            .add_source(
                Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true),
            );

        builder.build()?.try_deserialize()
    }
}
```

### Profile Files

**`config/default.yaml`**:
```yaml
server:
  host: "0.0.0.0"
  port: 8080

rules:
  default_rules_path: "rules/uk_skilled_worker_points.grl"
  pass_mark_threshold: 70.0
  max_cycles: 5

database:
  url: "postgres://localhost:5432/rules_db"
  max_connections: 10
```

**`config/production.yaml`**:
```yaml
server:
  port: 443

database:
  max_connections: 50
```

### Overriding via Environment Variables (Kubernetes / Docker)
Just like in Spring Boot (`APP_SERVER_PORT=9090`), double-underscore mapping resolves nested paths:
```bash
export APP__SERVER__PORT=9090
export APP__RULES__PASS_MARK_THRESHOLD=65.0
```

---

## 3. Dependency Injection in Web Services

Rust web frameworks do not use runtime component scanning. Instead, they use **Type-Safe Application State** (`Arc<AppState>`).

### Axum (REST) Pattern
```rust
use axum::{extract::State, routing::get, Json, Router};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub audit_service: Arc<AuditService>,
    pub rule_engine: Arc<KnowledgeBase>,
}

async fn get_server_info(State(state): State<Arc<AppState>>) -> Json<ServerConfig> {
    // Injected properties and services are accessible through state
    Json(state.config.server.clone())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(AppConfig::load()?);
    let state = Arc::new(AppState {
        config: Arc::clone(&config),
        audit_service: Arc::new(AuditService::new()),
        rule_engine: Arc::new(KnowledgeBase::new("ProdKB")),
    });

    let app = Router::new()
        .route("/api/v1/info", get(get_server_info))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

---

## 4. Global Static Access (`std::sync::OnceLock`)

When deep call stacks or background workers need access to configuration without threading parameters through every constructor:

```rust
use std::sync::OnceLock;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

pub fn get_config() -> &'static AppConfig {
    CONFIG.get_or_init(|| {
        AppConfig::load().expect("Failed to initialize application configuration")
    })
}
```

---

## 5. Exposing External Properties to Rules (`rust-rule-engine`)

External configuration can be made available to business rules using two native patterns:

### Method A: Fact Working Memory (`Global` Namespace)
```rust
let config = get_config();

let mut global_props = std::collections::HashMap::new();
global_props.insert("vip_threshold".to_string(), Value::Number(config.rules.pass_mark_threshold));
facts.add_value("Global", Value::Object(global_props))?;
```
In GRL:
```grl
rule "CheckPassMark" {
    when
        Applicant.score >= Global.vip_threshold
    then
        Applicant.eligible = true;
}
```

### Method B: RETE Persistent Globals (`GlobalsRegistry`)
```rust
use rust_rule_engine::rete::globals::GlobalsRegistry;
use rust_rule_engine::rete::facts::FactValue;

let globals = GlobalsRegistry::new();
globals.define_readonly("PASS_MARK", FactValue::Float(config.rules.pass_mark_threshold))?;
```
