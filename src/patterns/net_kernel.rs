use crate::transport::traits::{Connection, Transport};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct NodeInfo {
    pub name: String,
    pub address: String,
    pub conn: Arc<Mutex<Box<dyn Connection>>>,
}

pub struct NetKernel {
    pub transport: Arc<dyn Transport>,
    pub nodes: HashMap<String, NodeInfo>,
}

impl NetKernel {
    pub fn new(transport: Arc<dyn Transport>) -> Self {
        Self {
            transport,
            nodes: HashMap::new(),
        }
    }

    pub async fn connect_node(&mut self, name: &str, addr: &str) -> Result<()> {
        let conn = self.transport.connect(addr).await?;
        let info = NodeInfo {
            name: name.to_string(),
            address: addr.to_string(),
            conn: Arc::new(Mutex::new(conn)),
        };
        self.nodes.insert(name.to_string(), info);
        Ok(())
    }

    pub fn get_node(&self, name: &str) -> Option<Arc<Mutex<Box<dyn Connection>>>> {
        self.nodes.get(name).map(|info| info.conn.clone())
    }

    pub fn get_all_nodes(&self) -> Vec<String> {
        self.nodes.keys().cloned().collect()
    }
}
