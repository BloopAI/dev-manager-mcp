use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dev-manager-mcp")]
#[command(about = "MCP development server manager with shared session state")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Run as HTTP/SSE daemon server (default)")]
    Daemon {
        #[arg(long, env = "PORT", default_value_t = 3009)]
        port: u16,
        #[arg(long, env = "MCP_IDLE_TIMEOUT", default_value_t = 120)]
        idle_timeout: u64,
    },
    #[command(about = "Run as STDIO proxy that connects to daemon")]
    Stdio {
        #[arg(
            long,
            env = "MCP_DAEMON_URL",
            default_value = "http://127.0.0.1:3009/sse"
        )]
        daemon_url: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Daemon { port: 3009, idle_timeout: 120 }) {
        Command::Daemon { port, idle_timeout } => dev_manager_mcp::run_daemon(port, idle_timeout).await,
        Command::Stdio { daemon_url } => dev_manager_mcp::run_stdio_proxy(&daemon_url).await,
    }
}
