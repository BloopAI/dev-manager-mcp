mod log_buffer;
mod manager;
mod port_allocator;
mod server_entry;
mod service;

use anyhow::Result;
use manager::Manager;
use rmcp::service::RxJsonRpcMessage;
use rmcp::transport::async_rw::AsyncRwTransport;
use rmcp::transport::sse_client::SseClientTransport;
use rmcp::transport::sse_server::SseServer;
use rmcp::transport::{io::stdio, Transport};
use rmcp::RoleServer;
use serde_json::Value;
use service::DevManagerService;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

fn inject_cwd_if_start_tool(
    msg: RxJsonRpcMessage<RoleServer>,
    client_cwd: Option<String>,
) -> RxJsonRpcMessage<RoleServer> {
    let client_cwd = match client_cwd {
        Some(cwd) => cwd,
        None => return msg,
    };

    let mut msg_value = match serde_json::to_value(&msg) {
        Ok(v) => v,
        Err(_) => return msg,
    };

    if let Some(params) = msg_value.get_mut("params").and_then(|p| p.as_object_mut()) {
        if let Some(name) = params.get("name").and_then(|n| n.as_str()) {
            if name == "start" {
                if let Some(arguments) = params.get_mut("arguments").and_then(|a| a.as_object_mut()) {
                    if !arguments.contains_key("cwd") {
                        arguments.insert("cwd".to_string(), Value::String(client_cwd));
                        return serde_json::from_value(msg_value).unwrap_or(msg);
                    }
                }
            }
        }
    }

    msg
}

pub async fn run_daemon(port: u16, idle_timeout_secs: u64) -> Result<()> {
    let manager = Arc::new(Manager::new(Duration::from_secs(idle_timeout_secs)));
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

pub async fn run_stdio_proxy(daemon_url: &str) -> Result<()> {
    let client_cwd = std::env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(String::from));

    if let Some(ref cwd) = client_cwd {
        eprintln!("STDIO proxy starting in CWD: {}", cwd);
    }
    eprintln!("STDIO proxy connecting to daemon at {}", daemon_url);

    let (stdin, stdout) = stdio();
    let mut stdio_transport = AsyncRwTransport::<RoleServer, _, _>::new_server(stdin, stdout);
    let mut sse_transport = SseClientTransport::start(daemon_url).await?;

    let (req_tx, mut req_rx) = mpsc::unbounded_channel();
    let (resp_tx, mut resp_rx) = mpsc::unbounded_channel();

    let stdio_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                msg = stdio_transport.receive() => {
                    match msg {
                        Some(c2s) => {
                            let modified_msg = inject_cwd_if_start_tool(c2s, client_cwd.clone());
                            let _ = req_tx.send(modified_msg);
                        }
                        None => break,
                    }
                }
                Some(s2c) = resp_rx.recv() => {
                    if let Err(e) = stdio_transport.send(s2c).await {
                        eprintln!("STDIO send error: {}", e);
                        break;
                    }
                }
            }
        }
    });

    let sse_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                msg = sse_transport.receive() => {
                    match msg {
                        Some(s2c) => { let _ = resp_tx.send(s2c); }
                        None => break,
                    }
                }
                Some(c2s) = req_rx.recv() => {
                    if let Err(e) = sse_transport.send(c2s).await {
                        eprintln!("SSE send error: {}", e);
                        break;
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = stdio_task => eprintln!("STDIO task ended"),
        _ = sse_task => eprintln!("SSE task ended"),
        _ = tokio::signal::ctrl_c() => eprintln!("Interrupted"),
    }

    Ok(())
}
