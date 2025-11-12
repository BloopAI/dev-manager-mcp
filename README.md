# MCP Dev Server Manager

A daemon that accepts requests from MCP clients to start dev servers, allocating unique ports to avoid collisions and shutting down idle connections after 120s of inactivity.

## Example

- User starts daemon
- User's first MCP client (eg coding CLI) requests to run a dev server
  - Dev server is started on PORT 3010
- User's second MCP client (eg coding CLI) requests to run a dev server
  - Dev server is started on PORT 3011

## Features

- **Run multiple dev servers in parallel**: useful if you want to use automated testing tools eg [Playwright](https://github.com/microsoft/playwright-mcp) or [Google Dev Tools MCP](https://developer.chrome.com/blog/chrome-devtools-mcp)
- **Avoid port collisions**: when working with websites, it's often necessary to specify different ports if you want to run multiple dev servers
- **Automatic port allocation** starting at 3010 with reuse
- **Log capture** with 512KB ring buffers per server
- **Auto-cleanup** of idle sessions after 120 seconds (configurable via `--idle-timeout`)

## Installation & Usage

**Requirements:** Node.js >= 18

The recommended way to use dev-manager-mcp is via `npx`:

### Quick Start

Make sure the daemon is running in a terminal somewhere:

```bash
npx -y dev-manager-mcp
```

Add this MCP config to your coding CLI:

```bash
{
  "mcpServers": {
    "dev-manager": {
      "command": "npx",
      "args": ["dev-manager-mcp", "stdio"]
    }
  }
}
```

Finally, ask your coding CLI to start a dev server. You should see it use the MCP server.

### Agent-Specific Installation

**Standard config** works in most tools:

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

<details>
<summary>Amp</summary>

Add via the Amp VS Code extension settings screen or by updating your settings.json file:

```json
"amp.mcpServers": {
  "dev-manager": {
    "command": "npx",
    "args": ["dev-manager-mcp", "stdio"]
  }
}
```

**Amp CLI Setup:**

Add via the `amp mcp add` command below

```bash
amp mcp add dev-manager -- npx dev-manager-mcp stdio
```

</details>

<details>
<summary>Claude Code</summary>

Use the Claude Code CLI to add the dev-manager MCP server:

```bash
claude mcp add dev-manager npx dev-manager-mcp stdio
```
</details>

<details>
<summary>Claude Desktop</summary>

Follow the MCP install [guide](https://modelcontextprotocol.io/quickstart/user), use the standard config above.

</details>

<details>
<summary>Codex</summary>

Use the Codex CLI to add the dev-manager MCP server:

```bash
codex mcp add dev-manager npx "dev-manager-mcp" "stdio"
```

Alternatively, create or edit the configuration file `~/.codex/config.toml` and add:

```toml
[mcp_servers.dev-manager]
command = "npx"
args = ["dev-manager-mcp", "stdio"]
```

For more information, see the [Codex MCP documentation](https://github.com/openai/codex/blob/main/codex-rs/config.md#mcp_servers).

</details>

<details>
<summary>Cursor</summary>

Go to `Cursor Settings` -> `MCP` -> `Add new MCP Server`. Name to your liking, use `command` type with the command `npx dev-manager-mcp stdio`. You can also verify config or add command arguments via clicking `Edit`.

</details>

<details>
<summary>Factory</summary>

Use the Factory CLI to add the dev-manager MCP server:

```bash
droid mcp add dev-manager "npx dev-manager-mcp stdio"
```

Alternatively, type `/mcp` within Factory droid to open an interactive UI for managing MCP servers.

For more information, see the [Factory MCP documentation](https://docs.factory.ai/cli/configuration/mcp).

</details>

<details>
<summary>Gemini CLI</summary>

Follow the MCP install [guide](https://github.com/google-gemini/gemini-cli/blob/main/docs/tools/mcp-server.md#configure-the-mcp-server-in-settingsjson), use the standard config above.

</details>

<details>
<summary>Goose</summary>

Go to `Advanced settings` -> `Extensions` -> `Add custom extension`. Name to your liking, use type `STDIO`, and set the `command` to `npx dev-manager-mcp stdio`. Click "Add Extension".
</details>

<details>
<summary>Kiro</summary>

Follow the MCP Servers [documentation](https://kiro.dev/docs/mcp/). For example in `.kiro/settings/mcp.json`:

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
</details>

<details>
<summary>LM Studio</summary>

Go to `Program` in the right sidebar -> `Install` -> `Edit mcp.json`. Use the standard config above.
</details>

<details>
<summary>opencode</summary>

Follow the MCP Servers [documentation](https://opencode.ai/docs/mcp-servers/). For example in `~/.config/opencode/opencode.json`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "dev-manager": {
      "type": "local",
      "command": [
        "npx",
        "dev-manager-mcp",
        "stdio"
      ],
      "enabled": true
    }
  }
}
```
</details>

<details>
<summary>Qodo Gen</summary>

Open [Qodo Gen](https://docs.qodo.ai/qodo-documentation/qodo-gen) chat panel in VSCode or IntelliJ → Connect more tools → + Add new MCP → Paste the standard config above.

Click <code>Save</code>.
</details>

<details>
<summary>VS Code</summary>

Follow the MCP install [guide](https://code.visualstudio.com/docs/copilot/chat/mcp-servers#_add-an-mcp-server), use the standard config above. You can also install the dev-manager MCP server using the VS Code CLI:

```bash
# For VS Code
code --add-mcp '{"name":"dev-manager","command":"npx","args":["dev-manager-mcp","stdio"]}'
```

After installation, the dev-manager MCP server will be available for use with your GitHub Copilot agent in VS Code.
</details>

<details>
<summary>Warp</summary>

Go to `Settings` -> `AI` -> `Manage MCP Servers` -> `+ Add` to [add an MCP Server](https://docs.warp.dev/knowledge-and-collaboration/mcp#adding-an-mcp-server). Use the standard config above.

Alternatively, use the slash command `/add-mcp` in the Warp prompt and paste the standard config from above:
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

</details>

<details>
<summary>Windsurf</summary>

Follow Windsurf MCP [documentation](https://docs.windsurf.com/windsurf/cascade/mcp). Use the standard config above.

</details>

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
- Background sweeper runs every 5 seconds to clean up idle sessions (>120s by default)

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
