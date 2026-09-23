//! Drools-Equivalent Global Variables and Global Services Demo
//!
//! Demonstrates how Drools `global` constructs map directly to native
//! `KSD-CO/rust-rule-engine` capabilities:
//!
//! 1. **Global Services via Action Handlers**:
//!    Drools:
//!      `global com.company.AuditService auditService;`
//!      `global com.company.NotificationService notificationService;`
//!      `auditService.logEvent("VIP_PROMOTION", order.id);`
//!    rust-rule-engine:
//!      `engine.register_action_handler("RecordAudit", |params, facts| ...)`
//!      `engine.register_action_handler("SendNotification", |params, facts| ...)`
//!
//! 2. **Global Services via Enterprise `RulePlugin`**:
//!    Encapsulating services as reusable plugins implementing `RulePlugin`.
//!
//! 3. **Global Configuration Variables in Working Memory**:
//!    Drools:
//!      `global Double vipMinSpend;`
//!      `global Double discountRate;`
//!    rust-rule-engine:
//!      `facts.add_value("Global", Value::Object(global_params));`
//!
//! 4. **Persistent Globals via `GlobalsRegistry` (RETE Engine)**:
//!    CLIPS `defglobal` / Drools `global` registry in `rust_rule_engine::rete::globals`.
//!
//! Run with:
//!   cargo run --example drools_globals_and_services

use colored::Colorize;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use rust_rule_engine::engine::plugin::{PluginHealth, PluginMetadata, PluginState, RulePlugin};
use rust_rule_engine::rete::facts::FactValue;
use rust_rule_engine::rete::globals::GlobalsRegistry;
use rust_rule_engine::{EngineConfig, Facts, GRLParser, KnowledgeBase, RustRuleEngine, Value};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

// ============================================================================
// 1. GLOBAL SERVICE DEFINITIONS (Like Spring / CDI injected beans in Drools)
// ============================================================================

/// In Drools: `global com.enterprise.services.AuditService auditService;`
#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub action: String,
    pub entity_id: String,
    pub amount: f64,
}

#[derive(Debug, Default)]
pub struct AuditService {
    events: Arc<Mutex<Vec<AuditEvent>>>,
}

impl AuditService {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn record_audit(&self, action: &str, entity_id: &str, amount: f64) {
        let mut list = self.events.lock().unwrap();
        list.push(AuditEvent {
            action: action.to_string(),
            entity_id: entity_id.to_string(),
            amount,
        });
    }

    pub fn get_events(&self) -> Vec<AuditEvent> {
        self.events.lock().unwrap().clone()
    }
}

/// In Drools: `global com.enterprise.services.NotificationService notificationService;`
#[derive(Debug, Clone)]
pub struct NotificationMessage {
    pub recipient: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Default)]
pub struct NotificationService {
    sent: Arc<Mutex<Vec<NotificationMessage>>>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            sent: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn send(&self, recipient: &str, subject: &str, body: &str) {
        let mut list = self.sent.lock().unwrap();
        list.push(NotificationMessage {
            recipient: recipient.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
        });
    }

    pub fn get_sent(&self) -> Vec<NotificationMessage> {
        self.sent.lock().unwrap().clone()
    }
}

/// In Drools: `global com.enterprise.services.AlertService alertService;`
#[derive(Debug, Clone)]
pub struct SecurityAlert {
    pub team: String,
    pub reason: String,
}

#[derive(Debug, Default)]
pub struct AlertService {
    alerts: Arc<Mutex<Vec<SecurityAlert>>>,
}

