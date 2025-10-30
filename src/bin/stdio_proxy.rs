use anyhow::Result;
use rmcp::service::RxJsonRpcMessage;
use rmcp::transport::async_rw::AsyncRwTransport;
use rmcp::transport::sse_client::SseClientTransport;
use rmcp::transport::{io::stdio, Transport};
use rmcp::RoleServer;
use serde_json::Value;
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

#[tokio::main]
async fn main() -> Result<()> {
    let daemon_url = std::env::var("MCP_DAEMON_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3009/sse".to_string());

    let client_cwd = std::env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(String::from));

    if let Some(ref cwd) = client_cwd {
        eprintln!("STDIO proxy starting in CWD: {}", cwd);
    }
    eprintln!("STDIO proxy connecting to daemon at {}", daemon_url);

    let (stdin, stdout) = stdio();
    let mut stdio_transport = AsyncRwTransport::<RoleServer, _, _>::new_server(stdin, stdout);
    let mut sse_transport = SseClientTransport::start(daemon_url.as_str()).await?;

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
