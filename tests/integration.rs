use anyhow::Result;
use bytes::Bytes;
use simple_network::patterns::gen_server::{GenServer, GenServerClient, GenServerHandler};
use simple_network::patterns::net_kernel::NetKernel;
use simple_network::transport::tcp::TcpTransport;
use simple_network::transport::traits::Transport;
use std::sync::Arc;

struct PingHandler;

#[async_trait::async_trait]
impl GenServerHandler for PingHandler {
    async fn handle_call(&self, request: Bytes) -> Result<Bytes> {
        let msg = String::from_utf8(request.to_vec())?;
        if msg == "ping" {
            Ok(Bytes::from("pong"))
        } else {
            Ok(Bytes::from("unknown"))
        }
    }

    async fn handle_cast(&self, _message: Bytes) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_gen_server_rpc() -> Result<()> {
    let transport = TcpTransport;
    let listener = transport.bind("127.0.0.1:0").await?;
    let local_addr = listener.local_addr()?.to_string();

    let handler = Arc::new(PingHandler);
    GenServer::start_link(listener, handler).await?;

    let mut kernel = NetKernel::new(Arc::new(transport));
    kernel.connect_node("node2", &local_addr).await?;

    let conn = kernel.get_node("node2").unwrap();
    let client = GenServerClient::from_arc(conn);

    let resp = client.call(Bytes::from("ping")).await?;
    assert_eq!(resp, Bytes::from("pong"));

    Ok(())
}
