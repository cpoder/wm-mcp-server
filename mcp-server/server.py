#!/usr/bin/env python3
"""
webMethods Integration Server MCP Server

Pure HTTP API-based server for managing webMethods IS: create packages,
flow services with full logic, document types, adapter connections, and more.
No disk access required - works with remote IS instances.

Key discovery: wm.server.ns/putNode accepts the full flow tree as JSON,
including nested flow steps (INVOKE, MAP, BRANCH, LOOP, etc.) via the
"nodes" array field. MAPSET data uses XMLValues encoding.
"""

import json
import os
import textwrap
from mcp.server.fastmcp import FastMCP
from is_client import ISClient

mcp = FastMCP(
    "webMethods Integration Server",
    instructions=textwrap.dedent("""\
        MCP server for managing webMethods Integration Server via pure HTTP API.

        KEY CONCEPTS:
        - Packages contain services, document types, and adapter configurations
        - Services are identified by "folder.subfolder:serviceName" paths
        - Flow services have steps: INVOKE, MAP, BRANCH, LOOP, SEQUENCE, EXIT
        - The putNode API is the core for creating/updating services with full flow logic
        - Adapter connections link IS to external systems (SAP, JDBC, OPC)

        FLOW STEP TYPES AND THEIR JSON KEYS:
        - INVOKE: {type:"INVOKE", service:"pub.string:concat", validate-in:"$none", validate-out:"$none", nodes:[input_map, output_map]}
        - MAP (standalone): {type:"MAP", mode:"STANDALONE", nodes:[MAPSET/MAPCOPY/MAPDELETE nodes]}
        - MAP (input): {type:"MAP", mode:"INPUT", nodes:[...]} -- goes inside INVOKE's nodes array
        - MAP (output): {type:"MAP", mode:"OUTPUT", nodes:[...]} -- goes inside INVOKE's nodes array
        - MAPCOPY: {type:"MAPCOPY", from:"/srcField;1;0", to:"/dstField;1;0"}
        - MAPSET: {type:"MAPSET", field:"/field;1;0", overwrite:"true", d_enc:"XMLValues", mapseti18n:"true", data:"<Values version=\\"2.0\\"><value name=\\"xml\\">theValue</value></Values>"}
        - MAPDELETE: {type:"MAPDELETE", field:"/field;1;0"}
        - BRANCH: {type:"BRANCH", switch:"/field", nodes:[SEQUENCE children with label names]}
        - LOOP: {type:"LOOP", in-array:"/arrayField", out-array:"/outArray", nodes:[child steps]}
        - SEQUENCE: {type:"SEQUENCE", label:"name", exit-on:"FAILURE", nodes:[child steps]}
        - EXIT: {type:"EXIT", from:"$flow", signal:"FAILURE"}

        WMPATH FORMAT for field references: /fieldName;type;dim
        - type: 1=String, 2=Record, 3=Object, 4=RecordRef
        - dim: 0=scalar, 1=array
        Example: /myString;1;0 (scalar string), /myList;1;1 (string array), /myDoc;2;0 (record)
    """),
)

client = ISClient(
    base_url=os.environ.get("WM_IS_URL", "http://localhost:5555"),
    username=os.environ.get("WM_IS_USER", "Administrator"),
    password=os.environ.get("WM_IS_PASSWORD", "manage"),
    timeout=float(os.environ.get("WM_IS_TIMEOUT", "30")),
)


# ═══════════════════════════════════════════════════════════════════════
# SERVER STATUS
# ═══════════════════════════════════════════════════════════════════════

@mcp.tool()
async def is_status() -> str:
    """Check if the Integration Server is running and responsive."""
    running = await client.is_running()
    if not running:
        return "Server is NOT running or not responding on the configured port."
    try:
        status = await client.get_server_status()
        return f"Server is RUNNING.\n{json.dumps(status, indent=2)}"
    except Exception:
        return "Server is RUNNING (responding to requests)."


@mcp.tool()
async def is_shutdown(bounce: bool = False) -> str:
    """Shutdown or restart the Integration Server via HTTP API.

    NOTE: Starting the server is not possible via HTTP (the server must already be
    running to accept requests). Use your OS process manager, SSH, or the IS
    startup script for that.

    Args:
        bounce: If True, restart the server instead of stopping it.
    """
    try:
        result = await client.shutdown(bounce=bounce)
        return json.dumps(result, indent=2)
    except Exception as e:
        return f"Shutdown failed: {e}"


