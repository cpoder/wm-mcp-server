# wm-mcp-server

MCP server for [webMethods Integration Server](https://www.ibm.com/docs/en/webmethods-integration/wm-integration-server/11.1.0) — 336 tools replacing Designer for development and administration.

## Quick Start

```bash
npx @wm-mcp-server/cli
```

Or install globally:

```bash
npm install -g @wm-mcp-server/cli
```

## MCP Client Configuration

Add to `.mcp.json` in your project:

```json
{
  "mcpServers": {
    "webmethods-is": {
      "command": "npx",
      "args": ["-y", "@wm-mcp-server/cli"],
      "env": {
        "WM_IS_URL": "http://localhost:5555",
        "WM_IS_USER": "Administrator",
        "WM_IS_PASSWORD": "manage"
      }
    }
  }
}
```

## Alternative Installation

```bash
# From crates.io (requires Rust)
cargo install wm-mcp-server

# Pre-built binaries
# https://github.com/cpoder/wm-mcp-server/releases
```

## Documentation

See the [full documentation](https://github.com/cpoder/wm-mcp-server) on GitHub.
