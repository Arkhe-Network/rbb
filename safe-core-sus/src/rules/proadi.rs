pub const RULES: &str = r#"
[
  {
    "id": "PROADI-001",
    "pattern": ".*",
    "description": "Número de médicos",
    "condition": "get_int(ctx, \"medicos\") >= 2",
    "severity": "Block",
    "enabled": true
  },
  {
    "id": "PROADI-002",
    "pattern": ".*",
    "description": "Compatibilidade",
    "condition": "get_bool(ctx, \"compatibilidade\")",
    "severity": "Block",
    "enabled": true
  },
  {
    "id": "PROADI-003",
    "pattern": ".*",
    "description": "Centro habilitado",
    "condition": "get_bool(ctx, \"centro_habilitado\")",
    "severity": "Block",
    "enabled": true
  },
  {
    "id": "PROADI-004",
    "pattern": ".*",
    "description": "Lista SNT",
    "condition": "get_bool(ctx, \"na_lista_snt\")",
    "severity": "Block",
    "enabled": true
  }
]
"#;
