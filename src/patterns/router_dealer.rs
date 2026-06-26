use anyhow::Result;
use bytes::Bytes;

pub struct Router;

impl Router {
    pub async fn route_message(&self, _target: &str, _message: Bytes) -> Result<()> {
        Err(anyhow::anyhow!("Not implemented"))
    }
}

pub struct Dealer;

impl Dealer {
    pub async fn receive_and_reply(&self) -> Result<(Bytes, String)> {
        Err(anyhow::anyhow!("Not implemented"))
    }
}
