# webMethods Integration Server MCP Server

A Model Context Protocol (MCP) server that enables AI assistants to manage webMethods Integration Server instances through **pure HTTP APIs**. No filesystem access to the IS installation is required -- it works with remote IS instances.

## Architecture

```
Claude Code / AI Assistant
        |
        | MCP Protocol (stdio)
        v
  MCP Server (Python)
        |
        | HTTP/JSON (Basic Auth)
        v
  webMethods Integration Server
    (any host, port 5555)
```

All operations use IS built-in HTTP services (`wm.server.*`, `pub.art.*`, `wm.art.dev.*`). Flow service creation leverages `wm.server.ns/putNode`, which accepts the full flow tree as JSON -- the same `FlowElement` / `Values` serialization format used internally by the IS runtime.

## Setup

### Prerequisites

- Python 3.10+
- A running webMethods Integration Server (11.x) with HTTP access
- IS admin credentials

### Installation

```bash
git clone https://github.com/cpoder/wm-mcp-server.git
cd wm-mcp-server
python3 -m venv venv
source venv/bin/activate
pip install mcp httpx
```

### Configuration

Add to your Claude Code project's `.mcp.json`:

```json
{
  "mcpServers": {
    "webmethods-is": {
      "command": "/path/to/venv/bin/python",
      "args": ["/path/to/mcp-server/server.py"],
      "env": {
        "PYTHONPATH": "/path/to/mcp-server",
        "WM_IS_URL": "http://your-is-host:5555",
        "WM_IS_USER": "Administrator",
        "WM_IS_PASSWORD": "your-password"
      }
    }
  }
}
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `WM_IS_URL` | `http://localhost:5555` | IS base URL |
| `WM_IS_USER` | `Administrator` | IS username |
| `WM_IS_PASSWORD` | `manage` | IS password |
| `WM_IS_TIMEOUT` | `30` | HTTP request timeout (seconds) |

---

## Tools Reference (39 tools)

### Server Management (2)

| Tool | Description |
|------|-------------|
| `is_status` | Check if the IS is running and get server info |
| `is_shutdown(bounce)` | Shutdown or restart the IS. Set `bounce=true` to restart. **Note:** Starting the IS requires OS-level tools (SSH, systemd, startup script). |

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
| `node_get(name)` | Get full definition of any node (service, document, etc.) |
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
| `port_list` | List all ports (HTTP, HTTPS, FTP, FilePolling, Email, WebSocket) |
| `port_factory_list` | List available listener factory types |
| `port_get(port_key, pkg)` | Get detailed config for a specific port |
| `port_add(settings)` | Add a new port (HTTP, FTP, FilePolling, etc.) |
| `port_update(port_key, pkg, settings)` | Update port configuration |
| `port_enable(port_key, pkg)` | Enable a port |
| `port_disable(port_key, pkg)` | Disable a port |
| `port_delete(port_key, pkg)` | Delete a port |

### Adapter Management (15)

| Tool | Description |
|------|-------------|
| `adapter_type_list` | List registered adapter types (JDBC, SAP, OPC, etc.) |
| `adapter_connection_metadata(adapter_type, factory)` | Get available settings for a connection type |
| `adapter_connection_list` | List all adapter connections |
| `adapter_connection_create(...)` | Create a JDBC, SAP, or OPC connection |
| `adapter_connection_enable(alias)` | Enable a connection |
| `adapter_connection_disable(alias)` | Disable a connection |
| `adapter_connection_state(alias)` | Query connection state |
| `adapter_service_create(...)` | Create an adapter service (Select, Insert, CustomSQL, etc.) |
| `adapter_listener_list(adapter_type)` | List adapter listeners for a type |
| `adapter_listener_create(...)` | Create a listener (SAP RFC, OPC subscription, etc.) |
| `adapter_listener_enable(alias)` | Enable a listener |
| `adapter_listener_disable(alias)` | Disable a listener |
| `adapter_notification_list(adapter_type)` | List polling notifications for a type |
| `adapter_notification_create_polling(...)` | Create a polling notification |
| `adapter_notification_create_listener_based(...)` | Create a listener-based notification |

---

## Examples

### Example 1: Create a Hello World flow service