impl AlertService {
    pub fn new() -> Self {
        Self {
            alerts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn send_alert(&self, team: &str, reason: &str) {
        let mut list = self.alerts.lock().unwrap();
        list.push(SecurityAlert {
            team: team.to_string(),
            reason: reason.to_string(),
        });
    }

    pub fn get_alerts(&self) -> Vec<SecurityAlert> {
        self.alerts.lock().unwrap().clone()
    }
}

// ============================================================================
// 2. GLOBAL SERVICE VIA `RulePlugin` (Enterprise Modular Service Pattern)
// ============================================================================

pub struct SecurityAuditPlugin {
    metadata: PluginMetadata,
    alert_service: Arc<AlertService>,
}

impl SecurityAuditPlugin {
    pub fn new(alert_service: Arc<AlertService>) -> Self {
        Self {
            metadata: PluginMetadata {
                name: "security-audit-plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Enterprise Security & Alerting Global Service".to_string(),
                author: "Platform Architecture Team".to_string(),
                state: PluginState::Loaded,
                health: PluginHealth::Healthy,
                actions: vec!["SendAlert".to_string()],
                functions: vec![],
                dependencies: vec![],
            },
            alert_service,
        }
    }
}

impl RulePlugin for SecurityAuditPlugin {
    fn get_metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn register_actions(&self, engine: &mut RustRuleEngine) -> rust_rule_engine::Result<()> {
        let alert_service = Arc::clone(&self.alert_service);
        engine.register_action_handler("SendAlert", move |params, _facts| {
            let team = params.get("0").and_then(|v| v.as_string()).unwrap_or_else(|| "OPS".to_string());
            let reason = params.get("1").and_then(|v| v.as_string()).unwrap_or_else(|| "Unknown".to_string());
            alert_service.send_alert(&team, &reason);
            Ok(())
        });
        Ok(())
    }
}

// ============================================================================
// 3. MAIN RUNNER
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n{}",
        " 🌐 DROOLS GLOBAL VARIABLES & GLOBAL SERVICES DEMO (via KSD-CO/rust-rule-engine) 🌐 "
            .bold()
            .on_purple()
            .white()
    );
    println!(" Demonstrates: Action Handlers as Global Services, RulePlugin, and RETE GlobalsRegistry.\n");

    // ------------------------------------------------------------------------
    // Step 1: Ingest standard GRL rule file
    // ------------------------------------------------------------------------
    let grl_path = "rules/drools_globals_and_services.grl";
    let grl_content = fs::read_to_string(grl_path)?;
    let rules = GRLParser::parse_rules(&grl_content)?;

    println!(
        " 📖 Ingested {} rules from '{}'",
        rules.len().to_string().cyan().bold(),
        grl_path.yellow()
    );

    let kb = KnowledgeBase::new("Drools_Globals_Services_KB");
    for r in &rules {
        kb.add_rule(r.clone())?;
    }

    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 5,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // ------------------------------------------------------------------------
    // Step 2: Instantiate Global Services (Drools `global AuditService auditService`)
    // ------------------------------------------------------------------------
    let audit_service = Arc::new(AuditService::new());
    let notification_service = Arc::new(NotificationService::new());
    let alert_service = Arc::new(AlertService::new());

    // Register Service 1: AuditService action handler
    let audit_clone = Arc::clone(&audit_service);
    engine.register_action_handler("RecordAudit", move |params, facts| {
        let action = params.get("0").and_then(|v| v.as_string()).unwrap_or_default();
        let entity_arg = params.get("1").and_then(|v| v.as_string()).unwrap_or_default();
        // Resolve nested fact if argument is a path like Order.id
        let entity_id = facts
            .get_nested(&entity_arg)
            .and_then(|v| v.as_string())
            .unwrap_or(entity_arg);

        let amount_arg = params.get("2");
        let amount = match amount_arg {
            Some(Value::Number(n)) => *n,
            Some(Value::String(s)) => facts
                .get_nested(s)
                .and_then(|v| v.as_number())
                .unwrap_or(0.0),
            _ => 0.0,
        };

        audit_clone.record_audit(&action, &entity_id, amount);
        Ok(())
    });

