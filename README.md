# webMethods Integration Server MCP Server

[![CI](https://github.com/cpoder/wm-mcp-server/actions/workflows/ci.yml/badge.svg)](https://github.com/cpoder/wm-mcp-server/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/wm-mcp-server.svg)](https://crates.io/crates/wm-mcp-server)

An [MCP (Model Context Protocol)](https://modelcontextprotocol.io/) server that gives AI assistants full control over [webMethods Integration Server](https://www.ibm.com/docs/en/webmethods-integration/wm-integration-server/11.1.0) -- replacing webMethods Designer for most development and administration tasks.

Single ~4MB binary, no runtime dependencies, works with any remote IS instance via pure HTTP APIs.

Compatible with any MCP client: [IBM Bob](https://www.ibm.com/products/bob), [Claude Code](https://claude.ai/claude-code), Claude Desktop, Cursor, Windsurf, etc.

## What can it do?

| Area | What the AI can do | Tools |
|---|---|---|
| **Flow services** | Create services with full logic (INVOKE, MAP, BRANCH, LOOP, TRY/CATCH), signatures, test them | 5 |
| **Flow debugging** | Step-by-step execution, inspect pipeline at each step, set breakpoints, modify variables | 7 |
| **Unit testing** | Run test suites, get JUnit/text reports, mock services for isolated testing | 10 |
| **Namespace dependencies** | Find dependents, references, unresolved refs, search nodes, refactor/rename | 6 |
| **Adapter services** | Browse database tables/columns interactively, create Select/Insert/CustomSQL services -- like Designer | 5 |
| **JDBC/SAP/OPC adapters** | Create connections, listeners, notifications, query metadata | 20 |
| **Document type generation** | Generate IS doc types from JSON, JSON Schema, XSD, XML, DTD samples, SAP IDoc/RFC | 7 |
| **Flat file schemas** | Create/read/delete flat file schemas and dictionaries | 4 |
| **Kafka streaming** | Create Kafka connections, event specifications, triggers | 15 |
| **JMS messaging** | Create JMS connections and triggers via JNDI providers | 20 |
| **MQTT messaging** | Create MQTT connections and triggers | 12 |
| **Pub/Sub triggers** | Manage messaging triggers, inspect status, get stats | 9 |
| **Messaging** | Connection management, CSQ, publishable doc types, publish/deliver messages | 11 |
| **Package marketplace** | Browse, search, and install packages from [packages.webmethods.io](https://packages.webmethods.io) | 7 |
| **Scheduler** | Create/manage scheduled tasks (one-time, repeating, complex) | 10 |
| **Users & access** | Create users/groups, manage ACLs, assign ACLs to nodes, account locking, default access | 21 |
| **Server admin** | Health, stats, thread dumps, logs, thread/session kill, SSL cache, license info | 15 |
| **Packages & namespace** | Full CRUD, settings, compile, dependencies, startup services, JAR management | 21 |
| **Ports & port access** | HTTP/FTP/FilePolling/WebSocket listener management + per-port access control | 14 |
| **URL aliases** | Clean REST URL routing with CRUD | 5 |
| **JDBC pools** | Connection pool CRUD, driver management | 8 |
| **Global variables** | Configuration variable management | 5 |
| **Remote servers** | IS-to-IS remote server aliases | 4 |
| **Web services** | REST/SOAP endpoint CRUD, OpenAPI generation, connector refresh | 13 |
| **Auditing** | Audit logger management | 5 |
| **OAuth** | Client registration, scopes, token management, settings | 9 |
| **JWT** | Issuer management, global JWT settings | 6 |
| **SFTP** | Server and user alias management, connection testing | 9 |
| **HTTP proxy** | Proxy server alias CRUD | 6 |
| **Cache manager** | Cache CRUD, reset | 6 |
| **Logger config** | View/change log levels for all loggers, server logging config | 5 |
| **SAML** | SAML issuer management | 3 |
| **LDAP** | LDAP server configuration | 4 |
| **Outbound passwords** | Stored password management for adapter connections | 3 |
| **Enterprise gateway** | Threat protection rules and DoS settings | 7 |
| **Security** | Keystores, truststores, security settings, password policy | 6 |
| **Quiesce mode** | Graceful server drain | 3 |
| **Health indicators** | Server health indicator management | 3 |
| **Alerts** | Alert notifier enable/disable/status | 3 |
| **IP access** | Global IP allow/deny rules | 4 |
| **WebSocket** | Session management, endpoint creation, broadcast | 4 |

**336 tools + 9 interactive prompts + 5 RAG resources** in total. **184 end-to-end tests** validated against a live IS with real infrastructure (Mosquitto MQTT broker, ActiveMQ JMS, MySQL).

### Interactive setup wizards (prompts)

The AI can guide you step-by-step through setting up:

| Prompt | What it does |
|---|---|
| `setup_jdbc_connection` | Walks through JDBC connection + adapter service creation, browses tables/columns interactively |
| `setup_kafka_streaming` | Sets up Kafka connection alias + event specification |
| `setup_jms_connection` | Creates JNDI provider + JMS connection (guides JAR installation) |
| `setup_mqtt_connection` | Sets up MQTT connection + trigger subscription |
| `setup_sap_connection` | Configures SAP adapter connection |
| `setup_scheduled_task` | Schedules a service for execution |
| `setup_rest_api` | Exposes services via OpenAPI or imports an API spec |
| `setup_user_management` | Creates users, groups, ACLs |
| `setup_oauth` | Registers OAuth clients and scopes |

## Quick Start

### 1. Install

```bash
# From crates.io
cargo install wm-mcp-server

# Or download a pre-built binary from Releases
# https://github.com/cpoder/wm-mcp-server/releases
```

Binaries available for: Linux (x86_64, aarch64), macOS (x86_64, Apple Silicon), Windows (x86_64).

### 2. Configure your MCP client

Add to `.mcp.json` in your project directory:

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

#### Multiple IS instances

Create a config file (e.g., `wm-instances.json`):

```json
{
  "instances": {
    "dev": { "url": "http://dev-host:5555", "user": "Administrator", "password": "manage" },
    "prod": { "url": "http://prod-host:5555", "user": "Administrator", "password": "secret", "timeout": 60 }
  },
  "default": "dev"
}
```

```json
{
  "mcpServers": {
    "webmethods-is": {
      "command": "wm-mcp-server",
      "env": { "WM_CONFIG": "/path/to/wm-instances.json" }
    }
  }
}
```

Every tool accepts an optional `instance` parameter. Use `list_instances` to see what's configured.

### 3. Use

**Natural language examples:**

> "Create a flow service that greets the user by name"

The AI creates the package, folders, service with full flow logic, and tests it.

> "Set up a JDBC connection to my SQL Server and create a Select service for the orders table"

The AI uses the `setup_jdbc_connection` prompt to walk you through credentials, then browses your database tables/columns interactively (like Designer) to build the adapter service.

> "Debug the helloWorld service step by step"

The AI starts a debug session, steps through each flow step, shows you the pipeline state at each breakpoint.

> "Install JcPublicTools from the package registry"

The AI searches packages.webmethods.io, downloads the package from GitHub, installs it into IS, and activates it.

> "Run the test suite for MyPackage and show me the results"

The AI executes the unit tests, waits for completion, and retrieves the JUnit report.

> "Create an MQTT connection to my Mosquitto broker and subscribe to sensor/temperature"

The AI creates the connection, enables it, verifies connectivity, and sets up a trigger.

## How it works

```
AI Assistant ──MCP (stdio/HTTP)──> wm-mcp-server (Rust) ──HTTP/JSON──> webMethods IS (port 5555)
                                          │
                                          └──HTTPS──> packages.webmethods.io (marketplace)
```

All operations use IS built-in HTTP services. No filesystem access needed for most operations (marketplace install requires local access to the IS packages directory).

Flow services are created via `wm.server.ns/putNode` which accepts the full flow tree as JSON -- the same `FlowElement` / `Values` serialization used internally by the IS runtime. The server includes 5 embedded RAG resources with comprehensive flow language documentation, 15 working putNode examples (including TRY/CATCH, LOOP, BRANCH, MAPINVOKE, transactions), and adapter service configuration guides.

### Transport modes

- **stdio** (default): Standard MCP stdio transport, works with all MCP clients
- **HTTP**: `wm-mcp-server --http 8080` starts a Streamable HTTP server at `/mcp` for MCP gateways

### Tool scoping

Set `WM_SCOPES` to restrict which tools are exposed (useful for MCP gateways):

```bash
WM_SCOPES=develop,monitor  # Only development and monitoring tools
WM_SCOPES=readonly          # Only read-only tools (list, get, status)
```

Available scopes: `admin`, `develop`, `deploy`, `adapters`, `messaging`, `monitor`, `network`, `readonly`.

## Requirements

- webMethods Integration Server 11.x with HTTP access enabled
- IS admin credentials (Administrator or equivalent)
- For JMS: provider client JARs in `WmART/code/jars/static/` + IS restart
- For Kafka streaming: Kafka client JARs in `WmStreaming/code/jars/static/`
- For marketplace install: MCP server needs filesystem access to IS packages directory

## License

MIT
