use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mcp-dev-manager")]
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

    match cli.command.unwrap_or(Command::Daemon { port: 3009 }) {
        Command::Daemon { port } => mcp_dev_manager::run_daemon(port).await,
        Command::Stdio { daemon_url } => mcp_dev_manager::run_stdio_proxy(&daemon_url).await,
    }
}
