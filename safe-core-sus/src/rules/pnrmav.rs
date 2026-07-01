pub const RULES: &str = r#"
[
  {
    "id": "PNRMAV-001",
    "pattern": ".*",
    "description": "Anonimizado",
    "condition": "get_bool(ctx, \"anonimizado\")",
    "severity": "Block",
    "enabled": true
  },
  {
    "id": "PNRMAV-002",
    "pattern": ".*",
    "description": "Notificação",
    "condition": "get_bool(ctx, \"notificacao_compulsoria\")",
    "severity": "Block",
    "enabled": true
  },
  {
    "id": "PNRMAV-003",
    "pattern": ".*",
    "description": "Dados protegidos",
    "condition": "get_bool(ctx, \"dados_protegidos\")",
    "severity": "Block",
    "enabled": true
  },
  {
    "id": "PNRMAV-004",
    "pattern": ".*",
    "description": "LGPD compliant",
    "condition": "get_bool(ctx, \"lgpd_compliant\")",
    "severity": "Block",
    "enabled": true
  }
]
"#;
