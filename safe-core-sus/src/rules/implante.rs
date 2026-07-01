pub const RULES: &str = r#"
[
  {
    "id": "IMPLANTE-001",
    "pattern": ".*",
    "description": "Idade mínima",
    "condition": "get_int(ctx, \"idade\") >= 14",
    "severity": "Block",
    "enabled": true
  },
  {
    "id": "IMPLANTE-002",
    "pattern": ".*",
    "description": "Consentimento",
    "condition": "get_bool(ctx, \"consentimento\")",
    "severity": "Block",
    "enabled": true
  },
  {
    "id": "IMPLANTE-003",
    "pattern": ".*",
    "description": "Credenciado",
    "condition": "get_bool(ctx, \"credenciado\")",
    "severity": "Block",
    "enabled": true
  },
  {
    "id": "IMPLANTE-004",
    "pattern": ".*",
    "description": "Sigilo",
    "condition": "get_bool(ctx, \"sigilo\")",
    "severity": "Block",
    "enabled": true
  }
]
"#;
