use anyhow::Result;

pub struct PortForwardStub;

impl PortForwardStub {
    pub async fn setup_forwarding(_local_port: u16, _remote_addr: &str) -> Result<()> {
        Err(anyhow::anyhow!("Port forwarding not yet implemented"))
    }
}
