//! Cathedral ARKHE v28.2 — ACP ↔ Event Bus Bridge
//! Publica mensagens do barramento de agentes (orquestrador) no Cathedral Event Bus,
//! usando o formato ACP (Agent Communication Protocol) com assinatura SPHINCS+.
//!
//! Selo: CATHEDRAL-ARKHE-v28.2-ACP-EVENT-BRIDGE-2026-06-16

use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use redis::{AsyncCommands, Client as RedisClient};
use crate::orchestrator::{AgentMessage, AgentResponse};
use crate::crypto::sphincs::SphincsSigner;

/// Estrutura de envelope ACP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpEnvelope {
    pub protocol_version: u8,      // 1
    pub message_id: String,
    pub sender: String,
    pub recipient: String,         // "orchestrator" ou agente específico
    pub timestamp: u64,
    pub ttl_secs: u32,
    pub payload_type: String,      // "AgentMessage" | "AgentResponse"
    pub payload: serde_json::Value,
    pub sphincs_signature: String,
    pub temporal_hash: Option<String>,
}

/// Publicador para o Cathedral Event Bus
pub struct AcpEventBusPublisher {
    redis_client: RedisClient,
    signer: Arc<SphincsSigner>,
    channel_prefix: String,
}

impl AcpEventBusPublisher {
    pub fn new(redis_url: &str, signer: Arc<SphincsSigner>, channel_prefix: &str) -> Self {
        let client = RedisClient::open(redis_url).expect("Failed to connect to Redis");
        Self {
            redis_client: client,
            signer,
            channel_prefix: channel_prefix.to_string(),
        }
    }

    /// Converte uma mensagem interna do orquestrador em envelope ACP e publica.
    pub async fn publish_message(&self, msg: &AgentMessage) -> Result<(), String> {
        let envelope = self.create_envelope(
            &msg.msg_id,
            &msg.source,
            &msg.target,
            "AgentMessage",
            serde_json::to_value(msg).map_err(|e| e.to_string())?,
        ).await?;
        self.publish_envelope(&envelope).await
    }

    pub async fn publish_response(&self, resp: &AgentResponse) -> Result<(), String> {
        let envelope = self.create_envelope(
            &resp.msg_id,
            &resp.source,
            "orchestrator",
            "AgentResponse",
            serde_json::to_value(resp).map_err(|e| e.to_string())?,
        ).await?;
        self.publish_envelope(&envelope).await
    }

    async fn create_envelope(
        &self,
        msg_id: &str,
        sender: &str,
        recipient: &str,
        payload_type: &str,
        payload: serde_json::Value,
    ) -> Result<AcpEnvelope, String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut envelope = AcpEnvelope {
            protocol_version: 1,
            message_id: msg_id.to_string(),
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            timestamp,
            ttl_secs: 60,
            payload_type: payload_type.to_string(),
            payload,
            sphincs_signature: String::new(),
            temporal_hash: None,
        };
        // Assinar o envelope (excluindo o campo signature)
        let signature = self.signer.sign(&serde_json::to_vec(&envelope).unwrap())
            .map_err(|e| e.to_string())?;
        envelope.sphincs_signature = hex::encode(signature);
        Ok(envelope)
    }

    async fn publish_envelope(&self, envelope: &AcpEnvelope) -> Result<(), String> {
        let channel = format!("{}/{}", self.channel_prefix, envelope.payload_type);
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| e.to_string())?;
        let payload = serde_json::to_string(envelope).map_err(|e| e.to_string())?;

        // Publish to Pub/Sub for real-time delivery
        let _: () = conn.publish(&channel, &payload).await
            .map_err(|e| e.to_string())?;

        // Append to Stream for event replay
        let stream_key = format!("{}/stream", self.channel_prefix);
        let _: () = redis::cmd("XADD")
            .arg(&stream_key)
            .arg("*")
            .arg("data")
            .arg(&payload)
            .query_async(&mut conn).await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

/// Consumidor (subscriber) opcional para receber eventos externos
pub struct AcpEventBusSubscriber {
    redis_client: RedisClient,
    channel_prefix: String,
}

impl AcpEventBusSubscriber {
    pub fn new(redis_url: &str, channel_prefix: &str) -> Self {
        Self {
            redis_client: RedisClient::open(redis_url).unwrap(),
            channel_prefix: channel_prefix.to_string(),
        }
    }

    pub async fn subscribe(&self, payload_type: &str) -> Result<mpsc::UnboundedReceiver<AcpEnvelope>, String> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| e.to_string())?;
        let channel = format!("{}/{}", self.channel_prefix, payload_type);
        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe(&channel).await
            .map_err(|e| e.to_string())?;
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut pubsub = pubsub;
            while let Ok(msg) = pubsub.on_message().await {
                let payload: String = msg.get_payload().unwrap();
                if let Ok(envelope) = serde_json::from_str(&payload) {
                    let _ = tx.send(envelope);
                }
            }
        });
        Ok(rx)
    }
}

/// Integração com o orquestrador existente: publica cada mensagem no Event Bus
pub async fn attach_to_orchestrator(
    orchestrator: &mut crate::orchestrator::MultiAgentOrchestrator,
    publisher: Arc<AcpEventBusPublisher>,
) {
    let publisher_clone = publisher.clone();
    // Hook no message bus do orquestrador (assumindo que o orquestrador tem um método para registrar callback)
    // Exemplo: orquestrador.set_on_message(move |msg| publisher_clone.publish_message(msg));
    // Como o orquestrador atual não expõe esse hook, aqui mostramos como seria a adição.
    // Na prática, seria necessário modificar o orquestrador para notificar um canal de eventos.
    println!("ACP Event Bus bridge attached (stub)");
}