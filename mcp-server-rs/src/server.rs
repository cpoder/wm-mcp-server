//! MCP server definition with tool methods.

use crate::client::ISClient;
use crate::params::*;
use rmcp::{
    RoleServer, ServerHandler, handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters, model::*, service::RequestContext, tool, tool_handler,
    tool_router,
};
use serde_json::{Value, json};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

fn mcp_err(msg: String) -> ErrorData {
    ErrorData {
        code: ErrorCode::INTERNAL_ERROR,
        message: Cow::Owned(msg),
        data: None,
    }
}

fn text_result(s: &str) -> Result<CallToolResult, ErrorData> {
    Ok(CallToolResult::success(vec![Content::text(s)]))
}

fn json_result(v: &Value) -> Result<CallToolResult, ErrorData> {
    text_result(&serde_json::to_string_pretty(v).unwrap_or_default())
}

fn parse_json(s: &str) -> Result<Value, String> {
    serde_json::from_str(s).map_err(|e| format!("Invalid JSON: {e}"))
}

fn parse_optional_json(s: &Option<String>) -> Result<Option<Value>, String> {
    match s {
        None => Ok(None),
        Some(s) if s.is_empty() || s == "{}" => Ok(None),
        Some(s) => parse_json(s).map(Some),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// MCP Server
// ═══════════════════════════════════════════════════════════════════════

#[derive(Clone)]
pub struct WmServer {
    clients: HashMap<String, Arc<ISClient>>,
    default_instance: String,
    tool_router: ToolRouter<WmServer>,
}

impl WmServer {
    fn get_client(&self, instance: &Option<String>) -> Result<&ISClient, ErrorData> {
        let name = instance.as_deref().unwrap_or(&self.default_instance);
        self.clients.get(name).map(|c| c.as_ref()).ok_or_else(|| {
            let available: Vec<&str> = self.clients.keys().map(|s| s.as_str()).collect();
            ErrorData {
                code: ErrorCode::INVALID_PARAMS,
                message: Cow::Owned(format!(
                    "Unknown instance '{name}'. Available: {available:?}"
                )),
                data: None,
            }
        })
    }
}

#[tool_router]
impl WmServer {
    pub fn new(clients: HashMap<String, Arc<ISClient>>, default_instance: String) -> Self {
        Self {
            clients,
            default_instance,
            tool_router: Self::tool_router(),
        }
    }

    // ── Instance Management ────────────────────────────────────────────

    #[tool(
        description = "List all configured Integration Server instances.\n\nReturns instance names and which one is the default. Use an instance name in the 'instance' parameter of any other tool to target a specific server."
    )]
    async fn list_instances(&self) -> Result<CallToolResult, ErrorData> {
        let instances: Vec<Value> = self
            .clients
            .keys()
            .map(|name| {
                json!({
                    "name": name,
                    "default": name == &self.default_instance,
                })
            })
            .collect();
        json_result(&json!({ "instances": instances }))
    }

    // ── Server Status ──────────────────────────────────────────────────

    #[tool(description = "Check if the Integration Server is running and responsive.")]
    async fn is_status(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        if !c.is_running().await {
            return text_result("Server is NOT running or not responding on the configured port.");
        }
        match c.get_server_status().await {
            Ok(status) => text_result(&format!(
                "Server is RUNNING.\n{}",
                serde_json::to_string_pretty(&status).unwrap_or_default()
            )),
            Err(_) => text_result("Server is RUNNING (responding to requests)."),
        }
    }

    #[tool(
        description = "Shutdown or restart the Integration Server via HTTP API.\n\nNOTE: Starting the server is not possible via HTTP (the server must already be running to accept requests). Use your OS process manager, SSH, or the IS startup script for that."
    )]
    async fn is_shutdown(
        &self,
        Parameters(p): Parameters<ShutdownParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.shutdown(p.bounce.unwrap_or(false)).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Shutdown failed: {e}")),
        }
    }

    // ── Package Management ─────────────────────────────────────────────

    #[tool(description = "List all packages on the Integration Server.")]
    async fn package_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        json_result(&c.package_list().await.map_err(mcp_err)?)
    }

    #[tool(description = "Create and activate a new package.")]
    async fn package_create(
        &self,
        Parameters(p): Parameters<PackageNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        json_result(&c.package_create(&p.package_name).await.map_err(mcp_err)?)
    }

    #[tool(description = "Reload a package to pick up changes.")]
    async fn package_reload(
        &self,
        Parameters(p): Parameters<PackageNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        json_result(&c.package_reload(&p.package_name).await.map_err(mcp_err)?)
    }

    #[tool(description = "Enable a package.")]
    async fn package_enable(
        &self,
        Parameters(p): Parameters<PackageNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        json_result(&c.package_enable(&p.package_name).await.map_err(mcp_err)?)
    }

    #[tool(description = "Disable a package.")]
    async fn package_disable(
        &self,
        Parameters(p): Parameters<PackageNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        json_result(&c.package_disable(&p.package_name).await.map_err(mcp_err)?)
    }

    // ── Namespace Browsing ─────────────────────────────────────────────

    #[tool(description = "List services, folders, and document types in a package or folder.")]
    async fn node_list(
        &self,
        Parameters(p): Parameters<NodeListParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        json_result(
            &c.node_list(&p.package, p.folder.as_deref().unwrap_or(""))
                .await
                .map_err(mcp_err)?,
        )
    }

    #[tool(
        description = "Get the full definition of a node (service, document, connection).\n\nReturns signature, flow definition, fields, etc."
    )]
    async fn node_get(
        &self,
        Parameters(p): Parameters<NodeNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        json_result(&c.node_get(&p.name).await.map_err(mcp_err)?)
    }

    #[tool(description = "Delete a node (service, folder, document type).")]
    async fn node_delete(
        &self,
        Parameters(p): Parameters<NodeNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.node_delete(&p.name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Delete failed: {e}")),
        }
    }

    // ── Folder Management ──────────────────────────────────────────────

    #[tool(
        description = "Create a folder (namespace) in a package. Create parent folders first for nested paths."
    )]
    async fn folder_create(
        &self,
        Parameters(p): Parameters<FolderCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        json_result(
            &c.folder_create(&p.package, &p.folder_path)
                .await
                .map_err(mcp_err)?,
        )
    }

    // ── Flow Service Management ────────────────────────────────────────

    #[tool(description = "Create an empty flow service. Use put_node to add logic and signature.")]
    async fn flow_service_create(
        &self,
        Parameters(p): Parameters<FlowServiceCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        json_result(
            &c.service_create(&p.package, &p.service_path)
                .await
                .map_err(mcp_err)?,
        )
    }

    #[tool(
        description = "Create or update a namespace node (flow service, document type, etc.) via the IS putNode API.\n\nThis is THE core API for creating flow services with full logic, signatures, and flow steps.\nIt also works for updating document types with field definitions.\n\nThe node_data JSON must follow the IS Values serialization format.\n\nEXAMPLE - Complete flow service with signature and flow logic:\n{\n  \"node_nsName\": \"mypkg.services:greet\",\n  \"node_pkg\": \"MyPackage\",\n  \"node_type\": \"service\",\n  \"svc_type\": \"flow\",\n  \"svc_subtype\": \"default\",\n  \"svc_sigtype\": \"java 3.5\",\n  \"stateless\": \"yes\",\n  \"pipeline_option\": 1,\n  \"svc_sig\": {\n    \"sig_in\": {\n      \"node_type\": \"record\", \"field_type\": \"record\", \"field_dim\": \"0\", \"nillable\": \"true\",\n      \"rec_fields\": [\n        {\"node_type\": \"field\", \"field_name\": \"name\", \"field_type\": \"string\", \"field_dim\": \"0\", \"nillable\": \"true\"}\n      ]\n    },\n    \"sig_out\": {\n      \"node_type\": \"record\", \"field_type\": \"record\", \"field_dim\": \"0\", \"nillable\": \"true\",\n      \"rec_fields\": [\n        {\"node_type\": \"field\", \"field_name\": \"greeting\", \"field_type\": \"string\", \"field_dim\": \"0\", \"nillable\": \"true\"}\n      ]\n    }\n  },\n  \"flow\": {\n    \"type\": \"ROOT\", \"version\": \"3.0\", \"cleanup\": \"true\",\n    \"nodes\": [\n      {\n        \"type\": \"MAP\", \"mode\": \"STANDALONE\",\n        \"nodes\": [\n          {\"type\": \"MAPSET\", \"field\": \"/name;1;0\", \"overwrite\": \"false\",\n           \"d_enc\": \"XMLValues\", \"mapseti18n\": \"true\",\n           \"data\": \"<Values version=\\\"2.0\\\"><value name=\\\"xml\\\">World</value></Values>\"}\n        ]\n      },\n      {\n        \"type\": \"INVOKE\", \"service\": \"pub.string:concat\",\n        \"validate-in\": \"$none\", \"validate-out\": \"$none\",\n        \"nodes\": [\n          {\"type\": \"MAP\", \"mode\": \"INPUT\", \"nodes\": [\n            {\"type\": \"MAPSET\", \"field\": \"/inString1;1;0\", \"overwrite\": \"true\",\n             \"d_enc\": \"XMLValues\", \"mapseti18n\": \"true\",\n             \"data\": \"<Values version=\\\"2.0\\\"><value name=\\\"xml\\\">Hello, </value></Values>\"},\n            {\"type\": \"MAPCOPY\", \"from\": \"/name;1;0\", \"to\": \"/inString2;1;0\"}\n          ]},\n          {\"type\": \"MAP\", \"mode\": \"OUTPUT\", \"nodes\": [\n            {\"type\": \"MAPCOPY\", \"from\": \"/value;1;0\", \"to\": \"/greeting;1;0\"}\n          ]}\n        ]\n      }\n    ]\n  }\n}"
    )]
    async fn put_node(
        &self,
        Parameters(p): Parameters<PutNodeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let data = match parse_json(&p.node_data) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.put_node(&data).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("putNode failed: {e}")),
        }
    }

    // ── Document Type Management ───────────────────────────────────────

    #[tool(description = "Create a document type. Create parent folders first if needed.")]
    async fn document_type_create(
        &self,
        Parameters(p): Parameters<DocumentTypeCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        json_result(
            &c.document_type_create(&p.package, &p.doc_path)
                .await
                .map_err(mcp_err)?,
        )
    }

    // ── Service Invocation / Testing ───────────────────────────────────

    #[tool(description = "Invoke (execute/test) a service.")]
    async fn service_invoke(
        &self,
        Parameters(p): Parameters<ServiceInvokeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let inputs = match &p.inputs {
            None => None,
            Some(s) if s.is_empty() => None,
            Some(s) => match parse_json(s) {
                Ok(v) => Some(v),
                Err(e) => return text_result(&format!("Invalid JSON input: {e}")),
            },
        };
        match c.service_invoke(&p.service_path, inputs.as_ref()).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Service invocation failed: {e}")),
        }
    }

    // ── Port / Listener Management ─────────────────────────────────────

    #[tool(
        description = "List all ports/listeners (HTTP, HTTPS, FTP, FTPS, FilePolling, Email, WebSocket).\n\nReturns detailed configuration for each port including protocol, status, and settings."
    )]
    async fn port_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        json_result(&c.port_list().await.map_err(mcp_err)?)
    }

    #[tool(
        description = "List available listener factory types that can be used to create new ports.\n\nCommon factories: webMethods/HTTP, webMethods/FTP, webMethods/FTPS, webMethods/FilePolling, webMethods/Email, webMethods/WebSocket."
    )]
    async fn port_factory_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        json_result(&c.port_factory_list().await.map_err(mcp_err)?)
    }

    #[tool(description = "Get detailed configuration of a specific port/listener.")]
    async fn port_get(
        &self,
        Parameters(p): Parameters<PortKeyPkgParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.port_get(&p.port_key, &p.pkg).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Add a new port/listener (HTTP, FTP, FilePolling, etc.).\n\nThe settings JSON must include \"factoryKey\" and \"pkg\". Use port_factory_list to see types.\n\nExamples:\n- HTTP: {\"factoryKey\":\"webMethods/HTTP\",\"pkg\":\"WmRoot\",\"port\":\"5556\",\"portAlias\":\"myHttp\",\"enabled\":\"false\"}\n- FilePolling: {\"factoryKey\":\"webMethods/FilePolling\",\"pkg\":\"MyPkg\",\"portAlias\":\"myPoll\",\"monitorDir\":\"/path\",\"processingService\":\"pkg.folder:svc\",\"filePollingInterval\":\"10\",\"enabled\":\"false\"}\n- FTP: {\"factoryKey\":\"webMethods/FTP\",\"pkg\":\"WmRoot\",\"port\":\"8021\",\"portAlias\":\"myFtp\",\"enabled\":\"false\"}"
    )]
    async fn port_add(
        &self,
        Parameters(p): Parameters<PortAddParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let data = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.port_add(&data).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Update an existing port/listener configuration.")]
    async fn port_update(
        &self,
        Parameters(p): Parameters<PortUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let data = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.port_update(&p.port_key, &p.pkg, &data).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Enable a port/listener.")]
    async fn port_enable(
        &self,
        Parameters(p): Parameters<PortKeyPkgParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.port_enable(&p.port_key, &p.pkg).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Disable a port/listener.")]
    async fn port_disable(
        &self,
        Parameters(p): Parameters<PortKeyPkgParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.port_disable(&p.port_key, &p.pkg).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a port/listener.")]
    async fn port_delete(
        &self,
        Parameters(p): Parameters<PortKeyPkgParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.port_delete(&p.port_key, &p.pkg).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Adapter Connection Management ──────────────────────────────────

    #[tool(
        description = "List all registered adapter types (JDBC, SAP, OPC, MongoDB, etc.).\n\nReturns adapter names, versions, and vendors. Use the adapterName value as the adapter_type parameter in other adapter tools."
    )]
    async fn adapter_type_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.adapter_type_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get the metadata (available settings/parameters) for creating connections of a specific adapter type.\n\nUse this to discover what connectionSettings parameters are required."
    )]
    async fn adapter_connection_metadata(
        &self,
        Parameters(p): Parameters<AdapterConnectionMetadataParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .adapter_connection_metadata(&p.adapter_type, &p.connection_factory_type)
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List all adapter connections.")]
    async fn adapter_connection_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        json_result(&c.adapter_connection_list().await.map_err(mcp_err)?)
    }

    #[tool(description = "Create an adapter connection via WmART API.")]
    async fn adapter_connection_create(
        &self,
        Parameters(p): Parameters<AdapterConnectionCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.connection_settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        let pool_min = p.pool_min.unwrap_or(1);
        let pool_max = p.pool_max.unwrap_or(10);
        let mgr = json!({
            "poolable": "true",
            "minimumPoolSize": pool_min.to_string(),
            "maximumPoolSize": pool_max.to_string(),
            "poolIncrementSize": "1",
            "blockingTimeout": "1000",
            "expireTimeout": "1000",
            "startupRetryCount": "0",
            "startupBackoffSecs": "5",
        });
        match c
            .adapter_connection_create(
                &p.connection_alias,
                &p.package_name,
                &p.adapter_type,
                &p.connection_factory_type,
                &settings,
                &mgr,
            )
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Enable an adapter connection.")]
    async fn adapter_connection_enable(
        &self,
        Parameters(p): Parameters<ConnectionAliasParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.adapter_connection_enable(&p.connection_alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Disable an adapter connection.")]
    async fn adapter_connection_disable(
        &self,
        Parameters(p): Parameters<ConnectionAliasParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.adapter_connection_disable(&p.connection_alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Query state of an adapter connection.")]
    async fn adapter_connection_state(
        &self,
        Parameters(p): Parameters<ConnectionAliasParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.adapter_connection_state(&p.connection_alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Adapter Listener Management ────────────────────────────────────

    #[tool(description = "List all adapter listeners for a specific adapter type.")]
    async fn adapter_listener_list(
        &self,
        Parameters(p): Parameters<AdapterTypeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        json_result(
            &c.adapter_listener_list(&p.adapter_type)
                .await
                .map_err(mcp_err)?,
        )
    }

    #[tool(description = "Create an adapter listener.")]
    async fn adapter_listener_create(
        &self,
        Parameters(p): Parameters<AdapterListenerCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_optional_json(&p.listener_settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c
            .adapter_listener_create(
                &p.listener_alias,
                &p.package_name,
                &p.adapter_type,
                &p.connection_alias,
                settings.as_ref(),
            )
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Enable an adapter listener.")]
    async fn adapter_listener_enable(
        &self,
        Parameters(p): Parameters<ListenerAliasParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.adapter_listener_enable(&p.listener_alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Disable an adapter listener.")]
    async fn adapter_listener_disable(
        &self,
        Parameters(p): Parameters<ListenerAliasParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.adapter_listener_disable(&p.listener_alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Adapter Service Management ─────────────────────────────────────

    #[tool(
        description = "Create an adapter service (JDBC Select, Insert, CustomSQL, etc.).\n\nCommon JDBC templates:\n- com.wm.adapter.wmjdbc.services.Select\n- com.wm.adapter.wmjdbc.services.Insert\n- com.wm.adapter.wmjdbc.services.Update\n- com.wm.adapter.wmjdbc.services.Delete\n- com.wm.adapter.wmjdbc.services.CustomSQL\n- com.wm.adapter.wmjdbc.services.StoredProcedure\n- com.wm.adapter.wmjdbc.services.DynamicSQL"
    )]
    async fn adapter_service_create(
        &self,
        Parameters(p): Parameters<AdapterServiceCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_optional_json(&p.adapter_service_settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c
            .adapter_service_create(
                &p.service_name,
                &p.package_name,
                &p.connection_alias,
                &p.service_template,
                settings.as_ref(),
            )
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Adapter Notification Management ────────────────────────────────

    #[tool(description = "List adapter polling notifications for a specific adapter type.")]
    async fn adapter_notification_list(
        &self,
        Parameters(p): Parameters<AdapterTypeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        json_result(
            &c.adapter_notification_list(&p.adapter_type)
                .await
                .map_err(mcp_err)?,
        )
    }

    #[tool(
        description = "Create a polling notification (JDBC insert/update/delete detection, etc.).\n\nCommon JDBC templates:\n- com.wm.adapter.wmjdbc.notifications.InsertNotification\n- com.wm.adapter.wmjdbc.notifications.UpdateNotification\n- com.wm.adapter.wmjdbc.notifications.DeleteNotification\n- com.wm.adapter.wmjdbc.notifications.BasicNotification"
    )]
    async fn adapter_notification_create_polling(
        &self,
        Parameters(p): Parameters<AdapterNotificationPollingParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_optional_json(&p.notification_settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c
            .adapter_notification_create_polling(
                &p.notification_name,
                &p.package_name,
                &p.connection_alias,
                &p.notification_template,
                settings.as_ref(),
            )
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Create a listener-based notification (SAP IDoc, OPC subscription events, etc.)."
    )]
    async fn adapter_notification_create_listener_based(
        &self,
        Parameters(p): Parameters<AdapterNotificationListenerParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_optional_json(&p.notification_settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c
            .adapter_notification_create_listener(
                &p.notification_name,
                &p.package_name,
                &p.listener_alias,
                &p.notification_template,
                settings.as_ref(),
            )
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Adapter Metadata (Designer-like) ─────────────────────────────

    #[tool(
        description = "List available adapter service templates for a connection.\n\nReturns the service types (Select, Insert, CustomSQL, etc.) that can be created for this connection."
    )]
    async fn adapter_service_template_list(
        &self,
        Parameters(p): Parameters<AdapterServiceTemplateListParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.adapter_service_template_list(&p.connection_alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get detailed metadata for an adapter service template.\n\nReturns all configurable properties with their types, resource domains (for lookups), and defaults. Use this to understand what parameters are needed before creating a service."
    )]
    async fn adapter_service_template_metadata(
        &self,
        Parameters(p): Parameters<AdapterServiceTemplateMetadataParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .adapter_service_template_metadata(&p.connection_alias, &p.service_template)
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Look up valid values for an adapter resource domain from a live connection.\n\nThis is the key API for browsing database objects interactively -- like webMethods Designer does.\n\nCommon resource domains for JDBC Select:\n- \"catalogNames\" -- list database catalogs (no values needed)\n- \"schemaNames\" -- list schemas (values: [\"catalogName\"])\n- \"tableNames\" -- list tables (values: [\"catalogName\", \"schemaName\"])\n- \"tableTypes\" -- list table types (values: [\"catalogName\", \"schemaName\"])\n- \"columnInfo\" -- list columns (values: [\"catalogName\", \"schemaName\", \"tableName\"])\n- \"columnNames\" -- list column names for selection\n\nThe 'values' parameter provides dependent context values in order."
    )]
    async fn adapter_resource_domain_lookup(
        &self,
        Parameters(p): Parameters<AdapterResourceDomainLookupParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let values = match &p.values {
            Some(s) => match parse_json(s) {
                Ok(v) => Some(v),
                Err(e) => return text_result(&e),
            },
            None => None,
        };
        match c
            .adapter_resource_domain_lookup(
                &p.connection_alias,
                &p.service_template,
                &p.resource_domain_name,
                values.as_ref(),
            )
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get the current configuration of an existing adapter service.")]
    async fn adapter_service_get(
        &self,
        Parameters(p): Parameters<AdapterServiceGetParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.adapter_service_get(&p.service_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Update an existing adapter service's configuration (connection, settings)."
    )]
    async fn adapter_service_update(
        &self,
        Parameters(p): Parameters<AdapterServiceUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.adapter_service_update(&p.service_name, &settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Streaming Connection Aliases ──────────────────────────────────

    #[tool(description = "List all streaming connection aliases (Kafka, etc.) with their status.")]
    async fn streaming_connection_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.streaming_connection_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Create a streaming connection alias (e.g., Kafka).\n\nRequired: base_name, name, description, provider_type, package, host, client_id.\nOptional: security_protocol (none/SSL/SASL_SSL/SASL_PLAINTEXT), other_properties (newline-separated name=value)."
    )]
    async fn streaming_connection_create(
        &self,
        Parameters(p): Parameters<StreamingConnectionCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let mut settings = json!({
            "baseName": p.base_name,
            "name": p.name,
            "description": p.description,
            "type": p.provider_type,
            "package": p.package,
            "host": p.host,
            "clientId": p.client_id,
        });
        let obj = settings.as_object_mut().unwrap();
        if let Some(sp) = &p.security_protocol {
            obj.insert("securityProtocol".into(), json!(sp));
        }
        if let Some(op) = &p.other_properties {
            obj.insert("other_properties".into(), json!(op));
        }
        match c.streaming_connection_create(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Enable a streaming connection alias.")]
    async fn streaming_connection_enable(
        &self,
        Parameters(p): Parameters<StreamingConnectionNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.streaming_connection_enable(&p.name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Disable a streaming connection alias.")]
    async fn streaming_connection_disable(
        &self,
        Parameters(p): Parameters<StreamingConnectionNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.streaming_connection_disable(&p.name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a streaming connection alias (must be disabled first).")]
    async fn streaming_connection_delete(
        &self,
        Parameters(p): Parameters<StreamingConnectionNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.streaming_connection_delete(&p.name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Test a streaming connection alias by attempting to connect to the provider."
    )]
    async fn streaming_connection_test(
        &self,
        Parameters(p): Parameters<StreamingConnectionNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.streaming_connection_test(&p.name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List available streaming provider types (e.g., Kafka).")]
    async fn streaming_provider_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.streaming_providers().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Streaming Event Specifications ─────────────────────────────────

    #[tool(
        description = "List streaming event specifications (topic mappings). Optionally filter by connection alias name."
    )]
    async fn streaming_event_source_list(
        &self,
        Parameters(p): Parameters<StreamingEventSourceListParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.streaming_event_source_list(p.alias_name.as_deref()).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Create a streaming event specification (topic mapping).\n\nMaps a Kafka topic to an IS event with key/value type definitions.\nKey/value types: none, RAW, STRING, JSON, XML, DOUBLE, FLOAT, INTEGER, LONG."
    )]
    async fn streaming_event_source_create(
        &self,
        Parameters(p): Parameters<StreamingEventSourceCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let mut settings = json!({
            "create_aliasName": p.create_alias_name,
            "create_referenceId": p.create_reference_id,
            "topicName": p.topic_name,
        });
        let obj = settings.as_object_mut().unwrap();
        if let Some(v) = &p.key_type {
            obj.insert("keyType".into(), json!(v));
        }
        if let Some(v) = &p.value_type {
            obj.insert("valueType".into(), json!(v));
        }
        if let Some(v) = &p.key_type_document_type {
            obj.insert("keyType_documentType".into(), json!(v));
        }
        if let Some(v) = &p.value_type_document_type {
            obj.insert("valueType_documentType".into(), json!(v));
        }
        if let Some(v) = &p.key_type_charset {
            obj.insert("keyType_charset".into(), json!(v));
        }
        if let Some(v) = &p.value_type_charset {
            obj.insert("valueType_charset".into(), json!(v));
        }
        match c.streaming_event_source_create(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a streaming event specification.")]
    async fn streaming_event_source_delete(
        &self,
        Parameters(p): Parameters<StreamingEventSourceDeleteParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .streaming_event_source_delete(&p.alias_name, &p.reference_id)
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Streaming Triggers ─────────────────────────────────────────────

    #[tool(description = "List all streaming triggers with their status.")]
    async fn streaming_trigger_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.streaming_trigger_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Enable a streaming trigger (starts consuming events).")]
    async fn streaming_trigger_enable(
        &self,
        Parameters(p): Parameters<StreamingTriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.streaming_trigger_enable(&p.name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Disable a streaming trigger (stops and disconnects).")]
    async fn streaming_trigger_disable(
        &self,
        Parameters(p): Parameters<StreamingTriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.streaming_trigger_disable(&p.name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Suspend a streaming trigger (pauses but stays connected).")]
    async fn streaming_trigger_suspend(
        &self,
        Parameters(p): Parameters<StreamingTriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.streaming_trigger_suspend(&p.name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── JNDI Provider Aliases ──────────────────────────────────────────

    #[tool(description = "List all JNDI provider aliases.")]
    async fn jndi_alias_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jndi_alias_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Create or update a JNDI provider alias.\n\nRequired settings: jndiAliasName, description, initialContextFactory, providerURL.\nOptional: securityPrincipal, securityCredentials, otherProperties.\n\nCommon initialContextFactory values:\n- ActiveMQ: org.apache.activemq.jndi.ActiveMQInitialContextFactory\n- Universal Messaging: com.pcbsys.nirvana.nSpace.NirvanaContextFactory\n- File system: com.sun.jndi.fscontext.RefFSContextFactory\n- LDAP: com.sun.jndi.ldap.LdapCtxFactory\n\nIMPORTANT: The JNDI provider JARs must be in IS classpath (WmART/code/jars/static/) and IS must be restarted after adding JARs."
    )]
    async fn jndi_alias_set(
        &self,
        Parameters(p): Parameters<JndiAliasCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.jndi_alias_set(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get details of a JNDI provider alias.")]
    async fn jndi_alias_get(
        &self,
        Parameters(p): Parameters<JndiAliasNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jndi_alias_get(&p.jndi_alias_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a JNDI provider alias.")]
    async fn jndi_alias_delete(
        &self,
        Parameters(p): Parameters<JndiAliasNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jndi_alias_delete(&p.jndi_alias_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Test a JNDI lookup against a provider alias. Use to verify the JNDI connection works."
    )]
    async fn jndi_test_lookup(
        &self,
        Parameters(p): Parameters<JndiTestLookupParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jndi_test_lookup(&p.jndi_alias_name, &p.lookup_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "List available JNDI provider templates (UM, ActiveMQ, LDAP, file system, etc.)."
    )]
    async fn jndi_template_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jndi_templates().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── JMS Messaging ──────────────────────────────────────────────────

    #[tool(description = "List all JMS connection aliases with their status and configuration.")]
    async fn jms_connection_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jms_connection_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Create a JMS connection alias.\n\nRequired settings: aliasName, description, jndiProviderUrl, connectionFactoryLookupName, user, password, clientID.\nOptional: transactionType (0=no tx, 1=local, 2=xa), enabled (true/false)."
    )]
    async fn jms_connection_create(
        &self,
        Parameters(p): Parameters<JmsConnectionCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.jms_connection_create(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Update an existing JMS connection alias.")]
    async fn jms_connection_update(
        &self,
        Parameters(p): Parameters<JmsConnectionUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.jms_connection_update(&p.alias_name, &settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a JMS connection alias (must be disabled first).")]
    async fn jms_connection_delete(
        &self,
        Parameters(p): Parameters<JmsConnectionNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jms_connection_delete(&p.alias_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Enable a JMS connection alias.")]
    async fn jms_connection_enable(
        &self,
        Parameters(p): Parameters<JmsConnectionNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jms_connection_enable(&p.alias_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Disable a JMS connection alias.")]
    async fn jms_connection_disable(
        &self,
        Parameters(p): Parameters<JmsConnectionNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jms_connection_disable(&p.alias_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List all JMS triggers with their status and configuration.")]
    async fn jms_trigger_report(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jms_trigger_report().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Create a JMS trigger.\n\nRequired settings: triggerName (full ns path), packageName, connectionAlias, destinationName, destinationType (QUEUE/TOPIC), serviceName (service to invoke on message)."
    )]
    async fn jms_trigger_create(
        &self,
        Parameters(p): Parameters<JmsTriggerCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.jms_trigger_create(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Update a JMS trigger configuration.")]
    async fn jms_trigger_update(
        &self,
        Parameters(p): Parameters<JmsTriggerUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.jms_trigger_update(&p.trigger_name, &settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a JMS trigger.")]
    async fn jms_trigger_delete(
        &self,
        Parameters(p): Parameters<JmsTriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jms_trigger_delete(&p.trigger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Enable a JMS trigger (starts consuming messages).")]
    async fn jms_trigger_enable(
        &self,
        Parameters(p): Parameters<JmsTriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jms_trigger_enable(&p.trigger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Disable a JMS trigger (stops consuming messages).")]
    async fn jms_trigger_disable(
        &self,
        Parameters(p): Parameters<JmsTriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jms_trigger_disable(&p.trigger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Suspend a JMS trigger (pauses but stays connected).")]
    async fn jms_trigger_suspend(
        &self,
        Parameters(p): Parameters<JmsTriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jms_trigger_suspend(&p.trigger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List JMS destinations (queues/topics) available on a connection.")]
    async fn jms_destination_list(
        &self,
        Parameters(p): Parameters<JmsDestinationListParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jms_destination_list(&p.alias_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── MQTT Messaging ─────────────────────────────────────────────────

    #[tool(description = "List all MQTT connection aliases with their status and configuration.")]
    async fn mqtt_connection_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.mqtt_connection_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Create an MQTT connection alias.\n\nRequired settings: aliasName, description, brokerURL (e.g., \"tcp://host:1883\"), clientID.\nOptional: cleanSession, keepAliveInterval, connectionTimeout, user, password."
    )]
    async fn mqtt_connection_create(
        &self,
        Parameters(p): Parameters<MqttConnectionCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.mqtt_connection_create(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Update an existing MQTT connection alias.")]
    async fn mqtt_connection_update(
        &self,
        Parameters(p): Parameters<MqttConnectionUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.mqtt_connection_update(&p.alias_name, &settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete an MQTT connection alias (must be disabled first).")]
    async fn mqtt_connection_delete(
        &self,
        Parameters(p): Parameters<MqttConnectionNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.mqtt_connection_delete(&p.alias_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Enable an MQTT connection alias.")]
    async fn mqtt_connection_enable(
        &self,
        Parameters(p): Parameters<MqttConnectionNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.mqtt_connection_enable(&p.alias_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Disable an MQTT connection alias.")]
    async fn mqtt_connection_disable(
        &self,
        Parameters(p): Parameters<MqttConnectionNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.mqtt_connection_disable(&p.alias_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List all MQTT triggers with their status and configuration.")]
    async fn mqtt_trigger_report(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.mqtt_trigger_report().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Create an MQTT trigger.\n\nRequired settings: triggerName (full ns path), packageName, connectionAlias, topicName, qos (0/1/2), serviceName (service to invoke on message)."
    )]
    async fn mqtt_trigger_create(
        &self,
        Parameters(p): Parameters<MqttTriggerCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.mqtt_trigger_create(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete an MQTT trigger.")]
    async fn mqtt_trigger_delete(
        &self,
        Parameters(p): Parameters<MqttTriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.mqtt_trigger_delete(&p.trigger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Enable an MQTT trigger (starts subscribing to messages).")]
    async fn mqtt_trigger_enable(
        &self,
        Parameters(p): Parameters<MqttTriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.mqtt_trigger_enable(&p.trigger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Disable an MQTT trigger (stops subscribing).")]
    async fn mqtt_trigger_disable(
        &self,
        Parameters(p): Parameters<MqttTriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.mqtt_trigger_disable(&p.trigger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Suspend an MQTT trigger (pauses but stays connected).")]
    async fn mqtt_trigger_suspend(
        &self,
        Parameters(p): Parameters<MqttTriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.mqtt_trigger_suspend(&p.trigger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Helpers ─────────────────────────────────────────────────────────

    #[tool(
        description = "Helper: encode a value for use in MAPSET data field.\n\nReturns the XMLValues-encoded string to use in the \"data\" key of a MAPSET node."
    )]
    async fn mapset_value(
        &self,
        Parameters(p): Parameters<MapsetValueParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let escaped = p
            .value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;");
        text_result(&format!(
            "<Values version=\"2.0\"><value name=\"xml\">{escaped}</value></Values>"
        ))
    }
}

#[tool_handler]
impl ServerHandler for WmServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
        )
            .with_server_info(Implementation::new(
                "webmethods-is",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(concat!(
                "MCP server for managing webMethods Integration Server via pure HTTP API.\n\n",
                "CRITICAL: Only use tools provided by this MCP server. NEVER guess or invent IS service paths.\n",
                "CRITICAL: Read each tool's parameter schema carefully before calling it. Check required vs optional fields.\n\n",
                "MULTI-INSTANCE: Use list_instances to see available servers. Pass 'instance' parameter to target a specific one (omit for default).\n\n",
                "BROWSING DATABASE OBJECTS (replacing Designer):\n",
                "To browse tables, columns, etc. from a live JDBC connection, use this workflow:\n",
                "1. adapter_resource_domain_lookup(connection_alias, service_template, resource_domain_name=\"catalogNames\") -> list catalogs\n",
                "2. adapter_resource_domain_lookup(..., resource_domain_name=\"schemaNames\", values=[\"catalog\"]) -> list schemas\n",
                "3. adapter_resource_domain_lookup(..., resource_domain_name=\"tableNames\", values=[\"catalog\",\"schema\"]) -> list tables\n",
                "4. adapter_resource_domain_lookup(..., resource_domain_name=\"columnInfo\", values=[\"catalog\",\"schema\",\"table\"]) -> list columns\n",
                "Use service_template=\"com.wm.adapter.wmjdbc.services.Select\" for querying.\n",
                "NEVER call service_invoke with made-up IS service paths to browse metadata -- use the tools above.\n\n",
                "ADAPTER SERVICE CREATION (Select, Insert, Update, Delete):\n",
                "IMPORTANT: adapter_service_create creates an EMPTY shell unless you pass complete adapter_service_settings.\n",
                "You MUST include table and column configuration, otherwise the service will have NO inputs or outputs.\n\n",
                "Step 1 -- Browse the database:\n",
                "  adapter_resource_domain_lookup(connection_alias, service_template, \"catalogNames\") -> pick catalog\n",
                "  adapter_resource_domain_lookup(..., \"schemaNames\", values=[\"catalog\"]) -> pick schema\n",
                "  adapter_resource_domain_lookup(..., \"tableNames\", values=[\"catalog\",\"schema\"]) -> pick table\n",
                "  adapter_resource_domain_lookup(..., \"columnInfo\", values=[\"catalog\",\"schema\",\"table\"]) -> get columns\n",
                "  The columnInfo response contains column names, types, JDBC type codes, and ordinals.\n\n",
                "Step 2 -- Build adapter_service_settings with ALL required fields from the column metadata.\n\n",
                "EXAMPLE -- Select service settings for an 'orders' table with columns id(int), customer_name(nvarchar), product(nvarchar):\n",
                "{\"tables.tableIndexes\":[\"T1\"],\"tables.catalogName\":[\"<current catalog>\"],\"tables.schemaName\":[\"dbo\"],\n",
                " \"tables.tableName\":[\"orders\"],\"tables.tableType\":[\"TABLE\"],\"tables.realSchemaName\":[\"dbo\"],\n",
                " \"tables.columnInfo\":[\"id\\nint NOT NULL\\n4\\n1\\n....\"],\n",
                " \"select.expression\":[\"T1.id\",\"T1.customer_name\",\"T1.product\"],\n",
                " \"select.refColumn\":[\"T1.id\",\"T1.customer_name\",\"T1.product\"],\n",
                " \"select.columnType\":[\"int NOT NULL\",\"nvarchar NULL\",\"nvarchar NULL\"],\n",
                " \"select.JDBCType\":[\"INTEGER\",\"NVARCHAR\",\"NVARCHAR\"],\n",
                " \"select.outputFieldType\":[\"java.lang.String\",\"java.lang.String\",\"java.lang.String\"],\n",
                " \"select.resultFieldType\":[\"java.lang.String\",\"java.lang.String\",\"java.lang.String\"],\n",
                " \"select.outputField\":[\"id\",\"customer_name\",\"product\"],\n",
                " \"select.resultField\":[\"id\",\"customer_name\",\"product\"],\n",
                " \"select.realOutputField\":[\"id\",\"customer_name\",\"product\"]}\n\n",
                "EXAMPLE -- Insert service settings for the same table:\n",
                "{\"tables.tableIndexes\":[\"T1\"],\"tables.catalogName\":[\"<current catalog>\"],\"tables.schemaName\":[\"dbo\"],\n",
                " \"tables.tableName\":[\"orders\"],\"tables.tableType\":[\"TABLE\"],\"tables.realSchemaName\":[\"dbo\"],\n",
                " \"tables.columnInfo\":[\"...\"],\n",
                " \"update.column\":[\"customer_name\",\"product\",\"quantity\"],\n",
                " \"update.columnType\":[\"nvarchar(100) NULL\",\"nvarchar(100) NULL\",\"int NULL\"],\n",
                " \"update.JDBCType\":[\"NVARCHAR\",\"NVARCHAR\",\"INTEGER\"],\n",
                " \"update.inputField\":[\"customer_name\",\"product\",\"quantity\"],\n",
                " \"update.inputFieldType\":[\"java.lang.String\",\"java.lang.String\",\"java.lang.String\"]}\n",
                "Note: For Insert, exclude identity/auto-increment columns (like 'id') from update.* arrays.\n\n",
                "Step 3 -- Create: adapter_service_create(service_name, package_name, connection_alias, service_template, adapter_service_settings)\n",
                "Step 4 -- Verify: adapter_service_get(service_name) -> check inputs/outputs are populated\n\n",
                "STREAMING (Kafka):\n",
                "Use streaming_* tools for Kafka connections, event specifications, and triggers.\n",
                "Use streaming_provider_list to see available providers.\n\n",
                "FLOW SERVICES:\n",
                "- Services are identified by \"folder.subfolder:serviceName\" paths\n",
                "- The put_node tool is the core for creating/updating flow services with full logic\n",
                "- Flow step types: INVOKE, MAP, BRANCH, LOOP, SEQUENCE, EXIT\n",
                "- INVOKE: {type:\"INVOKE\", service:\"pub.string:concat\", validate-in:\"$none\", validate-out:\"$none\", nodes:[input_map, output_map]}\n",
                "- MAP: {type:\"MAP\", mode:\"STANDALONE|INPUT|OUTPUT\", nodes:[MAPSET/MAPCOPY/MAPDELETE]}\n",
                "- MAPCOPY: {type:\"MAPCOPY\", from:\"/srcField;1;0\", to:\"/dstField;1;0\"}\n",
                "- MAPSET: {type:\"MAPSET\", field:\"/field;1;0\", overwrite:\"true\", d_enc:\"XMLValues\", mapseti18n:\"true\", data:\"<Values version=\\\"2.0\\\"><value name=\\\"xml\\\">theValue</value></Values>\"}\n",
                "- BRANCH: {type:\"BRANCH\", switch:\"/field\", nodes:[SEQUENCE children]}\n",
                "- LOOP: {type:\"LOOP\", in-array:\"/arrayField\", out-array:\"/outArray\", nodes:[...]}\n",
                "- SEQUENCE: {type:\"SEQUENCE\", label:\"name\", exit-on:\"FAILURE\", nodes:[...]}\n",
                "- EXIT: {type:\"EXIT\", from:\"$flow\", signal:\"FAILURE\"}\n",
                "- WmPath format: /fieldName;type;dim (type: 1=String, 2=Record, 3=Object, 4=RecordRef; dim: 0=scalar, 1=array)\n",
            ))
    }

    fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListPromptsResult, ErrorData>> + Send + '_ {
        std::future::ready(Ok(ListPromptsResult {
            prompts: crate::prompts::list(),
            ..Default::default()
        }))
    }

    fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<GetPromptResult, ErrorData>> + Send + '_ {
        std::future::ready(crate::prompts::get(&request.name).ok_or_else(|| ErrorData {
            code: ErrorCode::INVALID_PARAMS,
            message: Cow::Owned(format!("Unknown prompt: {}", request.name)),
            data: None,
        }))
    }
}