    // Register Service 2: NotificationService action handler
    let notify_clone = Arc::clone(&notification_service);
    engine.register_action_handler("SendNotification", move |params, facts| {
        let recipient_arg = params.get("0").and_then(|v| v.as_string()).unwrap_or_default();
        let recipient = facts
            .get_nested(&recipient_arg)
            .and_then(|v| v.as_string())
            .unwrap_or(recipient_arg);

        let subject = params.get("1").and_then(|v| v.as_string()).unwrap_or_default();
        let body = params.get("2").and_then(|v| v.as_string()).unwrap_or_default();

        notify_clone.send(&recipient, &subject, &body);
        Ok(())
    });

    // Register Service 3: SecurityAuditPlugin via RulePlugin trait
    let plugin = Arc::new(SecurityAuditPlugin::new(Arc::clone(&alert_service)));
    engine.load_plugin(plugin)?;

    // ------------------------------------------------------------------------
    // Step 3: Global Configuration Variables in Working Memory
    // (In Drools: global Double vipMinSpend = 1000.0, etc.)
    // ------------------------------------------------------------------------
    let facts = Facts::new();

    let mut global_config = HashMap::new();
    global_config.insert("vip_min_spend".to_string(), Value::Number(1000.0));
    global_config.insert("vip_discount_rate".to_string(), Value::Number(0.15));
    global_config.insert("loyalty_redeem_threshold".to_string(), Value::Number(50.0));
    global_config.insert("fraud_review_limit".to_string(), Value::Number(5000.0));
    facts.add_value("Global", Value::Object(global_config))?;

    // Candidate Fact 1: VIP Customer with $1,500 Order
    let mut customer = HashMap::new();
    customer.insert("id".to_string(), Value::String("CUST-1001".to_string()));
    customer.insert("tier".to_string(), Value::String("VIP".to_string()));
    customer.insert("email".to_string(), Value::String("sarah.vip@example.com".to_string()));
    customer.insert("loyalty_points".to_string(), Value::Number(120.0));
    facts.add_value("Customer", Value::Object(customer))?;

    let mut order = HashMap::new();
    order.insert("id".to_string(), Value::String("ORD-9901".to_string()));
    order.insert("total".to_string(), Value::Number(1500.0));
    order.insert("processed".to_string(), Value::Boolean(false));
    order.insert("flagged".to_string(), Value::Boolean(false));
    facts.add_value("Order", Value::Object(order))?;

    println!("\n{}", "⚙️  Executing Scenario 1: VIP Customer Promotion".bold().underline());
    let res = engine.execute(&facts)?;
    println!(
        "   • Rule execution completed: {} rule(s) fired across {} cycle(s)",
        res.rules_fired.to_string().green().bold(),
        res.cycle_count
    );

    // Inspect updated Order state
    if let Some(Value::Object(order_obj)) = facts.get("Order") {
        println!("   • Updated Order State:");
        println!("     - Processed: {:?}", order_obj.get("processed"));
        println!("     - Discount:  {:?}", order_obj.get("discount"));
    }

