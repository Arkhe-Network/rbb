use nostr_sdk::{Client, Keys, Event, Kind, Tag};

pub struct NostrRelayClient {
    client: Client,
    keys: Keys,
}

impl NostrRelayClient {
    pub async fn new(relay_url: &str, private_key: &str) -> Result<Self, String> {
        let keys = Keys::parse(private_key).map_err(|e| format!("Invalid keys: {}", e))?;
        let client = Client::new(keys.clone());
        client.add_relay(relay_url).await.map_err(|e| e.to_string())?;
        Ok(Self { client, keys })
    }
}
