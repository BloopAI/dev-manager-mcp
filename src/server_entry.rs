use crate::log_buffer::LogBuffer;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::time::{timeout, Duration};

enum ProcessState {
    Running(Child),
    Exited { exited_at: Instant },
}

pub struct ServerEntry {
    state: ProcessState,
    pub port: u16,
    pub last_activity: Instant,
    stdout_log: Arc<Mutex<LogBuffer>>,
    stderr_log: Arc<Mutex<LogBuffer>>,
}

impl ServerEntry {
    pub fn new(mut child: Child, port: u16) -> Self {
        let stdout_log = Arc::new(Mutex::new(LogBuffer::new()));
        let stderr_log = Arc::new(Mutex::new(LogBuffer::new()));

        if let Some(stdout) = child.stdout.take() {
            let log = stdout_log.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if let Ok(mut buffer) = log.lock() {
                        buffer.push(line + "\n");
                    }
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let log = stderr_log.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if let Ok(mut buffer) = log.lock() {
                        buffer.push(line + "\n");
                    }
                }
            });
        }

        Self {
            state: ProcessState::Running(child),
            port,
            last_activity: Instant::now(),
            stdout_log,
            stderr_log,
        }
    }

    pub fn poll_exit(&mut self) -> bool {
        if let ProcessState::Running(child) = &mut self.state {
            if let Ok(Some(_)) = child.try_wait() {
                self.state = ProcessState::Exited {
                    exited_at: Instant::now(),
                };
                return true;
            }
        }
        false
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, ProcessState::Running(_))
    }

    pub fn exited_at(&self) -> Option<Instant> {
        if let ProcessState::Exited { exited_at } = self.state {
            Some(exited_at)
        } else {
            None
        }
    }

    pub async fn stop(&mut self) -> anyhow::Result<()> {
        if let ProcessState::Running(child) = &mut self.state {
            child.kill().await?;
            
            match timeout(Duration::from_secs(5), child.wait()).await {
                Ok(Ok(_)) => {},
                Ok(Err(e)) => return Err(e.into()),
                Err(_) => anyhow::bail!("Timeout waiting for process to exit"),
            }

            self.state = ProcessState::Exited {
                exited_at: Instant::now(),
            };
        }
        Ok(())
    }

    pub fn get_logs(&self) -> (String, String) {
        let stdout = if let Ok(buffer) = self.stdout_log.lock() {
            buffer.tail().0
        } else {
            String::new()
        };

        let stderr = if let Ok(buffer) = self.stderr_log.lock() {
            buffer.tail().0
        } else {
            String::new()
        };

        (stdout, stderr)
    }
}