# ═══════════════════════════════════════════════════════════════════════
# PACKAGE MANAGEMENT
# ═══════════════════════════════════════════════════════════════════════

@mcp.tool()
async def package_list() -> str:
    """List all packages on the Integration Server."""
    return json.dumps(await client.package_list(), indent=2)

@mcp.tool()
async def package_create(package_name: str) -> str:
    """Create and activate a new package.

    Args:
        package_name: Package name in PascalCase (e.g., "MyNewPackage")
    """
    return json.dumps(await client.package_create(package_name), indent=2)

@mcp.tool()
async def package_reload(package_name: str) -> str:
    """Reload a package to pick up changes.

    Args:
        package_name: Package name
    """
    return json.dumps(await client.package_reload(package_name), indent=2)

@mcp.tool()
async def package_enable(package_name: str) -> str:
    """Enable a package. Args: package_name"""
    return json.dumps(await client.package_enable(package_name), indent=2)

@mcp.tool()
async def package_disable(package_name: str) -> str:
    """Disable a package. Args: package_name"""
    return json.dumps(await client.package_disable(package_name), indent=2)


# ═══════════════════════════════════════════════════════════════════════
# NAMESPACE BROWSING
# ═══════════════════════════════════════════════════════════════════════

@mcp.tool()
async def node_list(package: str, folder: str = "") -> str:
    """List services, folders, and document types in a package or folder.

    Args:
        package: Package name
        folder: Optional folder path (e.g., "services.utils"). Empty = package root.
    """
    return json.dumps(await client.node_list(package, folder), indent=2)

@mcp.tool()
async def node_get(name: str) -> str:
    """Get the full definition of a node (service, document, connection).

    Returns signature, flow definition, fields, etc.

    Args:
        name: Full namespace path (e.g., "claudedemo.services:helloWorld")
    """
    return json.dumps(await client.node_get(name), indent=2)

@mcp.tool()
async def node_delete(name: str) -> str:
    """Delete a node (service, folder, document type).

    Args:
        name: Full namespace path
    """
    try:
        return json.dumps(await client.node_delete(name), indent=2)
    except Exception as e:
        return f"Delete failed: {e}"


# ═══════════════════════════════════════════════════════════════════════
# FOLDER MANAGEMENT
# ═══════════════════════════════════════════════════════════════════════

@mcp.tool()
async def folder_create(package: str, folder_path: str) -> str:
    """Create a folder (namespace) in a package. Create parent folders first for nested paths.

    Args:
        package: Package name
        folder_path: Dot-separated path (e.g., "services" or "services.utils")
    """
    return json.dumps(await client.folder_create(package, folder_path), indent=2)


# ═══════════════════════════════════════════════════════════════════════
# FLOW SERVICE MANAGEMENT (via putNode - the core API)
# ═══════════════════════════════════════════════════════════════════════

@mcp.tool()
async def flow_service_create(package: str, service_path: str) -> str:
    """Create an empty flow service. Use put_node to add logic and signature.

    Args:
        package: Package name
        service_path: Path as "folder:serviceName" (e.g., "services:helloWorld")
    """
    return json.dumps(await client.service_create(package, service_path), indent=2)


