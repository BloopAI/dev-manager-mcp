# MCP Dev Server Manager

A daemon process that runs continuously and accepts multiple concurrent MCP client connections, sharing dev server session state across all clients.

## Features

- **Multi-client daemon** with shared global state
- **STDIO and SSE transport** support
- **Automatic port allocation** starting at 3010 with reuse
- **Log capture** with 512KB ring buffers per server
- **Auto-cleanup** of idle sessions after 60 seconds
- **Process management** with stdout/stderr capture

## Installation & Usage

**Requirements:** Node.js >= 18

The recommended way to use dev-manager-mcp is via `npx`:

### Quick Start

```bash
# STDIO mode (recommended for MCP clients)
npx -y dev-manager-mcp stdio

# Start daemon in foreground
npx -y dev-manager-mcp daemon

# Start daemon in background
npx -y dev-manager-mcp daemon &

# Custom port
npx -y dev-manager-mcp daemon --port 3010
# or via environment variable:
PORT=3010 npx -y dev-manager-mcp daemon
```

**Note:** Use `-y` flag to skip npx prompts. First run downloads the binary (~5MB).

The daemon listens on `http://127.0.0.1:3009` by default.

### Client Configuration

#### STDIO Transport (Recommended)

For Claude Desktop (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "dev-manager": {
      "command": "npx",
      "args": ["dev-manager-mcp", "stdio"]
    }
  }
}
```

The STDIO proxy connects to the daemon at `http://127.0.0.1:3009/sse` by default. To use a different URL:

```json
{
  "mcpServers": {
    "dev-manager": {
      "command": "npx",
      "args": ["dev-manager-mcp", "stdio", "--daemon-url", "http://127.0.0.1:3010/sse"]
    }
  }
}
```

Or use the environment variable:

```json
{
  "mcpServers": {
    "dev-manager": {
      "command": "npx",
      "args": ["dev-manager-mcp", "stdio"],
      "env": {
        "MCP_DAEMON_URL": "http://127.0.0.1:3009/sse"
      }
    }
  }
}
```

#### SSE Transport (Legacy)

For Claude Desktop (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "dev-manager": {
      "url": "http://127.0.0.1:3009/sse"
    }
  }
}
```

Multiple clients can use the same configuration and will share session state.

## MCP Tools

### `start`
Start a development server. Auto-generates a unique 4-character session key.

**Parameters:**
- `command` (string): Shell command to execute (e.g., "npm run dev", "python -m http.server 8080")
- `cwd` (optional string): Working directory for the command. When using STDIO transport, defaults to client's working directory.

**Returns:**
```json
{
  "status": "started",
  "port": 3010,
  "session_key": "A3X9"
}
```

### `stop`
Stop a running development server session.

**Parameters:**
- `session_key` (string): Session identifier

**Returns:**
```json
{
  "status": "stopped",
  "session_key": "A3X9"
}
```

### `status`
Get status of one or all development server sessions.

**Parameters:**
- `session_key` (optional string): Specific session to query, or omit for all sessions

**Returns:**
```json
{
  "sessions": [
    {
      "session_key": "B7K2",
      "port": 3010,
      "running": true
    }
  ]
}
```

### `tail`
Get stdout/stderr logs for a development server session.

**Parameters:**
- `session_key` (string): Session identifier (e.g., "A3X9")

**Returns:**
```json
{
  "session_key": "A3X9",
  "stdout": "Server started on port 3010...",
  "stderr": ""
}
```

## Architecture

### Modules

- **port_allocator.rs** - Sequential port allocation from 3010 with free list
- **log_buffer.rs** - Bounded 512KB ring buffer with Clone support
- **server_entry.rs** - Process wrapper with async log capture
- **manager.rs** - Shared state manager with auto-cleanup sweeper
- **service.rs** - MCP service with tool definitions
- **main.rs** - HTTP/SSE daemon server

### State Management

- Single `Arc<Manager>` shared across all client connections
- Each connection gets a fresh `DevManagerService` instance
- Mutex-protected HashMap for session storage
- Background sweeper runs every 5 seconds to clean up idle sessions (>60s)

### Session Keys

- Auto-generated 4-character uppercase alphanumeric codes (e.g., "A3X9", "K7M2")
- Guaranteed unique across active sessions
- 1,679,616 possible combinations (36^4)

### Port Allocation

- Starts at 3010 and increments sequentially
- Maintains free list for reused ports
- Probes availability with TcpListener before assignment

### Log Capture

- Each server spawns two async tasks for stdout/stderr
- Logs stored in bounded VecDeque with byte tracking
- Oldest entries evicted when 512KB limit reached
- Non-blocking reads with line buffering

## Testing Multi-Client Behavior

1. Start daemon: `npx -y dev-manager-mcp daemon`
2. Connect Client A and start a server session
3. Connect Client B and query status - should see Client A's session
4. Client B can stop Client A's session
5. All clients share the same session state

## Building from Source

If you prefer to build from source or need offline/advanced usage:

```bash
cargo build --release
```

The binary will be at `target/release/dev-manager-mcp`.

To use the built binary, replace `npx dev-manager-mcp` with `./target/release/dev-manager-mcp` in all examples above.

## License

MIT