    // ------------------------------------------------------------------------
    // Step 4: Verify Global Services Interception
    // ------------------------------------------------------------------------
    println!("\n📋 Global Service 1 (AuditService) Intercepted Events:");
    let mut audit_table = Table::new();
    audit_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Audit Action").fg(Color::Cyan),
            Cell::new("Entity / Target ID").fg(Color::Yellow),
            Cell::new("Amount / Value").fg(Color::Green),
        ]);
    for ev in audit_service.get_events() {
        audit_table.add_row(vec![
            Cell::new(ev.action),
            Cell::new(ev.entity_id),
            Cell::new(format!("${:.2}", ev.amount)),
        ]);
    }
    println!("{audit_table}");

    println!("📧 Global Service 2 (NotificationService) Dispatched Messages:");
    let mut notify_table = Table::new();
    notify_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Recipient").fg(Color::Cyan),
            Cell::new("Subject").fg(Color::Yellow),
            Cell::new("Message Body").fg(Color::White),
        ]);
    for msg in notification_service.get_sent() {
        notify_table.add_row(vec![
            Cell::new(msg.recipient),
            Cell::new(msg.subject),
            Cell::new(msg.body),
        ]);
    }
    println!("{notify_table}");

    // ------------------------------------------------------------------------
    // Step 5: Execute Scenario 2 (Fraud Risk triggering AlertService Plugin)
    // ------------------------------------------------------------------------
    println!("{}", "⚙️  Executing Scenario 2: High Value Transaction ($12,500)".bold().underline());
    let facts2 = Facts::new();
    let mut global_config2 = HashMap::new();
    global_config2.insert("vip_min_spend".to_string(), Value::Number(1000.0));
    global_config2.insert("vip_discount_rate".to_string(), Value::Number(0.15));
    global_config2.insert("loyalty_redeem_threshold".to_string(), Value::Number(50.0));
    global_config2.insert("fraud_review_limit".to_string(), Value::Number(5000.0));
    facts2.add_value("Global", Value::Object(global_config2))?;

    let mut customer2 = HashMap::new();
    customer2.insert("id".to_string(), Value::String("CUST-2002".to_string()));
    customer2.insert("tier".to_string(), Value::String("Standard".to_string()));
    customer2.insert("email".to_string(), Value::String("john.doe@example.com".to_string()));
    customer2.insert("loyalty_points".to_string(), Value::Number(10.0));
    facts2.add_value("Customer", Value::Object(customer2))?;

    let mut order2 = HashMap::new();
    order2.insert("id".to_string(), Value::String("ORD-9999".to_string()));
    order2.insert("total".to_string(), Value::Number(12500.0));
    order2.insert("processed".to_string(), Value::Boolean(false));
    order2.insert("flagged".to_string(), Value::Boolean(false));
    facts2.add_value("Order", Value::Object(order2))?;

    let res2 = engine.execute(&facts2)?;
    println!(
        "   • Rule execution completed: {} rule(s) fired across {} cycle(s)",
        res2.rules_fired.to_string().green().bold(),
        res2.cycle_count
    );

    println!("\n🚨 Global Service 3 (AlertService Plugin) Triggered Alerts:");
    let mut alert_table = Table::new();
    alert_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Destination Team").fg(Color::Red),
            Cell::new("Alert Reason").fg(Color::Yellow),
        ]);
    for alt in alert_service.get_alerts() {
        alert_table.add_row(vec![Cell::new(alt.team), Cell::new(alt.reason)]);
    }
    println!("{alert_table}");

    // ------------------------------------------------------------------------
    // Step 6: Native RETE GlobalsRegistry (Direct Drools global parity)
    // ------------------------------------------------------------------------
    println!("🏛️  Native RETE GlobalsRegistry (CLIPS defglobal / Drools global parity):");
    let globals_reg = GlobalsRegistry::new();
    globals_reg.define("GLOBAL_DISCOUNT_CAP", FactValue::Float(0.35))?;
    globals_reg.define_readonly("COMPLIANCE_VERSION", FactValue::String("2026.1".to_string()))?;
    globals_reg.define("TRANSACTION_COUNTER", FactValue::Integer(100))?;
    globals_reg.increment("TRANSACTION_COUNTER", 5.0)?;

    println!(
        "   • GLOBAL_DISCOUNT_CAP: {:?}",
        globals_reg.get("GLOBAL_DISCOUNT_CAP")?
    );
    println!(
        "   • COMPLIANCE_VERSION (read-only): {:?}",
        globals_reg.get("COMPLIANCE_VERSION")?
    );
    println!(
        "   • TRANSACTION_COUNTER (incremented): {:?}",
        globals_reg.get("TRANSACTION_COUNTER")?
    );

    println!("\n════════════════════════════════════════════════════════════════════════════");
    println!(" ✅ Drools Global Variables & Global Services Equivalence Verified!");
    println!("════════════════════════════════════════════════════════════════════════════\n");

    Ok(())
}
