
pub mod transport;

pub enum TransportId {
    QUIC,
    WebSocket,
    Nostr,
    Bluetooth,
    UsbSerial,
    LoRa,
    NFC,
    ZeroCopy,
}

#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    async fn connect(&self, address: &str) -> Result<(), ()>;
    async fn send(&self, data: &[u8]) -> Result<(), ()>;
    async fn receive(&self) -> Result<Vec<u8>, ()>;
    fn mtu(&self) -> usize;
    fn name(&self) -> &'static str;
}
