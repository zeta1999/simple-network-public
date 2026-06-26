use anyhow::Result;
use bytes::Bytes;

pub struct PipelinePushClient;

impl PipelinePushClient {
    pub async fn push(&self, _work: Bytes) -> Result<()> {
        Err(anyhow::anyhow!("Not implemented"))
    }
}

pub struct PipelinePullWorker;

impl PipelinePullWorker {
    pub async fn pull(&self) -> Result<Bytes> {
        Err(anyhow::anyhow!("Not implemented"))
    }
}