```
# 1. Create package and folder
package_create("MyDemo")
folder_create("MyDemo", "mydemo")
folder_create("MyDemo", "mydemo.services")

# 2. Create empty service shell
flow_service_create("MyDemo", "mydemo.services:helloWorld")

# 3. Define logic and signature via put_node
put_node({
  "node_nsName": "mydemo.services:helloWorld",
  "node_pkg": "MyDemo",
  "node_type": "service",
  "svc_type": "flow",
  "svc_subtype": "default",
  "svc_sigtype": "java 3.5",
  "stateless": "yes",
  "pipeline_option": 1,
  "svc_sig": {
    "sig_in": {
      "node_type": "record", "field_type": "record", "field_dim": "0", "nillable": "true",
      "rec_fields": [
        {"node_type": "field", "field_name": "name", "field_type": "string", "field_dim": "0", "nillable": "true", "field_opt": "true"}
      ]
    },
    "sig_out": {
      "node_type": "record", "field_type": "record", "field_dim": "0", "nillable": "true",
      "rec_fields": [
        {"node_type": "field", "field_name": "greeting", "field_type": "string", "field_dim": "0", "nillable": "true"}
      ]
    }
  },
  "flow": {
    "type": "ROOT", "version": "3.0", "cleanup": "true",
    "nodes": [
      {
        "type": "MAP", "mode": "STANDALONE",
        "nodes": [
          {"type": "MAPSET", "field": "/name;1;0", "overwrite": "false",
           "d_enc": "XMLValues", "mapseti18n": "true",
           "data": "<Values version=\"2.0\"><value name=\"xml\">World</value></Values>"}
        ]
      },
      {
        "type": "INVOKE", "service": "pub.string:concat",
        "validate-in": "$none", "validate-out": "$none",
        "nodes": [
          {"type": "MAP", "mode": "INPUT", "nodes": [
            {"type": "MAPSET", "field": "/inString1;1;0", "overwrite": "true",
             "d_enc": "XMLValues", "mapseti18n": "true",
             "data": "<Values version=\"2.0\"><value name=\"xml\">Hello, </value></Values>"},
            {"type": "MAPCOPY", "from": "/name;1;0", "to": "/inString2;1;0"}
          ]},
          {"type": "MAP", "mode": "OUTPUT", "nodes": [
            {"type": "MAPCOPY", "from": "/value;1;0", "to": "/greeting;1;0"}
          ]}
        ]
      }
    ]
  }
})

# 4. Test it
service_invoke("mydemo.services:helloWorld", '{"name": "Claude"}')
# Returns: {"greeting": "Hello, Claude"}
```

### Example 2: Math calculator with BRANCH

```
put_node({
  "node_nsName": "mydemo.services:calc",
  "node_pkg": "MyDemo",
  "node_type": "service", "svc_type": "flow", "svc_subtype": "default", "svc_sigtype": "java 3.5",
  "stateless": "yes", "pipeline_option": 1,
  "svc_sig": {
    "sig_in": {"node_type":"record","field_type":"record","field_dim":"0","nillable":"true",
      "rec_fields": [
        {"node_type":"field","field_name":"num1","field_type":"string","field_dim":"0","nillable":"true"},
        {"node_type":"field","field_name":"num2","field_type":"string","field_dim":"0","nillable":"true"},
        {"node_type":"field","field_name":"op","field_type":"string","field_dim":"0","nillable":"true"}
      ]},
    "sig_out": {"node_type":"record","field_type":"record","field_dim":"0","nillable":"true",
      "rec_fields": [
        {"node_type":"field","field_name":"result","field_type":"string","field_dim":"0","nillable":"true"}
      ]}
  },
  "flow": {"type":"ROOT","version":"3.0","cleanup":"true",
    "nodes":[
      {"type":"BRANCH","switch":"/op","nodes":[
        {"type":"SEQUENCE","label":"add","nodes":[
          {"type":"INVOKE","service":"pub.math:addFloats","validate-in":"$none","validate-out":"$none",
           "nodes":[
             {"type":"MAP","mode":"INPUT","nodes":[
               {"type":"MAPCOPY","from":"/num1;1;0","to":"/num1;1;0"},
               {"type":"MAPCOPY","from":"/num2;1;0","to":"/num2;1;0"}]},
             {"type":"MAP","mode":"OUTPUT","nodes":[
               {"type":"MAPCOPY","from":"/value;1;0","to":"/result;1;0"}]}
           ]}]},
        {"type":"SEQUENCE","label":"multiply","nodes":[
          {"type":"INVOKE","service":"pub.math:multiplyFloats","validate-in":"$none","validate-out":"$none",
           "nodes":[
             {"type":"MAP","mode":"INPUT","nodes":[
               {"type":"MAPCOPY","from":"/num1;1;0","to":"/num1;1;0"},
               {"type":"MAPCOPY","from":"/num2;1;0","to":"/num2;1;0"}]},
             {"type":"MAP","mode":"OUTPUT","nodes":[
               {"type":"MAPCOPY","from":"/value;1;0","to":"/result;1;0"}]}
           ]}]}
      ]}
    ]}
})

service_invoke("mydemo.services:calc", '{"num1":"6","num2":"7","op":"multiply"}')
# Returns: {"result": "42.0"}
```

