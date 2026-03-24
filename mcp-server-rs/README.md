# webMethods Integration Server MCP Server (Rust)

A Model Context Protocol (MCP) server that enables AI assistants to manage webMethods Integration Server instances through **pure HTTP APIs**. Single binary, no runtime dependencies.

## Architecture

```
Claude Code / AI Assistant
        |
        | MCP Protocol (stdio)
        v
  MCP Server (Rust binary)
        |
        | HTTP/JSON (Basic Auth)
        v
  webMethods IS instance(s)
    dev  (host-a:5555)
    prod (host-b:5555)
    ...
```

## Installation

### From source (cargo)

```bash
cargo install --git https://github.com/cpoder/wm-mcp-server.git --root ~/.local
```

### From pre-built binary

Download from [Releases](https://github.com/cpoder/wm-mcp-server/releases) and place in your PATH.

### Build locally

```bash
git clone https://github.com/cpoder/wm-mcp-server.git
cd wm-mcp-server/mcp-server-rs
cargo build --release
# Binary at target/release/wm-mcp-server
```

## Configuration

### Single instance (environment variables)

Add to your `.mcp.json` (Claude Code, IBM Bob, or any MCP client):

```json
{
  "mcpServers": {
    "webmethods-is": {
      "command": "wm-mcp-server",
      "env": {
        "WM_IS_URL": "http://your-is-host:5555",
        "WM_IS_USER": "Administrator",
        "WM_IS_PASSWORD": "your-password"
      }
    }
  }
}
```

| Variable | Default | Description |
|----------|---------|-------------|
| `WM_IS_URL` | `http://localhost:5555` | IS base URL |
| `WM_IS_USER` | `Administrator` | IS username |
| `WM_IS_PASSWORD` | `manage` | IS password |
| `WM_IS_TIMEOUT` | `30` | HTTP request timeout (seconds) |

### Multiple instances (config file)

Create a JSON config file (e.g., `wm-instances.json`):

```json
{
  "instances": {
    "dev": {
      "url": "http://dev-host:5555",
      "user": "Administrator",
      "password": "manage"
    },
    "prod": {
      "url": "http://prod-host:5555",
      "user": "Administrator",
      "password": "secret",
      "timeout": 60
    }
  },
  "default": "dev"
}
```

Point to it via `WM_CONFIG`:

```json
{
  "mcpServers": {
    "webmethods-is": {
      "command": "wm-mcp-server",
      "env": {
        "WM_CONFIG": "/path/to/wm-instances.json"
      }
    }
  }
}
```

Every tool accepts an optional `instance` parameter to target a specific server. Omit it to use the default. Use `list_instances` to see all configured instances.

## Tools (40 total)

### Instance Management (1)

| Tool | Description |
|------|-------------|
| `list_instances` | List all configured IS instances and which is the default |

### Server Management (2)

| Tool | Description |
|------|-------------|
| `is_status` | Check if the IS is running and get server info |
| `is_shutdown(bounce)` | Shutdown or restart the IS |

### Package Management (5)

| Tool | Description |
|------|-------------|
| `package_list` | List all packages with enabled status |
| `package_create(package_name)` | Create and activate a new package |
| `package_reload(package_name)` | Reload a package to pick up changes |
| `package_enable(package_name)` | Enable a disabled package |
| `package_disable(package_name)` | Disable a package |

### Namespace Browsing (4)

| Tool | Description |
|------|-------------|
| `node_list(package, folder?)` | List services, folders, documents in a package/folder |
| `node_get(name)` | Get full definition of any node |
| `node_delete(name)` | Delete a node |
| `folder_create(package, folder_path)` | Create a namespace folder |

### Flow Service Development (4)

| Tool | Description |
|------|-------------|
| `flow_service_create(package, service_path)` | Create an empty flow service shell |
| `put_node(node_data)` | **Core API** -- create or update a flow service with full signature and flow logic |
| `service_invoke(service_path, inputs?)` | Execute/test any service |
| `mapset_value(value)` | Helper to encode a value for MAPSET data field |

### Document Types (1)

| Tool | Description |
|------|-------------|
| `document_type_create(package, doc_path)` | Create a document type |

### Port / Listener Management (8)

| Tool | Description |
|------|-------------|
| `port_list` | List all ports |
| `port_factory_list` | List available listener factory types |
| `port_get(port_key, pkg)` | Get detailed config for a specific port |
| `port_add(settings)` | Add a new port |
| `port_update(port_key, pkg, settings)` | Update port configuration |
| `port_enable(port_key, pkg)` | Enable a port |
| `port_disable(port_key, pkg)` | Disable a port |
| `port_delete(port_key, pkg)` | Delete a port |

### Adapter Management (15)

| Tool | Description |
|------|-------------|
| `adapter_type_list` | List registered adapter types |
| `adapter_connection_metadata(adapter_type, factory)` | Get available settings for a connection type |
| `adapter_connection_list` | List all adapter connections |
| `adapter_connection_create(...)` | Create a JDBC, SAP, or OPC connection |
| `adapter_connection_enable(alias)` | Enable a connection |
| `adapter_connection_disable(alias)` | Disable a connection |
| `adapter_connection_state(alias)` | Query connection state |
| `adapter_service_create(...)` | Create an adapter service |
| `adapter_listener_list(adapter_type)` | List adapter listeners for a type |
| `adapter_listener_create(...)` | Create a listener |
| `adapter_listener_enable(alias)` | Enable a listener |
| `adapter_listener_disable(alias)` | Disable a listener |
| `adapter_notification_list(adapter_type)` | List polling notifications for a type |
| `adapter_notification_create_polling(...)` | Create a polling notification |
| `adapter_notification_create_listener_based(...)` | Create a listener-based notification |

