use regex::Regex;
use rhai::{Dynamic, Engine, Map, AST};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Block,
    Warning,
    Log,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicsRule {
    pub id: String,
    pub pattern: String,
    pub description: String,
    pub condition: String,
    pub severity: Severity,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    pub rule_id: String,
    pub passed: bool,
    pub severity: Severity,
    pub message: String,
}

pub struct CompiledRule {
    pub config: EthicsRule,
    pub ast: AST,
    pub regex: Regex,
}

pub struct RuleEngine {
    engine: Engine,
    rules: Vec<CompiledRule>,
}

impl RuleEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Registrar funções auxiliares no engine Rhai
        engine.register_fn("get_string", |ctx: &mut Map, key: &str| -> String {
            match ctx.get(key) {
                Some(v) => v.clone().into_string().unwrap_or_default(),
                None => String::new(),
            }
        });

        engine.register_fn("get_bool", |ctx: &mut Map, key: &str| -> bool {
            match ctx.get(key) {
                Some(v) => v.as_bool().unwrap_or(false),
                None => false,
            }
        });

        engine.register_fn("get_int", |ctx: &mut Map, key: &str| -> i64 {
            match ctx.get(key) {
                Some(v) => v.as_int().unwrap_or(0),
                None => 0,
            }
        });

        engine.register_fn("get_float", |ctx: &mut Map, key: &str| -> f64 {
            match ctx.get(key) {
                Some(v) => v.as_float().unwrap_or(0.0),
                None => 0.0,
            }
        });

        engine.register_fn("string_contains", |s: &str, sub: &str| -> bool {
            s.contains(sub)
        });

        Self {
            engine,
            rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: EthicsRule) -> Result<(), String> {
        let ast = self
            .engine
            .compile(&rule.condition)
            .map_err(|e| format!("Erro compilando regra {}: {}", rule.id, e))?;

        let regex = Regex::new(&rule.pattern)
            .map_err(|e| format!("Regex inválido na regra {}: {}", rule.id, e))?;

        self.rules.push(CompiledRule {
            config: rule,
            ast,
            regex,
        });

        Ok(())
    }

    pub fn evaluate(
        &self,
        action: &str,
        context: &serde_json::Value,
    ) -> Result<Vec<RuleResult>, String> {
        let mut results = Vec::new();

        // Converter serde_json::Value para rhai::Map
        let mut rhai_map = Map::new();
        if let serde_json::Value::Object(map) = context {
            for (k, v) in map {
                let dynamic_val = match v {
                    serde_json::Value::String(s) => Dynamic::from(s.clone()),
                    serde_json::Value::Bool(b) => Dynamic::from(*b),
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            Dynamic::from(i)
                        } else if let Some(f) = n.as_f64() {
                            Dynamic::from(f as rhai::FLOAT)
                        } else {
                            Dynamic::UNIT
                        }
                    }
                    _ => Dynamic::UNIT, // Simples ignorar tipos complexos no primeiro momento
                };
                rhai_map.insert(k.clone().into(), dynamic_val);
            }
        } else {
            return Err("Contexto deve ser um objeto JSON".to_string());
        }

        let mut scope = rhai::Scope::new();
        scope.push("ctx", rhai_map);

        for compiled in &self.rules {
            if !compiled.config.enabled || !compiled.regex.is_match(action) {
                continue;
            }

            match self
                .engine
                .eval_ast_with_scope::<bool>(&mut scope, &compiled.ast)
            {
                Ok(passed) => {
                    results.push(RuleResult {
                        rule_id: compiled.config.id.clone(),
                        passed,
                        severity: compiled.config.severity.clone(),
                        message: if passed {
                            "OK".to_string()
                        } else {
                            "Condição falhou".to_string()
                        },
                    });
                }
                Err(e) => {
                    return Err(format!(
                        "Erro avaliando regra {}: {}",
                        compiled.config.id, e
                    ));
                }
            }
        }

        Ok(results)
    }
}
