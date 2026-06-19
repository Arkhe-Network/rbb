//! src/substrato_8000/cache_aligner.rs
//! CacheAligner — Otimização de KV Cache para múltiplos providers LLM
//! Prefixos estáveis para maximizar cache hits em chamadas repetidas
//!
//! Selo: CATHEDRAL-ARKHE-8000-CACHE-ALIGNER-v1.0.0-2026-06-18
//! Arquiteto: ORCID 0009-0005-2697-4668

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// ============================================================
/// 1. PROVIDER CONFIGURATIONS
/// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LlmProvider {
    Anthropic,
    OpenAI,
    Gemini,
    Rio35,       // Cathedral internal
    Vllm,        // Self-hosted vLLM
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCacheConfig {
    pub provider: LlmProvider,
    /// Tamanho do prefixo estável (tokens)
    pub stable_prefix_size: usize,
    /// Estratégia de alinhamento
    pub alignment_strategy: AlignmentStrategy,
    /// Se suporta prefix caching nativo
    pub native_prefix_caching: bool,
    /// Formato de mensagens esperado
    pub message_format: MessageFormat,
    /// Tokens de system prompt fixos
    pub system_prompt_tokens: Vec<String>,
    /// Separador de contexto
    pub context_separator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlignmentStrategy {
    /// Prefixo fixo no início de todas as mensagens
    FixedPrefix,
    /// Prefixo rotativo baseado em hash do contexto
    RotatingPrefix,
    /// Hierárquico: system + user + assistant padrões
    Hierarchical,
    /// Customizado por domínio
    DomainSpecific { domain: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageFormat {
    /// Anthropic: <system>...</system>

Human: ...

Assistant: ...
    AnthropicClaude,
    /// OpenAI: [{"role": "system", "content": ...}, ...]
    OpenAIChat,
    /// Gemini: parts[] com role
    GeminiPro,
    /// Cathedral: custom format
    CathedralNative,
    /// Raw text
    Raw,
}

/// ============================================================
/// 2. CACHE ALIGNER ENGINE
/// ============================================================

pub struct CacheAligner {
    configs: HashMap<LlmProvider, ProviderCacheConfig>,
    /// Cache de prefixos já computados
    prefix_cache: HashMap<String, Vec<String>>,
    /// Estatísticas de cache hit
    stats: CacheAlignerStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheAlignerStats {
    pub total_alignments: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_prefix_size: f64,
    pub provider_hits: HashMap<String, u64>,
}

impl CacheAligner {
    pub fn new() -> Self {
        let mut configs = HashMap::new();

        // Anthropic Claude
        configs.insert(LlmProvider::Anthropic, ProviderCacheConfig {
            provider: LlmProvider::Anthropic,
            stable_prefix_size: 1024,
            alignment_strategy: AlignmentStrategy::Hierarchical,
            native_prefix_caching: true,
            message_format: MessageFormat::AnthropicClaude,
            system_prompt_tokens: vec![
                "You are a helpful AI assistant.".to_string(),
                "Follow the instructions carefully.".to_string(),
            ],
            context_separator: " \n\nHuman: ".to_string(),
        });

        // OpenAI GPT
        configs.insert(LlmProvider::OpenAI, ProviderCacheConfig {
            provider: LlmProvider::OpenAI,
            stable_prefix_size: 512,
            alignment_strategy: AlignmentStrategy::FixedPrefix,
            native_prefix_caching: false,
            message_format: MessageFormat::OpenAIChat,
            system_prompt_tokens: vec![
                "You are a helpful assistant.".to_string(),
            ],
            context_separator: " \n".to_string(),
        });

        // Google Gemini
        configs.insert(LlmProvider::Gemini, ProviderCacheConfig {
            provider: LlmProvider::Gemini,
            stable_prefix_size: 768,
            alignment_strategy: AlignmentStrategy::RotatingPrefix,
            native_prefix_caching: true,
            message_format: MessageFormat::GeminiPro,
            system_prompt_tokens: vec![
                "You are a helpful AI.".to_string(),
            ],
            context_separator: " \n---\n".to_string(),
        });

        // Cathedral Rio-3.5
        configs.insert(LlmProvider::Rio35, ProviderCacheConfig {
            provider: LlmProvider::Rio35,
            stable_prefix_size: 2048,
            alignment_strategy: AlignmentStrategy::DomainSpecific {
                domain: "cathedral_arkhe".to_string()
            },
            native_prefix_caching: true,
            message_format: MessageFormat::CathedralNative,
            system_prompt_tokens: vec![
                "You are Cathedral ARKHE, an advanced cognitive system.".to_string(),
                "Follow EthicalFilter P1-P7.".to_string(),
                "Use Immersion-Driven Thinking for complex tasks.".to_string(),
            ],
            context_separator: " \n<|context|>\n".to_string(),
        });

        // vLLM (self-hosted)
        configs.insert(LlmProvider::Vllm, ProviderCacheConfig {
            provider: LlmProvider::Vllm,
            stable_prefix_size: 1024,
            alignment_strategy: AlignmentStrategy::FixedPrefix,
            native_prefix_caching: true,
            message_format: MessageFormat::OpenAIChat,
            system_prompt_tokens: vec![],
            context_separator: " \n".to_string(),
        });

        Self {
            configs,
            prefix_cache: HashMap::new(),
            stats: CacheAlignerStats::default(),
        }
    }

    /// ============================================================
    /// 2.1 ALIGN MESSAGES FOR PROVIDER
    /// ============================================================

    /// Alinha mensagens para maximizar KV cache hits
    pub fn align_messages(
        &mut self,
        provider: &LlmProvider,
        messages: &[LlmMessage],
        context_id: &str,
    ) -> Result<Vec<LlmMessage>, CacheAlignError> {
        let config = self.configs.get(provider)
            .ok_or(CacheAlignError::UnsupportedProvider(format!("{:?}", provider)))?;

        self.stats.total_alignments += 1;

        // Verifica cache de prefixo
        let cache_key = format!("{}:{}", provider_key(provider), context_id);
        if let Some(cached_prefix) = self.prefix_cache.get(&cache_key) {
            self.stats.cache_hits += 1;
            *self.stats.provider_hits.entry(provider_key(provider)).or_insert(0) += 1;

            return Ok(self.apply_cached_prefix(messages, cached_prefix, config));
        }

        self.stats.cache_misses += 1;

        // Computa novo prefixo estável
        let prefix = self.compute_stable_prefix(config, messages, context_id)?;
        self.prefix_cache.insert(cache_key, prefix.clone());

        let aligned = self.apply_cached_prefix(messages, &prefix, config);

        // Atualiza estatísticas
        let prefix_size = prefix.iter().map(|s| s.len()).sum::<usize>();
        self.stats.avg_prefix_size =
            (self.stats.avg_prefix_size * (self.stats.total_alignments - 1) as f64 + prefix_size as f64)
            / self.stats.total_alignments as f64;

        Ok(aligned)
    }

    /// ============================================================
    /// 2.2 PREFIX COMPUTATION
    /// ============================================================

    fn compute_stable_prefix(
        &self,
        config: &ProviderCacheConfig,
        messages: &[LlmMessage],
        context_id: &str,
    ) -> Result<Vec<String>, CacheAlignError> {
        let mut prefix = vec![];

        match &config.alignment_strategy {
            AlignmentStrategy::FixedPrefix => {
                // System prompt + primeiro user message como prefixo fixo
                prefix.extend(config.system_prompt_tokens.clone());
                if let Some(first) = messages.first() {
                    prefix.push(first.content.clone());
                }
            }
            AlignmentStrategy::RotatingPrefix => {
                // Hash do context_id determina rotação
                let hash = sha256(context_id);
                let rotation = hash[0] as usize % messages.len().max(1);

                prefix.extend(config.system_prompt_tokens.clone());
                if let Some(msg) = messages.get(rotation) {
                    prefix.push(msg.content.clone());
                }
            }
            AlignmentStrategy::Hierarchical => {
                // System + user + assistant padrões
                prefix.extend(config.system_prompt_tokens.clone());

                // Adiciona até 2 turnos completos como prefixo
                for msg in messages.iter().take(4) {
                    prefix.push(format!("[{}] {}", msg.role, msg.content));
                }
            }
            AlignmentStrategy::DomainSpecific { domain } => {
                // Domínio-specific prefix para Cathedral
                prefix.push(format!("<domain>{}</domain>", domain));
                prefix.extend(config.system_prompt_tokens.clone());

                // Adiciona metadados de contexto Cathedral
                prefix.push(format!("<context_id>{}</context_id>", context_id));
                prefix.push("<thinking_mode>adaptive</thinking_mode>".to_string());

                if let Some(first) = messages.first() {
                    prefix.push(first.content.clone());
                }
            }
        }

        Ok(prefix)
    }

    fn apply_cached_prefix(
        &self,
        messages: &[LlmMessage],
        prefix: &[String],
        config: &ProviderCacheConfig,
    ) -> Vec<LlmMessage> {
        let mut result = vec![];

        // Adiciona prefixo como system message
        if !prefix.is_empty() {
            let system_content = prefix.join(&config.context_separator);
            result.push(LlmMessage {
                role: "system".to_string(),
                content: system_content,
            });
        }

        // Adiciona mensagens originais
        for msg in messages {
            // Evita duplicar se a mensagem já está no prefixo
            if !prefix.contains(&msg.content) {
                result.push(msg.clone());
            }
        }

        result
    }

    /// ============================================================
    /// 2.3 FORMAT CONVERSION
    /// ============================================================

    /// Converte mensagens para formato específico do provider
    pub fn format_for_provider(
        &self,
        provider: &LlmProvider,
        messages: &[LlmMessage],
    ) -> Result<String, CacheAlignError> {
        let config = self.configs.get(provider)
            .ok_or(CacheAlignError::UnsupportedProvider(format!("{:?}", provider)))?;

        match &config.message_format {
            MessageFormat::AnthropicClaude => {
                let mut output = String::new();
                for msg in messages {
                    match msg.role.as_str() {
                        "system" => output.push_str(&format!("<system>{}</system>\n\n", msg.content)),
                        "user" | "human" => output.push_str(&format!("Human: {}\n\n", msg.content)),
                        "assistant" => output.push_str(&format!("Assistant: {}\n\n", msg.content)),
                        _ => output.push_str(&format!("{}: {}\n\n", msg.role, msg.content)),
                    }
                }
                Ok(output)
            }
            MessageFormat::OpenAIChat => {
                // Já está no formato correto, retorna JSON
                serde_json::to_string(messages)
                    .map_err(|e| CacheAlignError::FormatError(e.to_string()))
            }
            MessageFormat::GeminiPro => {
                let parts: Vec<HashMap<String, String>> = messages.iter().map(|msg| {
                    let mut part = HashMap::new();
                    part.insert("role".to_string(), msg.role.clone());
                    part.insert("text".to_string(), msg.content.clone());
                    part
                }).collect();
                serde_json::to_string(&parts)
                    .map_err(|e| CacheAlignError::FormatError(e.to_string()))
            }
            MessageFormat::CathedralNative => {
                let mut output = String::new();
                output.push_str("<cathedral>\n");
                for msg in messages {
                    output.push_str(&format!(
                        "<message role=\"{}\">{}</message>\n",
                        msg.role, msg.content
                    ));
                }
                output.push_str("</cathedral>");
                Ok(output)
            }
            MessageFormat::Raw => {
                Ok(messages.iter().map(|m| m.content.clone()).collect::<Vec<_>>().join("\n"))
            }
        }
    }

    /// ============================================================
    /// 2.4 ESTATÍSTICAS
    /// ============================================================

    pub fn get_stats(&self) -> &CacheAlignerStats {
        &self.stats
    }

    pub fn get_cache_hit_rate(&self) -> f64 {
        let total = self.stats.cache_hits + self.stats.cache_misses;
        if total == 0 { 0.0 } else { self.stats.cache_hits as f64 / total as f64 }
    }

    pub fn clear_prefix_cache(&mut self) {
        self.prefix_cache.clear();
    }
}

/// ============================================================
/// 3. TIPOS AUXILIARES
/// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Error)]
pub enum CacheAlignError {
    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),
    #[error("Format error: {0}")]
    FormatError(String),
    #[error("Prefix computation failed: {0}")]
    PrefixComputationFailed(String),
}

fn provider_key(provider: &LlmProvider) -> String {
    match provider {
        LlmProvider::Anthropic => "anthropic".to_string(),
        LlmProvider::OpenAI => "openai".to_string(),
        LlmProvider::Gemini => "gemini".to_string(),
        LlmProvider::Rio35 => "rio35".to_string(),
        LlmProvider::Vllm => "vllm".to_string(),
        LlmProvider::Custom(s) => format!("custom_{}", s),
    }
}

fn sha256(input: &str) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_anthropic() {
        let mut aligner = CacheAligner::new();
        let messages = vec![
            LlmMessage { role: "user".to_string(), content: "Hello".to_string() },
            LlmMessage { role: "assistant".to_string(), content: "Hi there".to_string() },
        ];

        let aligned = aligner.align_messages(&LlmProvider::Anthropic, &messages, "ctx_1").unwrap();
        assert!(aligned.len() >= 2);
        assert_eq!(aligned[0].role, "system");
    }

    #[test]
    fn test_cache_hit() {
        let mut aligner = CacheAligner::new();
        let messages = vec![
            LlmMessage { role: "user".to_string(), content: "Test".to_string() },
        ];

        // Primeira chamada: miss
        let _ = aligner.align_messages(&LlmProvider::OpenAI, &messages, "ctx_2").unwrap();
        assert_eq!(aligner.stats.cache_misses, 1);

        // Segunda chamada: hit
        let _ = aligner.align_messages(&LlmProvider::OpenAI, &messages, "ctx_2").unwrap();
        assert_eq!(aligner.stats.cache_hits, 1);
        assert!(aligner.get_cache_hit_rate() > 0.4);
    }

    #[test]
    fn test_format_conversion() {
        let aligner = CacheAligner::new();
        let messages = vec![
            LlmMessage { role: "system".to_string(), content: "You are helpful".to_string() },
            LlmMessage { role: "user".to_string(), content: "Hello".to_string() },
        ];

        let anthropic = aligner.format_for_provider(&LlmProvider::Anthropic, &messages).unwrap();
        assert!(anthropic.contains("<system>"));
        assert!(anthropic.contains("Human:"));

        let openai = aligner.format_for_provider(&LlmProvider::OpenAI, &messages).unwrap();
        assert!(openai.contains("\"role\""));

        let cathedral = aligner.format_for_provider(&LlmProvider::Rio35, &messages).unwrap();
        assert!(cathedral.contains("<cathedral>"));
    }

    #[test]
    fn test_domain_specific_prefix() {
        let mut aligner = CacheAligner::new();
        let messages = vec![
            LlmMessage { role: "user".to_string(), content: "Audit B20".to_string() },
        ];

        let aligned = aligner.align_messages(&LlmProvider::Rio35, &messages, "audit_ctx").unwrap();
        let system_msg = &aligned[0];
        assert!(system_msg.content.contains("<domain>cathedral_arkhe</domain>"));
        assert!(system_msg.content.contains("<thinking_mode>adaptive</thinking_mode>"));
    }
}
