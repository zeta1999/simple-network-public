use anyhow::Result;

pub struct QuicTransportStub;

impl Default for QuicTransportStub {
    fn default() -> Self {
        Self::new()
    }
}

impl QuicTransportStub {
    pub fn new() -> Self {
        Self
    }

    pub async fn connect(&self, _addr: &str) -> Result<()> {
        // TODO: implement quinn-based QUIC connection
        Err(anyhow::anyhow!("QUIC not yet implemented"))
    }
}
