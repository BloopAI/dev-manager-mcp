# dev-manager-mcp

MCP Dev Server Manager — manage development servers via npx.

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

## Options

- `--daemon-url <URL>` - Connect to daemon at specific URL (default: `http://127.0.0.1:3009/sse`)
- `PORT=<port>` - Set daemon port when starting daemon mode (e.g., `PORT=3010 npx -y dev-manager-mcp daemon`)
- Version pinning: `npx dev-manager-mcp@0.1.x stdio`

## Documentation

For features, MCP tools, architecture, and advanced usage, see the [main repository README](https://github.com/BloopAI/dev-manager-mcp).

## License

MIT