### Example 3: JDBC adapter connection to SQL Server

```
# 1. Create connection (use adapter_connection_metadata to discover parameters)
adapter_connection_create(
  connection_alias="mydemo.connections:sqlserver",
  package_name="MyDemo",
  adapter_type="JDBCAdapter",
  connection_factory_type="com.wm.adapter.wmjdbc.connection.JDBCConnectionFactory",
  connection_settings='{"transactionType":"NO_TRANSACTION","driverType":"Default","datasourceClass":"com.microsoft.sqlserver.jdbc.SQLServerDataSource","serverName":"localhost","user":"sa","password":"YourPass!","databaseName":"master","portNumber":"1433","otherProperties":"encrypt=false"}'
)

# 2. Enable it
adapter_connection_enable("mydemo.connections:sqlserver")

# 3. Create a CustomSQL adapter service
adapter_service_create(
  service_name="mydemo.services:queryOrders",
  package_name="MyDemo",
  connection_alias="mydemo.connections:sqlserver",
  service_template="com.wm.adapter.wmjdbc.services.CustomSQL",
  adapter_service_settings='{"sql":"SELECT id, customer_name, product FROM orders WHERE customer_name = ?","inputExpression":["customer_name"],"inputJDBCType":["VARCHAR"],"inputFieldType":["java.lang.String"],"inputField":["customer_name"],"outputExpression":["id","customer_name","product"],"outputJDBCType":["INTEGER","VARCHAR","VARCHAR"],"outputFieldType":["java.lang.String","java.lang.String","java.lang.String"],"resultFieldType":["java.lang.String","java.lang.String","java.lang.String"],"outputField":["id","customer_name","product"],"resultField":["id","customer_name","product"],"realOutputField":["id","customer_name","product"]}'
)

# 4. Test it
service_invoke("mydemo.services:queryOrders", '{"queryOrdersInput":{"customer_name":"Claude"}}')
# Returns: {"queryOrdersOutput":{"results":[{"id":"1","customer_name":"Claude","product":"Widget"}]}}
```

### Example 4: File Polling port

```
port_add('{"factoryKey":"webMethods/FilePolling","pkg":"MyDemo","portAlias":"orderFiles","monitorDir":"/data/orders/incoming","processingService":"mydemo.services:processFile","filePollingInterval":"10","runUser":"Administrator","maxThreads":"3","enabled":"false"}')

port_enable("FilePollingListener:/data/orders/incoming", "MyDemo")
```

### Example 5: JDBC polling notification

```
adapter_notification_create_polling(
  notification_name="mydemo.notifications:onNewOrder",
  package_name="MyDemo",
  connection_alias="mydemo.connections:sqlserver",
  notification_template="com.wm.adapter.wmjdbc.notifications.InsertNotification"
)
```

---

## Core Concept: `put_node`

The `put_node` tool wraps `wm.server.ns/putNode`, which creates or updates a flow service with its full signature AND flow logic in a single HTTP call.

> **CRITICAL:** Always include the `flow` field. If omitted, the existing flow is wiped and the service becomes non-functional.

### Flow Step Types

| Type | JSON Keys | Description |
|------|-----------|-------------|
| `INVOKE` | `service`, `validate-in`, `validate-out` | Call another service |
| `MAP` | `mode` (STANDALONE/INPUT/OUTPUT) | Pipeline variable manipulation |
| `MAPCOPY` | `from`, `to` | Copy a value |
| `MAPSET` | `field`, `overwrite`, `d_enc`, `data` | Set a constant value |
| `MAPDELETE` | `field` | Delete a variable |
| `BRANCH` | `switch` | Conditional execution |
| `LOOP` | `in-array`, `out-array` | Iterate over arrays |
| `SEQUENCE` | `label`, `exit-on` | Group steps / branch cases |
| `EXIT` | `from`, `signal` | Exit flow/loop/sequence |

### WmPath Format

Field references use: `/fieldName;type;dim`

- **type:** 1=String, 2=Record, 3=Object, 4=RecordRef
- **dim:** 0=scalar, 1=array, 2=table

Examples: `/name;1;0` (string), `/items;1;1` (string array), `/order;2;0` (record)

### MAPSET Data Encoding

Values must be XMLValues-encoded:
```
<Values version="2.0"><value name="xml">the value</value></Values>
```

Use the `mapset_value` helper tool to encode values with proper XML escaping.

---

## Adapter Connection Settings

Use `adapter_connection_metadata(adapter_type, factory)` to discover the exact parameters for each connection type.

