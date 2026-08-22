pub struct NostrReplicator;

impl NostrReplicator {
    pub fn default_relays(&self) -> &[String] {
        &[]
    }

    pub async fn publish_to_relays(
        &self,
        _event: &nostr_sdk::Event,
        _relays: &[String],
    ) -> Result<nostr_sdk::EventId, nostr_sdk::event::id::Error> {
        nostr_sdk::EventId::from_hex("00")
    }
}
