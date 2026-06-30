pub const RULES: &str = r#"
[
  {
    "id": "CIS-001",
    "pattern": ".*",
    "description": "Conflito de interesses",
    "condition": "!string_contains(get_string(ctx, \"vinculos\"), get_string(ctx, \"cnpj\"))",
    "severity": "Block",
    "enabled": true
  },
  {
    "id": "CIS-002",
    "pattern": ".*",
    "description": "Custo vs orçamento",
    "condition": "get_float(ctx, \"custo\") <= get_float(ctx, \"orcamento\")",
    "severity": "Block",
    "enabled": true
  },
  {
    "id": "CIS-003",
    "pattern": ".*",
    "description": "Impacto calculado",
    "condition": "get_bool(ctx, \"impacto_calculado\")",
    "severity": "Warning",
    "enabled": true
  }
]
"#;