@mcp.tool()
async def put_node(node_data: str) -> str:
    """Create or update a namespace node (flow service, document type, etc.) via the IS putNode API.

    This is THE core API for creating flow services with full logic, signatures, and flow steps.
    It also works for updating document types with field definitions.

    The node_data JSON must follow the IS Values serialization format.

    EXAMPLE - Complete flow service with signature and flow logic:
    {
      "node_nsName": "mypkg.services:greet",
      "node_pkg": "MyPackage",
      "node_type": "service",
      "svc_type": "flow",
      "svc_subtype": "default",
      "svc_sigtype": "java 3.5",
      "stateless": "yes",
      "pipeline_option": 1,
      "node_comment": "A greeting service",
      "svc_sig": {
        "sig_in": {
          "node_type": "record", "field_type": "record", "field_dim": "0", "nillable": "true",
          "rec_fields": [
            {"node_type": "field", "field_name": "name", "field_type": "string", "field_dim": "0", "nillable": "true"}
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
        "comment": "Builds a greeting",
        "nodes": [
          {
            "type": "MAP", "mode": "STANDALONE",
            "nodes": [
              {"type": "MAPSET", "field": "/name;1;0", "overwrite": "false",
               "d_enc": "XMLValues", "mapseti18n": "true",
               "data": "<Values version=\\"2.0\\"><value name=\\"xml\\">World</value></Values>"}
            ]
          },
          {
            "type": "INVOKE", "service": "pub.string:concat",
            "validate-in": "$none", "validate-out": "$none",
            "nodes": [
              {"type": "MAP", "mode": "INPUT", "nodes": [
                {"type": "MAPSET", "field": "/inString1;1;0", "overwrite": "true",
                 "d_enc": "XMLValues", "mapseti18n": "true",
                 "data": "<Values version=\\"2.0\\"><value name=\\"xml\\">Hello, </value></Values>"},
                {"type": "MAPCOPY", "from": "/name;1;0", "to": "/inString2;1;0"}
              ]},
              {"type": "MAP", "mode": "OUTPUT", "nodes": [
                {"type": "MAPCOPY", "from": "/value;1;0", "to": "/greeting;1;0"}
              ]}
            ]
          }
        ]
      }
    }

    Args:
        node_data: JSON string with the full node definition
    """
    try:
        data = json.loads(node_data) if isinstance(node_data, str) else node_data
    except json.JSONDecodeError as e:
        return f"Invalid JSON: {e}"

    try:
        result = await client.put_node(data)
        return json.dumps(result, indent=2)
    except Exception as e:
        return f"putNode failed: {e}"


# ═══════════════════════════════════════════════════════════════════════
# DOCUMENT TYPE MANAGEMENT
# ═══════════════════════════════════════════════════════════════════════

@mcp.tool()
async def document_type_create(package: str, doc_path: str) -> str:
    """Create a document type. Create parent folders first if needed.

    Args:
        package: Package name
        doc_path: Document path as "folder.docTypes:docName"
    """
    return json.dumps(await client.document_type_create(package, doc_path), indent=2)


# ═══════════════════════════════════════════════════════════════════════
# SERVICE INVOCATION / TESTING
# ═══════════════════════════════════════════════════════════════════════

@mcp.tool()
async def service_invoke(service_path: str, inputs: str = "{}") -> str:
    """Invoke (execute/test) a service.

    Args:
        service_path: Service path (e.g., "claudedemo.services:helloWorld")
        inputs: JSON string of input parameters
    """
    try:
        input_dict = json.loads(inputs) if inputs else {}
    except json.JSONDecodeError as e:
        return f"Invalid JSON input: {e}"
    try:
        result = await client.service_invoke(service_path, input_dict if input_dict else None)
        return json.dumps(result, indent=2)
    except Exception as e:
        return f"Service invocation failed: {e}"


# ═══════════════════════════════════════════════════════════════════════
# PORT / LISTENER MANAGEMENT (HTTP, FTP, FilePolling, etc.)
# ═══════════════════════════════════════════════════════════════════════

@mcp.tool()
async def port_list() -> str:
    """List all ports/listeners (HTTP, HTTPS, FTP, FTPS, FilePolling, Email, WebSocket).

    Returns detailed configuration for each port including protocol, status, and settings.
    """
    return json.dumps(await client.port_list(), indent=2)

@mcp.tool()
async def port_factory_list() -> str:
    """List available listener factory types that can be used to create new ports.

    Common factories: webMethods/HTTP, webMethods/FTP, webMethods/FTPS,
    webMethods/FilePolling, webMethods/Email, webMethods/WebSocket.
    """
    return json.dumps(await client.port_factory_list(), indent=2)

@mcp.tool()
async def port_get(port_key: str, pkg: str) -> str:
    """Get detailed configuration of a specific port/listener.

    Args:
        port_key: Listener key from port_list (e.g., "HTTPListener@5555")
        pkg: Package that owns the listener (e.g., "WmRoot")
    """
    try:
        return json.dumps(await client.port_get(port_key, pkg), indent=2)
    except Exception as e:
        return f"Failed: {e}"

