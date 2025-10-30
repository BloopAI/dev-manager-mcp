mod log_buffer;
mod manager;
mod port_allocator;
mod server_entry;
mod service;

use manager::Manager;
use rmcp::transport::sse_server::SseServer;
use service::DevManagerService;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = Arc::new(Manager::new());
    
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3009);
    
    let bind = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

    println!("MCP daemon listening on {}", bind);
    let server = SseServer::serve(bind).await?;

    let cancel = server.with_service({
        let manager = Arc::clone(&manager);
        move || DevManagerService::new(Arc::clone(&manager))
    });

    tokio::signal::ctrl_c().await?;
    cancel.cancel();
    Ok(())
}
