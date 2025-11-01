# dev-manager-mcp

MCP Dev Server Manager — manage development servers via npx.

## Requirements

Node.js >= 18

## Quick Start

```bash
# STDIO mode (recommended for MCP clients like Claude Desktop)
npx -y dev-manager-mcp stdio

# Start daemon (optional)
npx -y dev-manager-mcp daemon
```

## Claude Desktop Configuration

Add to your `claude_desktop_config.json`:

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

### Custom Daemon URL

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

Or via environment variable:

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

## Options

- `--daemon-url <URL>` - Connect to daemon at specific URL (default: `http://127.0.0.1:3009/sse`)
- `PORT=<port>` - Set daemon port when starting daemon mode (e.g., `PORT=3010 npx -y dev-manager-mcp daemon`)
- Version pinning: `npx dev-manager-mcp@0.1.x stdio`

## Documentation

For features, MCP tools, architecture, and advanced usage, see the [main repository README](https://github.com/BloopAI/dev-manager-mcp).

## License

MIT