@mcp.tool()
async def port_add(settings: str) -> str:
    """Add a new port/listener (HTTP, FTP, FilePolling, etc.).

    The settings JSON must include "factoryKey" and "pkg". Use port_factory_list to see types.

    Examples:
    - HTTP: {"factoryKey":"webMethods/HTTP","pkg":"WmRoot","port":"5556","portAlias":"myHttp","enabled":"false"}
    - FilePolling: {"factoryKey":"webMethods/FilePolling","pkg":"MyPkg","portAlias":"myPoll","monitorDir":"/path","processingService":"pkg.folder:svc","filePollingInterval":"10","enabled":"false"}
    - FTP: {"factoryKey":"webMethods/FTP","pkg":"WmRoot","port":"8021","portAlias":"myFtp","enabled":"false"}

    Args:
        settings: JSON string with listener configuration
    """
    try:
        data = json.loads(settings)
    except json.JSONDecodeError as e:
        return f"Invalid JSON: {e}"
    try:
        return json.dumps(await client.port_add(data), indent=2)
    except Exception as e:
        return f"Failed: {e}"

@mcp.tool()
async def port_update(port_key: str, pkg: str, settings: str) -> str:
    """Update an existing port/listener configuration.

    Args:
        port_key: Listener key (e.g., "HTTPListener@5555")
        pkg: Package that owns the listener (e.g., "WmRoot")
        settings: JSON string with properties to update
    """
    try:
        data = json.loads(settings)
    except json.JSONDecodeError as e:
        return f"Invalid JSON: {e}"
    try:
        return json.dumps(await client.port_update(port_key, pkg, data), indent=2)
    except Exception as e:
        return f"Failed: {e}"

@mcp.tool()
async def port_enable(port_key: str, pkg: str) -> str:
    """Enable a port/listener.

    Args:
        port_key: Listener key (e.g., "HTTPListener@5556")
        pkg: Package (e.g., "WmRoot")
    """
    try:
        return json.dumps(await client.port_enable(port_key, pkg), indent=2)
    except Exception as e:
        return f"Failed: {e}"

@mcp.tool()
async def port_disable(port_key: str, pkg: str) -> str:
    """Disable a port/listener.

    Args:
        port_key: Listener key
        pkg: Package
    """
    try:
        return json.dumps(await client.port_disable(port_key, pkg), indent=2)
    except Exception as e:
        return f"Failed: {e}"

@mcp.tool()
async def port_delete(port_key: str, pkg: str) -> str:
    """Delete a port/listener.

    Args:
        port_key: Listener key
        pkg: Package
    """
    try:
        return json.dumps(await client.port_delete(port_key, pkg), indent=2)
    except Exception as e:
        return f"Failed: {e}"


# ═══════════════════════════════════════════════════════════════════════
# ADAPTER CONNECTION MANAGEMENT
# ═══════════════════════════════════════════════════════════════════════

@mcp.tool()
async def adapter_type_list() -> str:
    """List all registered adapter types (JDBC, SAP, OPC, MongoDB, etc.).

    Returns adapter names, versions, and vendors. Use the adapterName value
    as the adapter_type parameter in other adapter tools.
    """
    try:
        return json.dumps(await client.adapter_type_list(), indent=2)
    except Exception as e:
        return f"Failed: {e}"

@mcp.tool()
async def adapter_connection_metadata(adapter_type: str, connection_factory_type: str) -> str:
    """Get the metadata (available settings/parameters) for creating connections of a specific adapter type.

    Use this to discover what connectionSettings parameters are required.

    Args:
        adapter_type: Adapter type name (e.g., "JDBCAdapter", "WmSAP", "WmOPCAdapter")
        connection_factory_type: Factory class name
    """
    try:
        return json.dumps(await client.adapter_connection_metadata(adapter_type, connection_factory_type), indent=2)
    except Exception as e:
        return f"Failed: {e}"

@mcp.tool()
async def adapter_connection_list() -> str:
    """List all adapter connections."""
    return json.dumps(await client.adapter_connection_list(), indent=2)

