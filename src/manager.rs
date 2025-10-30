use crate::port_allocator::PortAllocator;
use crate::server_entry::ServerEntry;
use rand::{thread_rng, Rng};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::process::Command;

type SessionKey = String;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const RUNNING_IDLE_SECS: u64 = 60;
const EXITED_RETENTION_SECS: u64 = 600;

fn generate_session_key() -> String {
    let mut rng = thread_rng();
    (0..4)
        .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
        .collect()
}

struct ManagerInner {
    servers: HashMap<SessionKey, ServerEntry>,
    port_allocator: PortAllocator,
}

#[derive(Clone)]
pub struct Manager {
    inner: Arc<Mutex<ManagerInner>>,
}

impl Manager {
    pub fn new() -> Self {
        let manager = Self {
            inner: Arc::new(Mutex::new(ManagerInner {
                servers: HashMap::new(),
                port_allocator: PortAllocator::new(3010),
            })),
        };

        manager.start_sweeper();
        manager
    }

    fn start_sweeper(&self) {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;

                let (to_stop, to_prune, _ports_to_free) = {
                    let mut guard = match inner.lock() {
                        Ok(g) => g,
                        Err(_) => continue,
                    };

                    let now = Instant::now();
                    let idle_threshold = Duration::from_secs(RUNNING_IDLE_SECS);
                    let retention_threshold = Duration::from_secs(EXITED_RETENTION_SECS);

                    let mut to_stop = Vec::new();
                    let mut to_prune = Vec::new();
                    let mut ports_to_free = Vec::new();

                    for (key, entry) in guard.servers.iter_mut() {
                        entry.poll_exit();

                        if let Some(exited_at) = entry.exited_at() {
                            if entry.port != 0 {
                                ports_to_free.push((key.clone(), entry.port));
                            }
                            if now.duration_since(exited_at) > retention_threshold {
                                to_prune.push(key.clone());
                            }
                        } else if now.duration_since(entry.last_activity) > idle_threshold {
                            to_stop.push(key.clone());
                        }
                    }

                    for (key, port) in &ports_to_free {
                        guard.port_allocator.free(*port);
                        if let Some(entry) = guard.servers.get_mut(key) {
                            entry.port = 0;
                        }
                    }

                    (to_stop, to_prune, ports_to_free)
                };

                for key in to_stop {
                    let entry_opt = {
                        let mut guard = match inner.lock() {
                            Ok(g) => g,
                            Err(_) => continue,
                        };
                        guard.servers.remove(&key)
                    };

                    if let Some(mut entry) = entry_opt {
                        let port = entry.port;
                        let _ = entry.stop().await;
                        entry.port = 0;

                        let mut guard = match inner.lock() {
                            Ok(g) => g,
                            Err(_) => continue,
                        };
                        guard.port_allocator.free(port);
                        guard.servers.insert(key, entry);
                    }
                }

                for key in to_prune {
                    let mut guard = match inner.lock() {
                        Ok(g) => g,
                        Err(_) => continue,
                    };
                    if let Some(entry) = guard.servers.remove(&key) {
                        if entry.port != 0 {
                            guard.port_allocator.free(entry.port);
                        }
                    }
                }
            }
        });
    }

    pub async fn start(&self, command: String, cwd: Option<String>) -> serde_json::Value {
        let (session_key, port) = {
            let mut guard = match self.inner.lock() {
                Ok(g) => g,
                Err(e) => return json!({"error": format!("Lock error: {}", e)}),
            };

            let session_key = loop {
                let key = generate_session_key();
                if !guard.servers.contains_key(&key) {
                    break key;
                }
            };

            let port = match guard.port_allocator.allocate() {
                Ok(p) => p,
                Err(e) => return json!({"error": format!("Port allocation failed: {}", e)}),
            };

            (session_key, port)
        };

        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.arg("/C").arg(&command);
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c").arg(&command);
            c
        };
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        if let Some(ref cwd_path) = cwd {
            let path = std::path::Path::new(cwd_path);
            if !path.is_dir() {
                let mut guard = self.inner.lock().unwrap();
                guard.port_allocator.free(port);
                return json!({"error": format!("Invalid cwd: {}", cwd_path)});
            }
            cmd.current_dir(path);
        }

        cmd.env("PORT", port.to_string());

        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let mut guard = self.inner.lock().unwrap();
                guard.port_allocator.free(port);
                return json!({"error": format!("Failed to spawn process: {}", e)});
            }
        };

        let entry = ServerEntry::new(child, port);

        {
            let mut guard = match self.inner.lock() {
                Ok(g) => g,
                Err(e) => return json!({"error": format!("Lock error: {}", e)}),
            };
            guard.servers.insert(session_key.clone(), entry);
        }

        json!({
            "status": "started",
            "port": port,
            "session_key": session_key
        })
    }

    pub async fn stop(&self, session_key: String) -> serde_json::Value {
        let mut entry = {
            let mut guard = match self.inner.lock() {
                Ok(g) => g,
                Err(e) => return json!({"error": format!("Lock error: {}", e)}),
            };

            match guard.servers.remove(&session_key) {
                Some(e) => e,
                None => return json!({"error": "Session not found"}),
            }
        };

        let port = entry.port;
        entry.last_activity = Instant::now();

        match entry.stop().await {
            Ok(_) => {
                entry.port = 0;

                let mut guard = match self.inner.lock() {
                    Ok(g) => g,
                    Err(e) => return json!({"error": format!("Lock error: {}", e)}),
                };

                guard.port_allocator.free(port);
                guard.servers.insert(session_key.clone(), entry);

                json!({"status": "stopped", "session_key": session_key})
            }
            Err(e) => json!({"error": format!("Failed to stop server: {}", e)}),
        }
    }

    pub fn status(&self, session_key: Option<String>) -> serde_json::Value {
        let mut guard = match self.inner.lock() {
            Ok(g) => g,
            Err(e) => return json!({"error": format!("Lock error: {}", e)}),
        };

        if let Some(key) = session_key {
            if let Some(entry) = guard.servers.get_mut(&key) {
                entry.last_activity = Instant::now();
                let mut result = json!({
                    "session_key": key,
                    "running": entry.is_running()
                });
                if entry.port != 0 {
                    result["port"] = json!(entry.port);
                }
                return result;
            } else {
                return json!({"error": "Session not found"});
            }
        }

        let sessions: Vec<_> = guard
            .servers
            .iter_mut()
            .map(|(key, entry)| {
                entry.last_activity = Instant::now();
                let mut result = json!({
                    "session_key": key,
                    "running": entry.is_running()
                });
                if entry.port != 0 {
                    result["port"] = json!(entry.port);
                }
                result
            })
            .collect();

        json!({"sessions": sessions})
    }

    pub fn tail(&self, session_key: String) -> serde_json::Value {
        let mut guard = match self.inner.lock() {
            Ok(g) => g,
            Err(e) => return json!({"error": format!("Lock error: {}", e)}),
        };

        if let Some(entry) = guard.servers.get_mut(&session_key) {
            entry.last_activity = Instant::now();
            let (stdout, stderr) = entry.get_logs();
            json!({
                "session_key": session_key,
                "stdout": stdout,
                "stderr": stderr
            })
        } else {
            json!({"error": "Session not found"})
        }
    }
}
