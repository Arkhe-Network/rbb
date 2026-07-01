use safe_core_sus::{EthicsRule, RuleEngine, Severity};

fn create_engine() -> RuleEngine {
    let mut engine = RuleEngine::new();

    // CIS
    engine
        .add_rule(EthicsRule {
            id: "CIS-001".into(),
            pattern: ".*".into(),
            description: "Conflito de interesses".into(),
            condition: "!string_contains(get_string(ctx, \"vinculos\"), get_string(ctx, \"cnpj\"))"
                .into(),
            severity: Severity::Block,
            enabled: true,
        })
        .unwrap();
    engine
        .add_rule(EthicsRule {
            id: "CIS-002".into(),
            pattern: ".*".into(),
            description: "Custo vs orcamento".into(),
            condition: "get_float(ctx, \"custo\") <= get_float(ctx, \"orcamento\")".into(),
            severity: Severity::Block,
            enabled: true,
        })
        .unwrap();
    engine
        .add_rule(EthicsRule {
            id: "CIS-003".into(),
            pattern: ".*".into(),
            description: "Impacto calculado".into(),
            condition: "get_bool(ctx, \"impacto_calculado\")".into(),
            severity: Severity::Warning,
            enabled: true,
        })
        .unwrap();

    // PROADI
    engine
        .add_rule(EthicsRule {
            id: "PROADI-001".into(),
            pattern: ".*".into(),
            description: "Número de médicos".into(),
            condition: "get_int(ctx, \"medicos\") >= 2".into(),
            severity: Severity::Block,
            enabled: true,
        })
        .unwrap();
    engine
        .add_rule(EthicsRule {
            id: "PROADI-002".into(),
            pattern: ".*".into(),
            description: "Compatibilidade".into(),
            condition: "get_bool(ctx, \"compatibilidade\")".into(),
            severity: Severity::Block,
            enabled: true,
        })
        .unwrap();

    // IMPLANTE
    engine
        .add_rule(EthicsRule {
            id: "IMPLANTE-001".into(),
            pattern: ".*".into(),
            description: "Idade".into(),
            condition: "get_int(ctx, \"idade\") >= 14".into(),
            severity: Severity::Block,
            enabled: true,
        })
        .unwrap();
    engine
        .add_rule(EthicsRule {
            id: "IMPLANTE-002".into(),
            pattern: ".*".into(),
            description: "Consentimento".into(),
            condition: "get_bool(ctx, \"consentimento\")".into(),
            severity: Severity::Block,
            enabled: true,
        })
        .unwrap();

    // PNRMAV
    engine
        .add_rule(EthicsRule {
            id: "PNRMAV-001".into(),
            pattern: ".*".into(),
            description: "Anonimizado".into(),
            condition: "get_bool(ctx, \"anonimizado\")".into(),
            severity: Severity::Block,
            enabled: true,
        })
        .unwrap();

    engine
}

#[test]
fn test_cis_001_pass() {
    let engine = create_engine();
    let ctx = serde_json::json!({ "vinculos": "empresas_a_b", "cnpj": "empresa_c" });
    let res = engine.evaluate("acao", &ctx).unwrap();
    let rule_res = res.iter().find(|r| r.rule_id == "CIS-001").unwrap();
    assert!(rule_res.passed);
}

#[test]
fn test_cis_001_fail() {
    let engine = create_engine();
    let ctx = serde_json::json!({ "vinculos": "empresas_a_b", "cnpj": "empresas_a" });
    let res = engine.evaluate("acao", &ctx).unwrap();
    let rule_res = res.iter().find(|r| r.rule_id == "CIS-001").unwrap();
    assert!(!rule_res.passed);
}

#[test]
fn test_cis_002_pass() {
    let engine = create_engine();
    let ctx = serde_json::json!({ "custo": 100.0, "orcamento": 200.0 });
    let res = engine.evaluate("acao", &ctx).unwrap();
    let rule_res = res.iter().find(|r| r.rule_id == "CIS-002").unwrap();
    assert!(rule_res.passed);
}

#[test]
fn test_cis_002_fail() {
    let engine = create_engine();
    let ctx = serde_json::json!({ "custo": 300.0, "orcamento": 200.0 });
    let res = engine.evaluate("acao", &ctx).unwrap();
    let rule_res = res.iter().find(|r| r.rule_id == "CIS-002").unwrap();
    assert!(!rule_res.passed);
}

#[test]
fn test_cis_003_pass() {
    let engine = create_engine();
    let ctx = serde_json::json!({ "impacto_calculado": true });
    let res = engine.evaluate("acao", &ctx).unwrap();
    let rule_res = res.iter().find(|r| r.rule_id == "CIS-003").unwrap();
    assert!(rule_res.passed);
}

#[test]
fn test_cis_003_fail() {
    let engine = create_engine();
    let ctx = serde_json::json!({ "impacto_calculado": false });
    let res = engine.evaluate("acao", &ctx).unwrap();
    let rule_res = res.iter().find(|r| r.rule_id == "CIS-003").unwrap();
    assert!(!rule_res.passed);
}