@mcp.tool()
async def adapter_connection_create(
    connection_alias: str, package_name: str, adapter_type: str,
    connection_factory_type: str, connection_settings: str,
    pool_min: int = 1, pool_max: int = 10,
) -> str:
    """Create an adapter connection via WmART API.

    Args:
        connection_alias: Alias like "mypkg.connections:mydb"
        package_name: Package name
        adapter_type: "WmJDBCAdapter", "WmSAP", "WmOPCAdapter"
        connection_factory_type: Factory class name, e.g.:
            - "com.wm.adapter.wmjdbc.connection.JDBCConnectionFactory"
            - "com.wm.adapter.sap.spi.SAPConnectionFactory"
            - "com.wm.adapter.wmopcua.connection.WmOPCConnectionFactory"
        connection_settings: JSON string of connection properties
        pool_min: Min pool size
        pool_max: Max pool size
    """
    try:
        settings = json.loads(connection_settings)
    except json.JSONDecodeError as e:
        return f"Invalid JSON: {e}"
    mgr = {
        "poolable": "true", "minimumPoolSize": str(pool_min), "maximumPoolSize": str(pool_max),
        "poolIncrementSize": "1", "blockingTimeout": "1000", "expireTimeout": "1000",
        "startupRetryCount": "0", "startupBackoffSecs": "5",
    }
    try:
        return json.dumps(await client.adapter_connection_create(
            connection_alias, package_name, adapter_type,
            connection_factory_type, settings, mgr,
        ), indent=2)
    except Exception as e:
        return f"Failed: {e}"

@mcp.tool()
async def adapter_connection_enable(connection_alias: str) -> str:
    """Enable an adapter connection. Args: connection_alias (e.g., "demosap:connNode_sap")"""
    try:
        return json.dumps(await client.adapter_connection_enable(connection_alias), indent=2)
    except Exception as e:
        return f"Failed: {e}"

@mcp.tool()
async def adapter_connection_disable(connection_alias: str) -> str:
    """Disable an adapter connection. Args: connection_alias"""
    try:
        return json.dumps(await client.adapter_connection_disable(connection_alias), indent=2)
    except Exception as e:
        return f"Failed: {e}"

@mcp.tool()
async def adapter_connection_state(connection_alias: str) -> str:
    """Query state of an adapter connection. Args: connection_alias"""
    try:
        return json.dumps(await client.adapter_connection_state(connection_alias), indent=2)
    except Exception as e:
        return f"Failed: {e}"


# ═══════════════════════════════════════════════════════════════════════
# ADAPTER LISTENER MANAGEMENT
# ═══════════════════════════════════════════════════════════════════════

@mcp.tool()
async def adapter_listener_list(adapter_type: str) -> str:
    """List all adapter listeners for a specific adapter type.

    Args:
        adapter_type: Adapter type (e.g., "WmSAP", "WmOPCAdapter", "JDBCAdapter")
    """
    return json.dumps(await client.adapter_listener_list(adapter_type), indent=2)

@mcp.tool()
async def adapter_listener_create(
    listener_alias: str, package_name: str, adapter_type: str,
    connection_alias: str, listener_settings: str = "{}",
) -> str:
    """Create an adapter listener.

    Args:
        listener_alias: Alias like "mypkg.listeners:sapListener"
        package_name: Package name
        adapter_type: "WmSAP", "WmOPCAdapter", etc.
        connection_alias: Connection alias this listener uses
        listener_settings: JSON string of listener properties
    """
    try:
        settings = json.loads(listener_settings) if listener_settings else {}
    except json.JSONDecodeError as e:
        return f"Invalid JSON: {e}"
    try:
        return json.dumps(await client.adapter_listener_create(
            listener_alias, package_name, adapter_type, connection_alias,
            settings if settings else None,
        ), indent=2)
    except Exception as e:
        return f"Failed: {e}"

@mcp.tool()
async def adapter_listener_enable(listener_alias: str) -> str:
    """Enable an adapter listener. Args: listener_alias"""
    try:
        return json.dumps(await client.adapter_listener_enable(listener_alias), indent=2)
    except Exception as e:
        return f"Failed: {e}"

@mcp.tool()
async def adapter_listener_disable(listener_alias: str) -> str:
    """Disable an adapter listener. Args: listener_alias"""
    try:
        return json.dumps(await client.adapter_listener_disable(listener_alias), indent=2)
    except Exception as e:
        return f"Failed: {e}"


# ═══════════════════════════════════════════════════════════════════════
# ADAPTER NOTIFICATION MANAGEMENT
# ═══════════════════════════════════════════════════════════════════════