### JDBC (tested with SQL Server)
```json
{
  "transactionType": "NO_TRANSACTION",
  "driverType": "Default",
  "datasourceClass": "com.microsoft.sqlserver.jdbc.SQLServerDataSource",
  "serverName": "localhost",
  "user": "sa",
  "password": "...",
  "databaseName": "master",
  "portNumber": "1433",
  "otherProperties": "encrypt=false"
}
```

### SAP
```json
{
  "user": "DEVELOPER", "password": "...", "client": "001",
  "language": "en", "appServer": "host", "systemNumber": "00"
}
```

### OPC UA
```json
{
  "opcuaServerUri": "opc.tcp://host:port/path",
  "securityMode": "None", "authenticationMode": "Anonymous"
}
```

## Adapter Service Templates

### JDBC
| Template Class | Description |
|---|---|
| `com.wm.adapter.wmjdbc.services.Select` | SELECT query |
| `com.wm.adapter.wmjdbc.services.Insert` | INSERT statement |
| `com.wm.adapter.wmjdbc.services.Update` | UPDATE statement |
| `com.wm.adapter.wmjdbc.services.Delete` | DELETE statement |
| `com.wm.adapter.wmjdbc.services.CustomSQL` | Custom SQL (recommended) |
| `com.wm.adapter.wmjdbc.services.StoredProcedure` | Stored procedure call |
| `com.wm.adapter.wmjdbc.services.DynamicSQL` | Dynamic SQL |

### JDBC Notification Templates
| Template Class | Description |
|---|---|
| `com.wm.adapter.wmjdbc.notifications.InsertNotification` | Detect new rows |
| `com.wm.adapter.wmjdbc.notifications.UpdateNotification` | Detect updated rows |
| `com.wm.adapter.wmjdbc.notifications.DeleteNotification` | Detect deleted rows |

---

## IS APIs Used

| API Endpoint | Purpose |
|---|---|
| `wm.server.ns/putNode` | Create/update services with full flow logic (core API) |
| `wm.server.ns/getNode` | Read node definitions |
| `wm.server.ns/getNodeList` | List nodes in a package/folder |
| `wm.server.ns/makeNode` | Create folders and empty document types |
| `wm.server.ns/deleteNode` | Delete nodes |
| `wm.server.ns/lockNode` | Lock nodes for editing |
| `wm.server.ns/unLockNode` | Unlock nodes |
| `wm.server.services/serviceAdd` | Create empty service shells |
| `wm.server.packages/*` | Package lifecycle |
| `wm.server.admin/getServerStatus` | Server status |
| `wm.server.admin/shutdown` | Server shutdown/restart |
| `wm.server.net.listeners/*` | Port management |
| `wm.art.admin:retrieveAdapterTypesList` | List registered adapter types |
| `wm.art.dev.connection:createConnectionNode` | Create adapter connections |
| `wm.art.dev.connection:fetchConnectionMetadata` | Discover connection parameters |
| `wm.art.admin.connection:listAllResources` | List all connections |
| `pub.art.connection:enableConnection` | Enable/disable/query connections |
| `wm.art.dev.service:createAdapterServiceNode` | Create adapter services |
| `wm.art.dev.service:updateAdapterServiceNode` | Configure adapter services |
| `wm.art.dev.listener:createListenerNode` | Create adapter listeners |
| `pub.art.listener:listAdapterListeners` | List listeners by type |
| `wm.art.dev.notification:create*` | Create notifications |
| `pub.art.notification:listAdapterPollingNotifications` | List notifications by type |

## Limitations

- **Server start:** Cannot start the IS via HTTP. Use `is_shutdown(bounce=true)` for restarts, or OS-level tools for cold starts.
- **Java services:** Only flow services can be fully created via API. Java services require IS-level compilation.
- **Adapter service configuration:** Complex adapter services (e.g., JDBC Select) require many metadata parameters. Use `CustomSQL` for simpler setup, or `adapter_connection_metadata` to discover parameters.

## Technical Notes

The `put_node` JSON format mirrors the `FlowElement` / `Values` serialization used internally by the IS runtime (`wm-isclient.jar`). Key classes: `FlowElement`, `FlowRoot`, `FlowInvoke`, `FlowBranch`, `FlowLoop`, `FlowMap`, `FlowMapSet`, `FlowMapCopy`, `NSSignature`, `NSRecord`, `NSField`.

`FlowElement.setValues(Values)` reads the `nodes` key as an array of Values objects and recursively instantiates child elements via its factory. MAP nodes with `mode=INPUT`/`OUTPUT` inside a non-ROOT parent are automatically assigned as that parent's input/output maps.