#[test]
fn test_proadi_001_pass() {
    let engine = create_engine();
    let ctx = serde_json::json!({ "medicos": 2 });
    let res = engine.evaluate("acao", &ctx).unwrap();
    let rule_res = res.iter().find(|r| r.rule_id == "PROADI-001").unwrap();
    assert!(rule_res.passed);
}

#[test]
fn test_proadi_001_fail() {
    let engine = create_engine();
    let ctx = serde_json::json!({ "medicos": 1 });
    let res = engine.evaluate("acao", &ctx).unwrap();
    let rule_res = res.iter().find(|r| r.rule_id == "PROADI-001").unwrap();
    assert!(!rule_res.passed);
}

#[test]
fn test_proadi_002_pass() {
    let engine = create_engine();
    let ctx = serde_json::json!({ "compatibilidade": true });
    let res = engine.evaluate("acao", &ctx).unwrap();
    let rule_res = res.iter().find(|r| r.rule_id == "PROADI-002").unwrap();
    assert!(rule_res.passed);
}

#[test]
fn test_proadi_002_fail() {
    let engine = create_engine();
    let ctx = serde_json::json!({ "compatibilidade": false });
    let res = engine.evaluate("acao", &ctx).unwrap();
    let rule_res = res.iter().find(|r| r.rule_id == "PROADI-002").unwrap();
    assert!(!rule_res.passed);
}

#[test]
fn test_implante_001_pass() {
    let engine = create_engine();
    let ctx = serde_json::json!({ "idade": 14 });
    let res = engine.evaluate("acao", &ctx).unwrap();
    let rule_res = res.iter().find(|r| r.rule_id == "IMPLANTE-001").unwrap();
    assert!(rule_res.passed);
}

#[test]
fn test_implante_001_fail() {
    let engine = create_engine();
    let ctx = serde_json::json!({ "idade": 13 });
    let res = engine.evaluate("acao", &ctx).unwrap();
    let rule_res = res.iter().find(|r| r.rule_id == "IMPLANTE-001").unwrap();
    assert!(!rule_res.passed);
}

#[test]
fn test_implante_002_pass() {
    let engine = create_engine();
    let ctx = serde_json::json!({ "consentimento": true });
    let res = engine.evaluate("acao", &ctx).unwrap();
    let rule_res = res.iter().find(|r| r.rule_id == "IMPLANTE-002").unwrap();
    assert!(rule_res.passed);
}

#[test]
fn test_implante_002_fail() {
    let engine = create_engine();
    let ctx = serde_json::json!({ "consentimento": false });
    let res = engine.evaluate("acao", &ctx).unwrap();
    let rule_res = res.iter().find(|r| r.rule_id == "IMPLANTE-002").unwrap();
    assert!(!rule_res.passed);
}

#[test]
fn test_pnrmav_001_pass() {
    let engine = create_engine();
    let ctx = serde_json::json!({ "anonimizado": true });
    let res = engine.evaluate("acao", &ctx).unwrap();
    let rule_res = res.iter().find(|r| r.rule_id == "PNRMAV-001").unwrap();
    assert!(rule_res.passed);
}

#[test]
fn test_pnrmav_001_fail() {
    let engine = create_engine();
    let ctx = serde_json::json!({ "anonimizado": false });
    let res = engine.evaluate("acao", &ctx).unwrap();
    let rule_res = res.iter().find(|r| r.rule_id == "PNRMAV-001").unwrap();
    assert!(!rule_res.passed);
}

#[test]
fn test_missing_property_returns_false() {
    let engine = create_engine();
    let ctx = serde_json::json!({});
    let res = engine.evaluate("acao", &ctx).unwrap();
    // Default int is 0, so PROADI-001 fails
    let rule_res = res.iter().find(|r| r.rule_id == "PROADI-001").unwrap();
    assert!(!rule_res.passed);
}

#[test]
fn test_regex_action() {
    let mut engine = RuleEngine::new();
    engine
        .add_rule(EthicsRule {
            id: "REG-001".into(),
            pattern: "^action_a$".into(),
            description: "".into(),
            condition: "true".into(),
            severity: Severity::Block,
            enabled: true,
        })
        .unwrap();

    let ctx = serde_json::json!({});
    let res = engine.evaluate("action_a", &ctx).unwrap();
    assert_eq!(res.len(), 1);

    let res_b = engine.evaluate("action_b", &ctx).unwrap();
    assert_eq!(res_b.len(), 0);
}

#[test]
fn test_disabled_rule() {
    let mut engine = RuleEngine::new();
    engine
        .add_rule(EthicsRule {
            id: "DIS-001".into(),
            pattern: ".*".into(),
            description: "".into(),
            condition: "true".into(),
            severity: Severity::Block,
            enabled: false,
        })
        .unwrap();

    let ctx = serde_json::json!({});
    let res = engine.evaluate("acao", &ctx).unwrap();
    assert_eq!(res.len(), 0);
}

#[test]
fn test_invalid_context() {
    let engine = create_engine();
    let ctx = serde_json::json!([]); // Not an object
    let res = engine.evaluate("acao", &ctx);
    assert!(res.is_err());
}