@mcp.tool()
async def adapter_service_create(
    service_name: str, package_name: str,
    connection_alias: str, service_template: str,
    adapter_service_settings: str = "{}",
) -> str:
    """Create an adapter service (JDBC Select, Insert, CustomSQL, etc.).

    Args:
        service_name: Full name like "mypkg.services:queryDb"
        package_name: Package name
        connection_alias: Connection to use (e.g., "mypkg.connections:sqlserver")
        service_template: Full template class name. Common JDBC templates:
            - "com.wm.adapter.wmjdbc.services.Select"
            - "com.wm.adapter.wmjdbc.services.Insert"
            - "com.wm.adapter.wmjdbc.services.Update"
            - "com.wm.adapter.wmjdbc.services.Delete"
            - "com.wm.adapter.wmjdbc.services.CustomSQL"
            - "com.wm.adapter.wmjdbc.services.StoredProcedure"
            - "com.wm.adapter.wmjdbc.services.DynamicSQL"
        adapter_service_settings: JSON string of service-specific settings
    """
    try:
        settings = json.loads(adapter_service_settings) if adapter_service_settings and adapter_service_settings != "{}" else None
    except json.JSONDecodeError as e:
        return f"Invalid JSON: {e}"
    try:
        return json.dumps(await client.adapter_service_create(
            service_name, package_name, connection_alias, service_template, settings,
        ), indent=2)
    except Exception as e:
        return f"Failed: {e}"


@mcp.tool()
async def adapter_notification_list(adapter_type: str) -> str:
    """List adapter polling notifications for a specific adapter type.

    Args:
        adapter_type: Adapter type (e.g., "JDBCAdapter", "WmSAP")
    """
    return json.dumps(await client.adapter_notification_list(adapter_type), indent=2)

@mcp.tool()
async def adapter_notification_create_polling(
    notification_name: str, package_name: str,
    connection_alias: str, notification_template: str,
    notification_settings: str = "{}",
) -> str:
    """Create a polling notification (JDBC insert/update/delete detection, etc.).

    Args:
        notification_name: Full name like "mypkg.notifications:onInsert"
        package_name: Package name
        connection_alias: Connection to use
        notification_template: Full template class. Common JDBC templates:
            - "com.wm.adapter.wmjdbc.notifications.InsertNotification"
            - "com.wm.adapter.wmjdbc.notifications.UpdateNotification"
            - "com.wm.adapter.wmjdbc.notifications.DeleteNotification"
            - "com.wm.adapter.wmjdbc.notifications.BasicNotification"
        notification_settings: JSON string of properties
    """
    try:
        settings = json.loads(notification_settings) if notification_settings and notification_settings != "{}" else None
    except json.JSONDecodeError as e:
        return f"Invalid JSON: {e}"
    try:
        return json.dumps(await client.adapter_notification_create_polling(
            notification_name, package_name, connection_alias, notification_template, settings,
        ), indent=2)
    except Exception as e:
        return f"Failed: {e}"

@mcp.tool()
async def adapter_notification_create_listener_based(
    notification_name: str, package_name: str,
    listener_alias: str, notification_template: str,
    notification_settings: str = "{}",
) -> str:
    """Create a listener-based notification (SAP IDoc, OPC subscription events, etc.).

    Args:
        notification_name: Full name like "mypkg.notifications:onSAPEvent"
        package_name: Package name
        listener_alias: Listener this notification is bound to
        notification_template: Full template class name
        notification_settings: JSON string of properties
    """
    try:
        settings = json.loads(notification_settings) if notification_settings and notification_settings != "{}" else None
    except json.JSONDecodeError as e:
        return f"Invalid JSON: {e}"
    try:
        return json.dumps(await client.adapter_notification_create_listener(
            notification_name, package_name, listener_alias, notification_template, settings,
        ), indent=2)
    except Exception as e:
        return f"Failed: {e}"


# ═══════════════════════════════════════════════════════════════════════
# HELPERS
# ═══════════════════════════════════════════════════════════════════════

@mcp.tool()
async def mapset_value(value: str) -> str:
    """Helper: encode a value for use in MAPSET data field.

    Returns the XMLValues-encoded string to use in the "data" key of a MAPSET node.

    Args:
        value: The string value to encode
    """
    # Escape XML special chars
    escaped = value.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace('"', "&quot;")
    return f'<Values version="2.0"><value name="xml">{escaped}</value></Values>'


if __name__ == "__main__":
    mcp.run(transport="stdio")
