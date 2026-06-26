use anyhow::Result;

#[async_trait::async_trait]
pub trait SecurityPlugin: Send + Sync {
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
    async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Establishes keys or parameters for the connection session
    async fn handshake(&mut self) -> Result<()>;
}
