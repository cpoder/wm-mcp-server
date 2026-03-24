# webMethods Integration Server MCP Server

[![CI](https://github.com/cpoder/wm-mcp-server/actions/workflows/ci.yml/badge.svg)](https://github.com/cpoder/wm-mcp-server/actions/workflows/ci.yml)

An [MCP (Model Context Protocol)](https://modelcontextprotocol.io/) server that gives AI assistants full control over [webMethods Integration Server](https://www.ibm.com/docs/en/webmethods-integration/wm-integration-server/11.1.0) through **pure HTTP APIs**. Single binary, no runtime dependencies. Works with any remote IS instance.

Compatible with any [MCP client](https://modelcontextprotocol.io/) (IBM Bob, Claude Code, Claude Desktop, Cursor, Windsurf, etc.).

## What can it do?

| Capability | Examples |
|---|---|
| **Flow services** | Create services with full logic (INVOKE, MAP, BRANCH, LOOP), set input/output signatures, test them -- all in one API call |
| **Packages** | Create, reload, enable, disable |
| **Ports** | Manage HTTP, FTP, FTPS, FilePolling, Email, WebSocket listeners |
| **JDBC adapters** | Connect to databases (SQL Server, PostgreSQL, Oracle, ...), create CustomSQL / Select / Insert services, configure polling notifications |
| **SAP adapters** | Create SAP connections, RFC listeners, IDoc notifications |
| **OPC UA adapters** | Create OPC connections, subscription listeners |
| **Server admin** | Check status, shutdown, restart |

### 40 tools across 8 categories

```
Instances       list_instances
Server          is_status, is_shutdown
Packages        package_list, package_create, package_reload, package_enable, package_disable
Namespace       node_list, node_get, node_delete, folder_create
Services        flow_service_create, put_node, service_invoke, mapset_value, document_type_create
Ports           port_list, port_factory_list, port_get, port_add, port_update, port_enable, port_disable, port_delete
Adapters        adapter_type_list, adapter_connection_metadata, adapter_connection_list,
                adapter_connection_create, adapter_connection_enable, adapter_connection_disable,
                adapter_connection_state, adapter_service_create, adapter_listener_list,
                adapter_listener_create, adapter_listener_enable, adapter_listener_disable,
                adapter_notification_list, adapter_notification_create_polling,
                adapter_notification_create_listener_based
```

Supports **multiple IS instances** -- configure via a JSON file and pass `instance` to any tool to target a specific server.

## Quick Start

### 1. Install

**Option A: cargo install (from source)**

```bash
cargo install --git https://github.com/cpoder/wm-mcp-server.git
```

**Option B: build locally**

```bash
git clone https://github.com/cpoder/wm-mcp-server.git
cd wm-mcp-server/mcp-server-rs
cargo build --release
# Binary at target/release/wm-mcp-server
```

**Option C: download pre-built binary**

Download from [Releases](https://github.com/cpoder/wm-mcp-server/releases) and place in your PATH.

### 2. Configure

Create `.mcp.json` in your project directory:

```json
{
  "mcpServers": {
    "webmethods-is": {
      "command": "wm-mcp-server",
      "env": {
        "WM_IS_URL": "http://your-is-host:5555",
        "WM_IS_USER": "Administrator",
        "WM_IS_PASSWORD": "manage"
      }
    }
  }
}
```

For **multiple instances**, use a config file instead (see [mcp-server-rs/README.md](mcp-server-rs/README.md#multiple-instances-config-file)):

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

### 3. Use

Ask your AI assistant to create a flow service:

> "Create a flow service in package MyDemo that takes a name as input, calls pub.string:concat to build a greeting, and returns it."

The assistant will use `package_create`, `folder_create`, `flow_service_create`, `put_node`, and `service_invoke` to build and test the service automatically.

## How it works

The server communicates with webMethods IS exclusively through its built-in HTTP admin services (`wm.server.ns`, `wm.art.dev`, `pub.art`):

```
AI Assistant ──MCP (stdio)──> wm-mcp-server ──HTTP/JSON──> webMethods IS (port 5555)
```

Flow service creation leverages the `wm.server.ns/putNode` API, which accepts the full flow tree as JSON -- signatures, steps, and mappings in a single call. The JSON structure mirrors the internal `FlowElement` / `Values` serialization used by the IS runtime (same format consumed by `FlowElement.create(Values)` in `wm-isclient.jar`).

A single `put_node` call can define a complete flow service including:
- Input/output signatures
- INVOKE steps with input/output pipeline mappings
- MAP, BRANCH, LOOP, SEQUENCE, EXIT steps
- Nested step hierarchies of arbitrary depth

## Example: JDBC adapter querying SQL Server

```
# 1. Create a JDBC connection
adapter_connection_create(
    connection_alias="myapp.db:sqlserver",
    package_name="MyApp",
    adapter_type="JDBCAdapter",
    connection_factory_type="com.wm.adapter.wmjdbc.connection.JDBCConnectionFactory",
    connection_settings='{"transactionType":"NO_TRANSACTION","driverType":"Default","datasourceClass":"com.microsoft.sqlserver.jdbc.SQLServerDataSource","serverName":"localhost","user":"sa","password":"...","databaseName":"mydb","portNumber":"1433","otherProperties":"encrypt=false"}'
)

# 2. Enable it
adapter_connection_enable("myapp.db:sqlserver")

# 3. Create a CustomSQL service
adapter_service_create(
    service_name="myapp.services:getCustomers",
    package_name="MyApp",
    connection_alias="myapp.db:sqlserver",
    service_template="com.wm.adapter.wmjdbc.services.CustomSQL",
    adapter_service_settings='{"sql":"SELECT id, name, email FROM customers WHERE name = ?",…}'
)

# 4. Query it
service_invoke("myapp.services:getCustomers", '{"getCustomersInput":{"name":"Alice"}}')
# {"getCustomersOutput":{"results":[{"id":"1","name":"Alice","email":"alice@example.com"}]}}
```

## Documentation

See [mcp-server-rs/README.md](mcp-server-rs/README.md) for full documentation including:
- All 39 tools with parameters
- Complete `put_node` JSON format reference
- Flow step types (INVOKE, MAP, BRANCH, LOOP, etc.)
- WmPath format for field references
- Adapter connection settings for JDBC, SAP, OPC
- Environment variable reference

## Requirements

- webMethods Integration Server 11.x with HTTP access
- IS admin credentials

To build from source: Rust toolchain (`cargo`). The compiled binary has no runtime dependencies.

## License

MIT
