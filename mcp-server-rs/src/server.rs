//! MCP server definition with tool methods.

use crate::client::ISClient;
use crate::params::*;
use rmcp::{
    RoleServer, ServerHandler, handler::server::router::tool::ToolRouter,
    handler::server::tool::ToolCallContext, handler::server::wrapper::Parameters, model::*,
    service::RequestContext, tool, tool_router,
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
    scopes: Vec<String>,
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
            scopes: Vec::new(),
        }
    }

    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
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

    // ── Scheduler ───────────────────────────────────────────────────────

    #[tool(description = "Get the current scheduler state (running/paused).")]
    async fn scheduler_state(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.scheduler_state().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "List all scheduled tasks with their status, schedule, and next run time."
    )]
    async fn scheduler_task_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.scheduler_task_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Add a scheduled task.\n\nRequired settings: service (full path e.g. \"mypkg.services:myService\"), description, type (\"once\"/\"repeat\"/\"complex\"), target (\"$any\" or specific server).\nFor once: startDate (MM/dd/yyyy), startTime (HH:mm:ss).\nFor repeat: interval (milliseconds), startDate, startTime.\nOptional: endDate, endTime, inputs (JSON string of service input params).\n\nReturns the task OID on success."
    )]
    async fn scheduler_task_add(
        &self,
        Parameters(p): Parameters<SchedulerTaskAddParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.scheduler_task_add(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get details of a scheduled task by its OID.")]
    async fn scheduler_task_get(
        &self,
        Parameters(p): Parameters<SchedulerTaskOidParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.scheduler_task_get(&p.oid).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Update a scheduled task's settings.")]
    async fn scheduler_task_update(
        &self,
        Parameters(p): Parameters<SchedulerTaskUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.scheduler_task_update(&p.oid, &settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Cancel (delete) a scheduled task by its OID.")]
    async fn scheduler_task_cancel(
        &self,
        Parameters(p): Parameters<SchedulerTaskOidParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.scheduler_task_cancel(&p.oid).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Suspend a scheduled task (pauses execution, keeps schedule).")]
    async fn scheduler_task_suspend(
        &self,
        Parameters(p): Parameters<SchedulerTaskOidParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.scheduler_task_suspend(&p.oid).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Resume a suspended scheduled task.")]
    async fn scheduler_task_resume(
        &self,
        Parameters(p): Parameters<SchedulerTaskOidParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.scheduler_task_resume(&p.oid).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Pause the entire scheduler (no tasks will execute until resumed).")]
    async fn scheduler_pause(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.scheduler_pause().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Resume the scheduler after it was paused.")]
    async fn scheduler_resume(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.scheduler_resume().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── User & Access Management ──────────────────────────────────────

    #[tool(description = "List all users with their group memberships.")]
    async fn user_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.user_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Create a new IS user.")]
    async fn user_add(
        &self,
        Parameters(p): Parameters<UserAddParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.user_add(&p.username, &p.password).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete an IS user.")]
    async fn user_delete(
        &self,
        Parameters(p): Parameters<UserNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.user_delete(&p.username).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Enable or disable an IS user account.")]
    async fn user_set_disabled(
        &self,
        Parameters(p): Parameters<UserDisableParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.user_set_disabled(&p.username, p.disabled).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List all disabled user accounts.")]
    async fn user_disabled_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.disabled_user_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List all groups with their members.")]
    async fn group_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.group_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Create a new group.")]
    async fn group_add(
        &self,
        Parameters(p): Parameters<GroupNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.group_add(&p.groupname).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a group.")]
    async fn group_delete(
        &self,
        Parameters(p): Parameters<GroupNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.group_delete(&p.groupname).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Update group membership (replaces current members with the provided list)."
    )]
    async fn group_change(
        &self,
        Parameters(p): Parameters<GroupChangeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let membership = match parse_json(&p.membership) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.group_change(&p.groupname, &membership).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List all Access Control Lists (ACLs) with their allow/deny groups.")]
    async fn acl_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.acl_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Create a new ACL.\n\nRequired settings: aclName, allowList (array of group names), denyList (array of group names)."
    )]
    async fn acl_add(
        &self,
        Parameters(p): Parameters<AclAddParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.acl_add(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete an ACL.")]
    async fn acl_delete(
        &self,
        Parameters(p): Parameters<AclNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.acl_delete(&p.acl_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get account locking settings (max login attempts, block duration, etc.)."
    )]
    async fn account_locking_get(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.account_locking_get().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── JDBC Connection Pools ──────────────────────────────────────────

    #[tool(description = "List all JDBC connection pools with their driver and description.")]
    async fn jdbc_pool_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jdbc_pool_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Create a JDBC connection pool.\n\nRequired settings: pool (name), drivers (driver alias from jdbc_driver_list), url (JDBC URL), uid (username), pwd (password).\nOptional: description, mincon, maxcon, idle (timeout).\n\nCommon JDBC URL formats:\n- SQL Server: jdbc:wm:sqlserver://host:1433;databaseName=mydb\n- PostgreSQL: jdbc:wm:postgresql://host:5432;databaseName=mydb\n- Oracle: jdbc:wm:oracle://host:1521;serviceName=mydb"
    )]
    async fn jdbc_pool_add(
        &self,
        Parameters(p): Parameters<JdbcPoolAddParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.jdbc_pool_add(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Update a JDBC connection pool's settings.")]
    async fn jdbc_pool_update(
        &self,
        Parameters(p): Parameters<JdbcPoolAddParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.jdbc_pool_update(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a JDBC connection pool.")]
    async fn jdbc_pool_delete(
        &self,
        Parameters(p): Parameters<JdbcPoolNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jdbc_pool_delete(&p.pool).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Test a JDBC connection pool.\n\nPass the same settings as jdbc_pool_add to test the connection."
    )]
    async fn jdbc_pool_test(
        &self,
        Parameters(p): Parameters<JdbcPoolTestParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.jdbc_pool_test(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Restart a JDBC connection pool (close and reopen all connections).")]
    async fn jdbc_pool_restart(
        &self,
        Parameters(p): Parameters<JdbcPoolNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jdbc_pool_restart(&p.pool).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List all available JDBC driver aliases with their class names.")]
    async fn jdbc_driver_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jdbc_driver_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "List all JDBC functional aliases (logical names mapped to connection pools)."
    )]
    async fn jdbc_function_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jdbc_function_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Global Variables ─────────────────────────────────────────────────

    #[tool(description = "List all global variables defined on the IS.")]
    async fn global_var_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.global_var_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get the value of a global variable.")]
    async fn global_var_get(
        &self,
        Parameters(p): Parameters<GlobalVarKeyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.global_var_get(&p.key).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Add a new global variable. Set is_password=true for secrets.")]
    async fn global_var_add(
        &self,
        Parameters(p): Parameters<GlobalVarAddParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .global_var_add(&p.key, &p.value, p.is_password.unwrap_or(false))
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Update the value of an existing global variable.")]
    async fn global_var_edit(
        &self,
        Parameters(p): Parameters<GlobalVarEditParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.global_var_edit(&p.key, &p.value).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Remove a global variable.")]
    async fn global_var_remove(
        &self,
        Parameters(p): Parameters<GlobalVarKeyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.global_var_remove(&p.key).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Server Monitoring & Config ─────────────────────────────────────

    #[tool(
        description = "Get server health status including adapter connections, triggers, and messaging."
    )]
    async fn server_health(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.server_health().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get server statistics: uptime, memory usage, thread counts, etc.")]
    async fn server_stats(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.server_stats().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get server settings (basic configuration).")]
    async fn server_settings(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.server_settings().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get all extended settings (watt.* properties). Returns ~1000 properties."
    )]
    async fn server_extended_settings(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.server_extended_settings().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get service execution statistics. Provide service_name for a specific service, or omit for all."
    )]
    async fn server_service_stats(
        &self,
        Parameters(p): Parameters<ServiceStatsParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.server_service_stats(p.service_name.as_deref()).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get a thread dump of the IS JVM (for diagnostics and deadlock detection)."
    )]
    async fn server_thread_dump(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.server_thread_dump().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List all active HTTP sessions.")]
    async fn server_session_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.server_session_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get IS license information (licensed features, expiry, etc.).")]
    async fn server_license_info(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.server_license_info().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get the IS server log. Optionally specify num_lines to get only the last N lines."
    )]
    async fn server_log(
        &self,
        Parameters(p): Parameters<ServerLogParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.server_log(p.num_lines.as_deref()).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get circuit breaker statistics for all services with circuit breakers enabled."
    )]
    async fn server_circuit_breaker_stats(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.server_circuit_breaker_stats().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Remote Server Aliases ──────────────────────────────────────────

    #[tool(description = "List all remote server aliases (for IS-to-IS communication).")]
    async fn remote_server_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.remote_server_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Add a remote server alias for IS-to-IS communication.\n\nRequired settings: alias (name), host, port, user, pass.\nOptional: ssl (yes/no), acl, keepalive (seconds), timeout (seconds)."
    )]
    async fn remote_server_add(
        &self,
        Parameters(p): Parameters<RemoteServerAddParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.remote_server_add(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a remote server alias.")]
    async fn remote_server_delete(
        &self,
        Parameters(p): Parameters<RemoteServerAliasParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.remote_server_delete(&p.alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Test connectivity to a remote server alias.")]
    async fn remote_server_test(
        &self,
        Parameters(p): Parameters<RemoteServerAliasParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.remote_server_test(&p.alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Auditing ────────────────────────────────────────────────────────

    #[tool(description = "List all audit loggers with their enabled/disabled status.")]
    async fn audit_logger_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.audit_logger_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get detailed configuration of an audit logger.")]
    async fn audit_logger_get(
        &self,
        Parameters(p): Parameters<AuditLoggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.audit_logger_get(&p.logger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Update audit logger settings (destination, async mode, etc.).")]
    async fn audit_logger_update(
        &self,
        Parameters(p): Parameters<AuditLoggerUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.audit_logger_update(&p.logger_name, &settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Enable an audit logger.")]
    async fn audit_logger_enable(
        &self,
        Parameters(p): Parameters<AuditLoggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.audit_logger_enable(&p.logger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Disable an audit logger.")]
    async fn audit_logger_disable(
        &self,
        Parameters(p): Parameters<AuditLoggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.audit_logger_disable(&p.logger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── OAuth Management ───────────────────────────────────────────────

    #[tool(description = "Get OAuth server settings (HTTPS requirement, PKCE, token lifetimes).")]
    async fn oauth_settings_get(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.oauth_settings_get().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Update OAuth server settings.")]
    async fn oauth_settings_update(
        &self,
        Parameters(p): Parameters<OAuthSettingsUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.oauth_settings_update(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List all registered OAuth clients.")]
    async fn oauth_client_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.oauth_client_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Register a new OAuth client.\n\nRequired settings: name, version, type (confidential/public).\nGrant types (set to \"true\"): authorization_code_allowed, implicit_allowed, client_credentials_allowed, owner_credentials_allowed.\nFor auth_code/implicit: redirect_uris is required.\nOptional: scopes, token_lifetime, enabled.\n\nReturns client_id and client_secret."
    )]
    async fn oauth_client_register(
        &self,
        Parameters(p): Parameters<OAuthClientRegisterParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.oauth_client_register(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Remove a registered OAuth client by client_id.")]
    async fn oauth_client_delete(
        &self,
        Parameters(p): Parameters<OAuthClientIdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.oauth_client_delete(&p.client_id).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List all OAuth scopes.")]
    async fn oauth_scope_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.oauth_scope_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Add an OAuth scope.\n\nRequired settings: name, values (array of service paths or scope values). Optional: description."
    )]
    async fn oauth_scope_add(
        &self,
        Parameters(p): Parameters<OAuthScopeAddParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.oauth_scope_add(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Remove an OAuth scope.")]
    async fn oauth_scope_remove(
        &self,
        Parameters(p): Parameters<OAuthScopeNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.oauth_scope_remove(&p.name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List all active OAuth access tokens.")]
    async fn oauth_token_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.oauth_token_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Web Services / REST / OpenAPI ─────────────────────────────────

    #[tool(description = "List all WS provider endpoints (SOAP web services exposed by IS).")]
    async fn ws_provider_endpoint_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.ws_provider_endpoint_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List all WS consumer endpoints (external SOAP services IS can call).")]
    async fn ws_consumer_endpoint_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.ws_consumer_endpoint_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get the WSDL document for a web service descriptor.")]
    async fn ws_wsdl_get(
        &self,
        Parameters(p): Parameters<WsEndpointNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.ws_wsdl_get(&p.name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List all REST API resources (REST descriptors) on the IS.")]
    async fn rest_resource_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.rest_resource_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get the OpenAPI document for a REST API descriptor.\n\nReturns the full OpenAPI 3.0 spec in JSON format."
    )]
    async fn openapi_doc_get(
        &self,
        Parameters(p): Parameters<OpenApiDocParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.openapi_doc_get(&p.rad_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Generate IS provider services from an OpenAPI specification.\n\nRequired settings: packageName, folderName (must exist -- use folder_create first), radName (descriptor name).\nRequired: sourceUri and openapiUrl (set both to 'inline' when providing openapiContent).\n\nRECOMMENDED APPROACH: Pass the spec content directly in openapiContent (JSON or YAML string). Set sourceUri and openapiUrl to 'inline'. IS may not be able to fetch remote URLs.\n\nTo use a remote spec: download it yourself first, then pass the content in openapiContent.\n\nOptional: isGroupByTag (default true).\n\nCreates a REST API descriptor + auto-generated service stubs and resource definitions."
    )]
    async fn openapi_generate_provider(
        &self,
        Parameters(p): Parameters<OpenApiGenerateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.openapi_generate_provider(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Generate IS consumer connectors from an OpenAPI specification.\n\nRequired settings: packageName, folderName, radName.\nProvide either sourceUri (URL) or openapiContent (inline spec)."
    )]
    async fn openapi_generate_consumer(
        &self,
        Parameters(p): Parameters<OpenApiGenerateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.openapi_generate_consumer(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Refresh a REST API provider descriptor from its OpenAPI source.")]
    async fn openapi_refresh_provider(
        &self,
        Parameters(p): Parameters<OpenApiGenerateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.openapi_refresh_provider(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Security & Keystore ──────────────────────────────────────────

    #[tool(description = "List all keystores with their configured key aliases.")]
    async fn keystore_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.keystore_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "List all truststores with their certificate aliases, types, and locations."
    )]
    async fn truststore_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.truststore_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get security settings (SSL cert requirements, hardware accelerator config)."
    )]
    async fn security_settings_get(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.security_settings_get().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Update security settings.")]
    async fn security_settings_update(
        &self,
        Parameters(p): Parameters<SecuritySettingsUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.security_settings_update(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Package Management Extended ────────────────────────────────────

    #[tool(
        description = "Delete a package from the IS (removes all contents). Package must be disabled first."
    )]
    async fn package_delete(
        &self,
        Parameters(p): Parameters<PackageNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.package_delete(&p.package_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get detailed information about a package (services, doc types, triggers, etc.)."
    )]
    async fn package_info(
        &self,
        Parameters(p): Parameters<PackageNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.package_info(&p.package_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get the dependency list for a package.")]
    async fn package_dependencies(
        &self,
        Parameters(p): Parameters<PackageNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.package_dependencies(&p.package_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List JAR files in a package's code/jars directory.")]
    async fn package_jar_list(
        &self,
        Parameters(p): Parameters<PackageNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.package_jar_list(&p.package_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Document Type Generation ────────────────────────────────────────

    #[tool(
        description = "Generate a document type from a JSON sample string.\n\nThe IS infers the structure (field names, types) from the JSON and creates a matching document type."
    )]
    async fn doctype_gen_from_json(
        &self,
        Parameters(p): Parameters<DocTypeGenJsonParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .doctype_gen_from_json(&p.json_string, &p.package_name, &p.ifc_name, &p.record_name)
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Generate a document type from a JSON Schema string.\n\nThe IS creates a document type matching the schema structure."
    )]
    async fn doctype_gen_from_json_schema(
        &self,
        Parameters(p): Parameters<DocTypeGenJsonSchemaParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .doctype_gen_from_json_schema(
                &p.json_schema,
                &p.package_name,
                &p.ifc_name,
                &p.record_name,
            )
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Generate a document type from an XSD (XML Schema Definition) string.")]
    async fn doctype_gen_from_xsd(
        &self,
        Parameters(p): Parameters<DocTypeGenXsdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .doctype_gen_from_xsd(&p.xsd_source, &p.package_name, &p.ifc_name, &p.record_name)
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Generate a document type from an XML sample string.\n\nThe IS infers the structure from the XML elements and attributes."
    )]
    async fn doctype_gen_from_xml(
        &self,
        Parameters(p): Parameters<DocTypeGenXmlParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .doctype_gen_from_xml(&p.xml_string, &p.package_name, &p.ifc_name, &p.record_name)
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── SAP Document Type Generation ────────────────────────────────────

    #[tool(
        description = "Generate a document type from an SAP IDoc type.\n\nRequires an active SAP adapter connection. The IS retrieves the IDoc metadata from the SAP system and creates a matching IS document type."
    )]
    async fn sap_idoc_doctype_create(
        &self,
        Parameters(p): Parameters<SapIDocDocTypeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let mut settings = json!({
            "systemId": p.system_id,
            "iDocType": p.idoc_type,
            "packageName": p.package_name,
            "folderName": p.folder_name,
            "documentTypeName": p.document_type_name,
        });
        if let Some(cim) = &p.cim_type {
            settings
                .as_object_mut()
                .unwrap()
                .insert("cimType".into(), json!(cim));
        }
        match c.sap_idoc_doctype_create(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Generate a document type from an SAP RFC function module or BAPI structure.\n\nRequires an active SAP adapter connection. The IS retrieves the RFC/BAPI metadata from the SAP system and creates a matching IS document type."
    )]
    async fn sap_rfc_doctype_create(
        &self,
        Parameters(p): Parameters<SapRfcDocTypeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .sap_rfc_doctype_create(&json!({
                "systemId": p.system_id,
                "structName": p.struct_name,
                "packageName": p.package_name,
                "folderName": p.folder_name,
                "documentTypeName": p.document_type_name,
            }))
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── URL Aliases ─────────────────────────────────────────────────────

    #[tool(description = "List all HTTP URL aliases configured on the IS.")]
    async fn url_alias_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.url_alias_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Add an HTTP URL alias.\n\nRequired settings: alias (name, no leading /), urlPath (e.g., \"invoke/mypkg.svc:name\" or \"restv2/mypkg.svc:name/resource\"), package.\nOptional: portList, association."
    )]
    async fn url_alias_add(
        &self,
        Parameters(p): Parameters<UrlAliasAddParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.url_alias_add(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get details of an HTTP URL alias.")]
    async fn url_alias_get(
        &self,
        Parameters(p): Parameters<UrlAliasNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.url_alias_get(&p.alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete an HTTP URL alias.")]
    async fn url_alias_delete(
        &self,
        Parameters(p): Parameters<UrlAliasNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.url_alias_delete(&p.alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Package Marketplace (packages.webmethods.io) ─────────────────

    #[tool(
        description = "Search packages on the webMethods Package Registry (packages.webmethods.io).\n\nOptional filter by name substring and/or category. Returns package name, description, category, source URL."
    )]
    async fn marketplace_search(
        &self,
        Parameters(p): Parameters<MarketplaceSearchParam>,
    ) -> Result<CallToolResult, ErrorData> {
        // Marketplace doesn't need an IS instance - it's a public registry
        let c = self.get_client(&None)?;
        match c
            .marketplace_search(
                p.filter.as_deref(),
                p.category.as_deref(),
                p.registry.as_deref(),
            )
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get detailed information about a package on the registry (description, source URL, owner, etc.)."
    )]
    async fn marketplace_package_info(
        &self,
        Parameters(p): Parameters<MarketplacePackageParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&None)?;
        match c
            .marketplace_package_info(&p.package_name, p.registry.as_deref())
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List available versions (tags) for a package on the registry.")]
    async fn marketplace_package_tags(
        &self,
        Parameters(p): Parameters<MarketplacePackageParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&None)?;
        match c
            .marketplace_package_tags(&p.package_name, p.registry.as_deref())
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "List package categories available on the registry (utility, connector, example, etc.)."
    )]
    async fn marketplace_categories(&self) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&None)?;
        match c.marketplace_categories(None).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List available package registries (public, supported, etc.).")]
    async fn marketplace_registries(&self) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&None)?;
        match c.marketplace_registries().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get git repository info for a package (GitHub URL, SSH URL, available tags).\n\nTo install a package: use the sourceUrl from the info to git clone into the IS packages directory, then use package_reload to activate it."
    )]
    async fn marketplace_package_git(
        &self,
        Parameters(p): Parameters<MarketplacePackageParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&None)?;
        match c
            .marketplace_package_git(&p.package_name, p.registry.as_deref())
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Download and install a package from the webMethods Package Registry into the IS.\n\nThis downloads the package zip from GitHub, extracts it into the IS packages directory, and activates it. Requires the MCP server to have filesystem access to the IS installation.\n\nIf tag is omitted, the latest version is installed."
    )]
    async fn marketplace_install(
        &self,
        Parameters(p): Parameters<MarketplaceInstallParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .marketplace_install(&p.package_name, p.tag.as_deref(), p.registry.as_deref())
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Pub/Sub Trigger Management ──────────────────────────────────────

    #[tool(description = "List all pub/sub triggers with their status, connection, and settings.")]
    async fn trigger_report(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.trigger_report().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Create a pub/sub trigger.\n\nRequired settings: triggerName (full ns path), package.\nKey fields: conditions (array with conditionName, serviceName, messageType array, filter array), joinTimeOut, queueCapacity, maxRetryAttempts, retryInterval, isConcurrent, maxExecutionThreads."
    )]
    async fn trigger_create(
        &self,
        Parameters(p): Parameters<TriggerCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.trigger_create(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a pub/sub trigger.")]
    async fn trigger_delete(
        &self,
        Parameters(p): Parameters<TriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.trigger_delete(&p.trigger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get properties of a pub/sub trigger (queue capacity, retry, threading, etc.)."
    )]
    async fn trigger_get_properties(
        &self,
        Parameters(p): Parameters<TriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.trigger_get_properties(&p.trigger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Update properties of a pub/sub trigger.")]
    async fn trigger_set_properties(
        &self,
        Parameters(p): Parameters<TriggerSetPropertiesParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let props = match parse_json(&p.properties) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.trigger_set_properties(&p.trigger_name, &props).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Suspend a pub/sub trigger (stops processing new documents).")]
    async fn trigger_suspend(
        &self,
        Parameters(p): Parameters<TriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.trigger_suspend(&p.trigger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get processing status of a trigger (active threads, queue counts, state)."
    )]
    async fn trigger_processing_status(
        &self,
        Parameters(p): Parameters<TriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.trigger_processing_status(&p.trigger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get retrieval status of a trigger (document retrieval state).")]
    async fn trigger_retrieval_status(
        &self,
        Parameters(p): Parameters<TriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.trigger_retrieval_status(&p.trigger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get execution statistics for a trigger (consume/process counts, latency)."
    )]
    async fn trigger_stats(
        &self,
        Parameters(p): Parameters<TriggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.trigger_stats(&p.trigger_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Messaging Connections ──────────────────────────────────────────

    #[tool(description = "List all messaging connection aliases (UM, Local) with their status.")]
    async fn messaging_connection_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.messaging_connection_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Create a messaging connection alias.\n\nRequired settings: aliasName, type (UM for Universal Messaging, LOCAL for local). For UM: um_rname (nsp://host:port). Optional: description, enabled, clientID, csqSize."
    )]
    async fn messaging_connection_create(
        &self,
        Parameters(p): Parameters<MessagingConnectionCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let settings = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.messaging_connection_create(&settings).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a messaging connection alias.")]
    async fn messaging_connection_delete(
        &self,
        Parameters(p): Parameters<MessagingConnectionNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.messaging_connection_delete(&p.alias_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Enable a messaging connection alias.")]
    async fn messaging_connection_enable(
        &self,
        Parameters(p): Parameters<MessagingConnectionNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.messaging_connection_enable(&p.alias_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Disable a messaging connection alias.")]
    async fn messaging_connection_disable(
        &self,
        Parameters(p): Parameters<MessagingConnectionNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.messaging_connection_disable(&p.alias_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "List all publishable document types (documents that can be published to messaging)."
    )]
    async fn messaging_publishable_doctypes(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.messaging_publishable_doctypes().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Get the client-side queue (CSQ) message count for a messaging connection."
    )]
    async fn messaging_csq_count(
        &self,
        Parameters(p): Parameters<MessagingConnectionNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.messaging_csq_count(&p.alias_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Clear the client-side queue (CSQ) for a messaging connection.")]
    async fn messaging_csq_clear(
        &self,
        Parameters(p): Parameters<MessagingConnectionNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.messaging_csq_clear(&p.alias_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Flow Debugging ──────────────────────────────────────────────────

    #[tool(
        description = "Start a flow debugging session.\n\nLaunches the service in debug mode. Returns a debug_oid (session ID) and the first breakpoint. Use flow_debug_execute with commands like 'stepOver' to step through.\n\nThe response includes $pipeline (current pipeline state), $current (current step path), $triggeredBreakPoint, $callStack."
    )]
    async fn flow_debug_start(
        &self,
        Parameters(p): Parameters<FlowDebugStartParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let pipeline = match &p.pipeline {
            Some(s) if !s.is_empty() => match parse_json(s) {
                Ok(v) => Some(v),
                Err(e) => return text_result(&format!("Invalid pipeline JSON: {e}")),
            },
            _ => None,
        };
        match c
            .flow_debug_start(
                &p.service,
                pipeline.as_ref(),
                p.stop_at_start.unwrap_or(true),
            )
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Execute a debug command in a flow debugging session.\n\nCommands: stepOver (execute current step), stepIn (enter invoked service), stepOut (exit to caller), resume (run to next breakpoint), stop.\n\nReturns the updated $pipeline, $current step, and $triggeredBreakPoint."
    )]
    async fn flow_debug_execute(
        &self,
        Parameters(p): Parameters<FlowDebugCommandParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.flow_debug_execute(&p.debug_oid, &p.command).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Close a flow debugging session and release resources.")]
    async fn flow_debug_close(
        &self,
        Parameters(p): Parameters<FlowDebugOidParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.flow_debug_close(&p.debug_oid).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Insert breakpoints in a debug session.")]
    async fn flow_debug_insert_breakpoints(
        &self,
        Parameters(p): Parameters<FlowDebugBreakpointParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let bp = match parse_json(&p.breakpoints) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.flow_debug_insert_breakpoints(&p.debug_oid, &bp).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Remove all breakpoints from a debug session.")]
    async fn flow_debug_remove_all_breakpoints(
        &self,
        Parameters(p): Parameters<FlowDebugOidParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.flow_debug_remove_all_breakpoints(&p.debug_oid).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Set/modify pipeline values in a paused debug session.")]
    async fn flow_debug_set_pipeline(
        &self,
        Parameters(p): Parameters<FlowDebugSetPipelineParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let pipe = match parse_json(&p.pipeline) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.flow_debug_set_pipeline(&p.debug_oid, &pipe).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Stop the currently running service in a debug session.")]
    async fn flow_debug_stop_service(
        &self,
        Parameters(p): Parameters<FlowDebugOidParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.flow_debug_stop_service(&p.debug_oid).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Unit Testing ────────────────────────────────────────────────────

    #[tool(
        description = "Run test suites from specified packages.\n\nReturns an executionID for checking status and retrieving reports. Use test_suite_packages as a JSON array of package names (e.g., '[\"MyPackage\"]')."
    )]
    async fn test_run(
        &self,
        Parameters(p): Parameters<TestRunParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let packages = match parse_json(&p.test_suite_packages) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c
            .test_run(
                &packages,
                p.test_user.as_deref(),
                p.test_user_password.as_deref(),
            )
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Check the status of a test execution (RUNNING, COMPLETED, FAILED).")]
    async fn test_check_status(
        &self,
        Parameters(p): Parameters<TestExecutionIdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.test_check_status(&p.execution_id).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get a human-readable text report for a test execution.")]
    async fn test_text_report(
        &self,
        Parameters(p): Parameters<TestExecutionIdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.test_text_report(&p.execution_id).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get a JUnit XML report for a test execution (for CI/CD integration).")]
    async fn test_junit_report(
        &self,
        Parameters(p): Parameters<TestExecutionIdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.test_junit_report(&p.execution_id).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Load a service mock.\n\nReplaces the real service with a mock for testing. Scope: 'session' (current session only) or 'global' (all sessions)."
    )]
    async fn mock_load(
        &self,
        Parameters(p): Parameters<MockLoadParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.mock_load(&p.scope, &p.service, &p.mock_object).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Clear a specific service mock.")]
    async fn mock_clear(
        &self,
        Parameters(p): Parameters<MockClearParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.mock_clear(&p.scope, &p.service).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Clear all service mocks.")]
    async fn mock_clear_all(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.mock_clear_all().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List all currently active service mocks.")]
    async fn mock_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.mock_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Suspend all service mocks (temporarily disable without removing).")]
    async fn mock_suspend(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.mock_suspend().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Resume all suspended service mocks.")]
    async fn mock_resume(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.mock_resume().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── JAR Installer ───────────────────────────────────────────────────

    #[tool(
        description = "Download and install JARs into IS by creating a dedicated package.\n\nDownloads JARs from Maven Central or URLs, creates a package with the JARs in code/jars/static/, activates it, and optionally bounces IS.\n\nJAR sources: use 'maven' for Maven coordinates (e.g., 'com.mysql:mysql-connector-j:9.2.0') or 'url' for direct download.\n\nThis is the recommended way to add JDBC drivers, JMS client libraries, or any JARs that need to be on the IS classpath.\n\nIMPORTANT: IS must be restarted (bounce=true) for JARs to appear on the classpath."
    )]
    async fn install_jars(
        &self,
        Parameters(p): Parameters<InstallJarsParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let jars = match parse_json(&p.jars) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        let desc = p
            .description
            .as_deref()
            .unwrap_or("JAR installer package created by MCP server");
        match c
            .install_jars(&jars, &p.package_name, desc, p.bounce.unwrap_or(true))
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── SFTP Client ─────────────────────────────────────────────────────

    #[tool(description = "List all SFTP server aliases.")]
    async fn sftp_server_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.sftp_server_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Create an SFTP server alias.\n\nRequired settings: alias (name), hostName, port (default 22)."
    )]
    async fn sftp_server_create(
        &self,
        Parameters(p): Parameters<SftpServerAliasCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.sftp_server_create(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get details of an SFTP server alias.")]
    async fn sftp_server_get(
        &self,
        Parameters(p): Parameters<SftpServerAliasNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.sftp_server_get(&p.alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete an SFTP server alias.")]
    async fn sftp_server_delete(
        &self,
        Parameters(p): Parameters<SftpServerAliasNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.sftp_server_delete(&p.alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List all SFTP user aliases.")]
    async fn sftp_user_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.sftp_user_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Create an SFTP user alias.\n\nRequired settings: alias (name), userName, authenticationType (password/publicKey), password or privateKeyFileLocation, sftpServerAlias."
    )]
    async fn sftp_user_create(
        &self,
        Parameters(p): Parameters<SftpUserAliasCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.sftp_user_create(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get details of an SFTP user alias.")]
    async fn sftp_user_get(
        &self,
        Parameters(p): Parameters<SftpUserAliasNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.sftp_user_get(&p.alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete an SFTP user alias.")]
    async fn sftp_user_delete(
        &self,
        Parameters(p): Parameters<SftpUserAliasNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.sftp_user_delete(&p.alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Test an SFTP connection using a user alias.")]
    async fn sftp_test_connection(
        &self,
        Parameters(p): Parameters<SftpTestConnectionParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.sftp_test_connection(&p.alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── HTTP Proxy ──────────────────────────────────────────────────────

    #[tool(description = "List all HTTP proxy server aliases.")]
    async fn proxy_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.proxy_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Create an HTTP proxy server alias.\n\nRequired settings: alias, host, port. Optional: user, password."
    )]
    async fn proxy_create(
        &self,
        Parameters(p): Parameters<ProxyCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.proxy_create(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get details of an HTTP proxy server alias.")]
    async fn proxy_get(
        &self,
        Parameters(p): Parameters<ProxyAliasNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.proxy_get(&p.alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete an HTTP proxy server alias.")]
    async fn proxy_delete(
        &self,
        Parameters(p): Parameters<ProxyAliasNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.proxy_delete(&p.alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Enable an HTTP proxy server alias.")]
    async fn proxy_enable(
        &self,
        Parameters(p): Parameters<ProxyAliasNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.proxy_enable(&p.alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Disable an HTTP proxy server alias.")]
    async fn proxy_disable(
        &self,
        Parameters(p): Parameters<ProxyAliasNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.proxy_disable(&p.alias).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── JWT Management ──────────────────────────────────────────────────

    #[tool(description = "List all trusted JWT issuers.")]
    async fn jwt_issuer_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jwt_issuer_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Add a trusted JWT issuer.\n\nRequired settings: issuerName, jwksUri or certificate info."
    )]
    async fn jwt_issuer_add(
        &self,
        Parameters(p): Parameters<JwtIssuerCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.jwt_issuer_add(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get details of a trusted JWT issuer.")]
    async fn jwt_issuer_get(
        &self,
        Parameters(p): Parameters<JwtIssuerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jwt_issuer_get(&p.issuer_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a trusted JWT issuer.")]
    async fn jwt_issuer_delete(
        &self,
        Parameters(p): Parameters<JwtIssuerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jwt_issuer_delete(&p.issuer_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get JWT global settings (skew tolerance, validation options).")]
    async fn jwt_settings_get(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.jwt_settings_get().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Update JWT global settings.")]
    async fn jwt_settings_update(
        &self,
        Parameters(p): Parameters<JwtSettingsUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.jwt_settings_update(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Quiesce Mode ────────────────────────────────────────────────────

    #[tool(description = "Get current quiesce mode status (ACTIVE or QUIESCE).")]
    async fn quiesce_status(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.quiesce_status().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Enable quiesce mode (graceful drain -- stops accepting new requests while existing ones complete)."
    )]
    async fn quiesce_enable(
        &self,
        Parameters(p): Parameters<QuiesceSetParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.quiesce_enable(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Disable quiesce mode (return to active -- start accepting new requests)."
    )]
    async fn quiesce_disable(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.quiesce_disable().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Health Indicators ───────────────────────────────────────────────

    #[tool(
        description = "List all health indicators with their enabled status (for K8s/container health checks)."
    )]
    async fn health_indicators_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.health_indicators_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get details of a specific health indicator.")]
    async fn health_indicator_get(
        &self,
        Parameters(p): Parameters<HealthIndicatorNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.health_indicator_get(&p.name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Enable or disable a health indicator.")]
    async fn health_indicator_change(
        &self,
        Parameters(p): Parameters<HealthIndicatorUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.health_indicator_change(&p.name, &s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Enterprise Gateway ──────────────────────────────────────────────

    #[tool(description = "List all Enterprise Gateway rules (deny, alert, DoS).")]
    async fn egw_rules_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.egw_rules_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get Enterprise Gateway DoS protection settings.")]
    async fn egw_dos_get(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.egw_dos_get().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Update Enterprise Gateway DoS protection settings.")]
    async fn egw_dos_update(
        &self,
        Parameters(p): Parameters<EgwDosUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.egw_dos_update(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List IP addresses denied by Enterprise Gateway.")]
    async fn egw_denied_ip_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.egw_denied_ip_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── IP Access Control ───────────────────────────────────────────────

    #[tool(description = "List IP access rules (global allow/deny list).")]
    async fn ip_access_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.ip_access_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Add an IP access rule.\n\nSettings: ip (IP address or CIDR), type (allow/deny)."
    )]
    async fn ip_access_add(
        &self,
        Parameters(p): Parameters<IpRuleAddParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.ip_access_add(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete an IP access rule.")]
    async fn ip_access_delete(
        &self,
        Parameters(p): Parameters<IpRuleParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.ip_access_delete(&p.ip).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Password Policy ─────────────────────────────────────────────────

    #[tool(description = "Get password expiry policy settings.")]
    async fn password_policy_get(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.password_policy_get().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Update password expiry policy settings.\n\nSettings: isEnabled, expirationInterval (days), expiryEmailBefore (days), emailIds."
    )]
    async fn password_policy_update(
        &self,
        Parameters(p): Parameters<PasswordPolicyUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.password_policy_update(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── Alerts ──────────────────────────────────────────────────────────

    #[tool(description = "Get alerting status (enabled/disabled).")]
    async fn alert_status(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.alert_status().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Enable alert notifiers.")]
    async fn alert_enable(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.alert_enable().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Disable all alert notifiers.")]
    async fn alert_disable(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.alert_disable().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ── WebSocket Server ────────────────────────────────────────────────

    #[tool(description = "List WebSocket sessions by port.")]
    async fn websocket_sessions_by_port(
        &self,
        Parameters(p): Parameters<WebSocketEndpointCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        let port = s.get("port").and_then(|v| v.as_str()).unwrap_or("");
        match c.websocket_sessions_by_port(port).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Close a specific WebSocket session.")]
    async fn websocket_close_session(
        &self,
        Parameters(p): Parameters<WebSocketSessionParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.websocket_close_session(&p.session_id).await {
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

    // ═══════════════════════════════════════════════════════════════
    // Namespace Dependency Analysis
    // ═══════════════════════════════════════════════════════════════

    #[tool(description = "Get nodes that depend on the given node (what uses this).")]
    async fn ns_dep_get_dependents(
        &self,
        Parameters(p): Parameters<NsDepNodeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.ns_dependency_get_dependents(&p.node_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get nodes that the given node references (what this depends on).")]
    async fn ns_dep_get_references(
        &self,
        Parameters(p): Parameters<NsDepNodeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.ns_dependency_get_references(&p.node_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get unresolved references in a package (missing dependencies).")]
    async fn ns_dep_get_unresolved(
        &self,
        Parameters(p): Parameters<NsDepUnresolvedParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.ns_dependency_get_unresolved(&p.package_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Search namespace nodes by name substring. Optional node type filter.")]
    async fn ns_dep_search(
        &self,
        Parameters(p): Parameters<NsDepSearchParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .ns_dependency_search(&p.search_string, p.node_type.as_deref())
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Preview what would change when refactoring (renaming/moving) a namespace node."
    )]
    async fn ns_dep_refactor_preview(
        &self,
        Parameters(p): Parameters<NsDepRefactorParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .ns_dependency_refactor_preview(&p.old_name, &p.new_name)
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Refactor (rename/move) a namespace node and update all references.")]
    async fn ns_dep_refactor(
        &self,
        Parameters(p): Parameters<NsDepRefactorParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.ns_dependency_refactor(&p.old_name, &p.new_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Flat File Schemas
    // ═══════════════════════════════════════════════════════════════

    #[tool(description = "Save an XML flat file schema definition.")]
    async fn flatfile_schema_save(
        &self,
        Parameters(p): Parameters<FlatFileSchemaSaveParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .flatfile_schema_save(&p.xml_content, &p.package_name, &p.schema_name)
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Create a flat file dictionary in a package.")]
    async fn flatfile_dictionary_create(
        &self,
        Parameters(p): Parameters<FlatFileDictionaryParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .flatfile_dictionary_create(&p.package_name, &p.dictionary_name)
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get a flat file schema as XML.")]
    async fn flatfile_schema_get(
        &self,
        Parameters(p): Parameters<FlatFileSchemaParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.flatfile_schema_get(&p.package_name, &p.schema_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a flat file schema.")]
    async fn flatfile_schema_delete(
        &self,
        Parameters(p): Parameters<FlatFileSchemaParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .flatfile_schema_delete(&p.package_name, &p.schema_name)
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Package Management Gaps
    // ═══════════════════════════════════════════════════════════════

    #[tool(description = "Get package settings (startup services, shutdown services, etc).")]
    async fn package_settings(
        &self,
        Parameters(p): Parameters<PackageNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.package_settings(&p.package_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Compile Java services in a package.")]
    async fn package_compile(
        &self,
        Parameters(p): Parameters<PackageNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.package_compile(&p.package_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Add a package dependency.")]
    async fn package_add_depend(
        &self,
        Parameters(p): Parameters<PackageDependParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let ver = p.version.as_deref().unwrap_or("1.0");
        match c
            .package_add_depend(&p.package_name, &p.dependency, ver)
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Remove a package dependency.")]
    async fn package_del_depend(
        &self,
        Parameters(p): Parameters<PackageDependParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.package_del_depend(&p.package_name, &p.dependency).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Add a startup service to a package.")]
    async fn package_add_startup_service(
        &self,
        Parameters(p): Parameters<PackageStartupServiceParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .package_add_startup_service(&p.package_name, &p.service)
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Remove a startup service from a package.")]
    async fn package_remove_startup_service(
        &self,
        Parameters(p): Parameters<PackageStartupServiceParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .package_remove_startup_service(&p.package_name, &p.service)
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a JAR file from a package.")]
    async fn package_jar_delete(
        &self,
        Parameters(p): Parameters<PackageJarParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.package_jar_delete(&p.package_name, &p.jar_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // URL Alias Update + DTD DocType Gen
    // ═══════════════════════════════════════════════════════════════

    #[tool(description = "Update an existing URL alias.")]
    async fn url_alias_update(
        &self,
        Parameters(p): Parameters<UrlAliasUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.url_alias_update(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Generate a document type from a DTD string.")]
    async fn doctype_gen_from_dtd(
        &self,
        Parameters(p): Parameters<DocTypeGenDtdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c
            .doctype_gen_from_dtd(&p.dtd_string, &p.package_name, &p.ifc_name, &p.record_name)
            .await
        {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Messaging Publish
    // ═══════════════════════════════════════════════════════════════

    #[tool(
        description = "Publish a document to the messaging system.\n\nSettings: documentTypeName (full ns path), document (the data). Optional: connectionAlias."
    )]
    async fn messaging_publish(
        &self,
        Parameters(p): Parameters<MessagingPublishParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.messaging_publish(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Publish a document and wait for all subscribers to process it.\n\nSettings: documentTypeName, document. Optional: connectionAlias, waitTime (ms)."
    )]
    async fn messaging_publish_and_wait(
        &self,
        Parameters(p): Parameters<MessagingPublishParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.messaging_publish_and_wait(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Deliver a document directly to a specific service (point-to-point).\n\nSettings: documentTypeName, document, destinationService."
    )]
    async fn messaging_deliver(
        &self,
        Parameters(p): Parameters<MessagingPublishParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.messaging_deliver(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Cache Manager (Tier 2)
    // ═══════════════════════════════════════════════════════════════

    #[tool(description = "List all cache managers configured on the server.")]
    async fn cache_manager_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.cache_manager_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get details of a specific cache manager.")]
    async fn cache_manager_get(
        &self,
        Parameters(p): Parameters<CacheNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.cache_manager_get(&p.name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Create a new cache manager.")]
    async fn cache_manager_create(
        &self,
        Parameters(p): Parameters<CacheSettingsParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.cache_manager_create(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Update a cache manager configuration.")]
    async fn cache_manager_update(
        &self,
        Parameters(p): Parameters<CacheUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.cache_manager_update(&p.name, &s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a cache manager.")]
    async fn cache_manager_delete(
        &self,
        Parameters(p): Parameters<CacheNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.cache_manager_delete(&p.name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Reset (clear) a named cache.")]
    async fn cache_reset(
        &self,
        Parameters(p): Parameters<CacheNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.cache_reset(&p.name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // SAML Configuration (Tier 2)
    // ═══════════════════════════════════════════════════════════════

    #[tool(description = "List SAML issuers.")]
    async fn saml_issuer_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.saml_issuer_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Add a SAML issuer.\n\nSettings: issuer (entity ID), certificate, etc.")]
    async fn saml_issuer_add(
        &self,
        Parameters(p): Parameters<SamlIssuerAddParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.saml_issuer_add(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a SAML issuer.")]
    async fn saml_issuer_delete(
        &self,
        Parameters(p): Parameters<SamlIssuerParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.saml_issuer_delete(&p.issuer).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // LDAP Configuration (Tier 2)
    // ═══════════════════════════════════════════════════════════════

    #[tool(description = "Get LDAP configuration settings.")]
    async fn ldap_settings_get(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.ldap_settings_get().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Add an LDAP server configuration.\n\nSettings: serverName, host, port, baseDN, bindDN, bindPassword, useTLS."
    )]
    async fn ldap_server_add(
        &self,
        Parameters(p): Parameters<LdapServerSettingsParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.ldap_server_add(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Edit an LDAP server configuration.")]
    async fn ldap_server_edit(
        &self,
        Parameters(p): Parameters<LdapServerSettingsParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.ldap_server_edit(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete an LDAP server configuration.")]
    async fn ldap_server_delete(
        &self,
        Parameters(p): Parameters<LdapServerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.ldap_server_delete(&p.server_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Logger Configuration (Tier 2)
    // ═══════════════════════════════════════════════════════════════

    #[tool(description = "List all configured loggers and their current log levels.")]
    async fn logger_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.logger_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get a specific logger configuration.")]
    async fn logger_get(
        &self,
        Parameters(p): Parameters<LoggerNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.logger_get(&p.name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Update a logger's settings (e.g., change log level).\n\nSettings: logLevel (Trace/Debug/Info/Warn/Error/Fatal/Off)."
    )]
    async fn logger_update(
        &self,
        Parameters(p): Parameters<LoggerUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.logger_update(&p.name, &s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get server-level logging configuration.")]
    async fn logger_server_config_get(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.logger_server_config_get().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Update server-level logging configuration.")]
    async fn logger_server_config_update(
        &self,
        Parameters(p): Parameters<LoggerServerConfigParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.logger_server_config_update(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Outbound Passwords (Tier 2)
    // ═══════════════════════════════════════════════════════════════

    #[tool(description = "Store an outbound password by handle.")]
    async fn outbound_password_store(
        &self,
        Parameters(p): Parameters<OutboundPasswordStoreParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.outbound_password_store(&p.handle, &p.password).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Retrieve an outbound password by handle.")]
    async fn outbound_password_retrieve(
        &self,
        Parameters(p): Parameters<OutboundPasswordHandleParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.outbound_password_retrieve(&p.handle).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Remove an outbound password by handle.")]
    async fn outbound_password_remove(
        &self,
        Parameters(p): Parameters<OutboundPasswordHandleParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.outbound_password_remove(&p.handle).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // ACL Extended (Tier 2)
    // ═══════════════════════════════════════════════════════════════

    #[tool(description = "Assign an ACL to a namespace node (service, document type, etc).")]
    async fn acl_assign(
        &self,
        Parameters(p): Parameters<AclAssignParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.acl_assign(&p.node_name, &p.acl_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get list of nodes that have a specific ACL assigned.")]
    async fn acl_get_nodes(
        &self,
        Parameters(p): Parameters<AclNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.acl_get_nodes_for_acl(&p.acl_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get default access settings for unauthenticated requests.")]
    async fn acl_get_default_access(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.acl_get_default_access().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Set default access settings for unauthenticated requests.")]
    async fn acl_set_default_access(
        &self,
        Parameters(p): Parameters<AclDefaultAccessParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.acl_set_default_access(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Account Locking Extended (Tier 2)
    // ═══════════════════════════════════════════════════════════════

    #[tool(description = "Update account locking settings (lockout threshold, duration, etc).")]
    async fn account_locking_update(
        &self,
        Parameters(p): Parameters<AccountLockingUpdateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.account_locking_update(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Reset account locking settings to defaults.")]
    async fn account_locking_reset(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.account_locking_reset().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "List currently locked user accounts.")]
    async fn account_locked_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.account_locked_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Unlock a locked user account.")]
    async fn account_unlock(
        &self,
        Parameters(p): Parameters<UserNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.account_unlock(&p.username).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Server Admin Gaps (Tier 2)
    // ═══════════════════════════════════════════════════════════════

    #[tool(description = "Change the global IP access type (allow or deny mode).")]
    async fn ip_access_change_type(
        &self,
        Parameters(p): Parameters<IpAccessTypeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.ip_access_change_type(&p.access_type).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Interrupt a running server thread.")]
    async fn server_thread_interrupt(
        &self,
        Parameters(p): Parameters<ThreadIdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.server_thread_interrupt(&p.thread_id).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Kill a running server thread (use with caution).")]
    async fn server_thread_kill(
        &self,
        Parameters(p): Parameters<ThreadIdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.server_thread_kill(&p.thread_id).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Kill a server session.")]
    async fn server_session_kill(
        &self,
        Parameters(p): Parameters<SessionIdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.server_session_kill(&p.session_id).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Clear the server's SSL cache.")]
    async fn server_ssl_cache_clear(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.server_ssl_cache_clear().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Enterprise Gateway CRUD (Tier 2)
    // ═══════════════════════════════════════════════════════════════

    #[tool(description = "Add an enterprise gateway threat protection rule.")]
    async fn egw_rule_add(
        &self,
        Parameters(p): Parameters<EgwRuleAddParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.egw_rule_add(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete an enterprise gateway rule.")]
    async fn egw_rule_delete(
        &self,
        Parameters(p): Parameters<EgwRuleNameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.egw_rule_delete(&p.rule_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Update an enterprise gateway rule.")]
    async fn egw_rule_update(
        &self,
        Parameters(p): Parameters<EgwRuleAddParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.egw_rule_update(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Port Access Control (Tier 2)
    // ═══════════════════════════════════════════════════════════════

    #[tool(description = "List port access configurations.")]
    async fn port_access_list(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.port_access_list().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Get access control settings for a specific port.")]
    async fn port_access_get(
        &self,
        Parameters(p): Parameters<PortAccessParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.port_access_get(&p.port).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(
        description = "Add IP/hostname nodes to a port's access control list.\n\nSettings: ipAddresses (array), hostNames (array)."
    )]
    async fn port_access_add_nodes(
        &self,
        Parameters(p): Parameters<PortAccessAddNodesParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.port_access_add_nodes(&p.port, &s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Remove an IP/hostname from a port's access control list.")]
    async fn port_access_delete_node(
        &self,
        Parameters(p): Parameters<PortAccessDeleteNodeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.port_access_delete_node(&p.port, &p.node_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Set the access type for a port (allow or deny).")]
    async fn port_access_set_type(
        &self,
        Parameters(p): Parameters<PortAccessSetTypeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.port_access_set_type(&p.port, &p.access_type).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Reset a port's access control to defaults.")]
    async fn port_access_reset(
        &self,
        Parameters(p): Parameters<PortAccessParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.port_access_reset(&p.port).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // WebSocket Extended (Tier 2)
    // ═══════════════════════════════════════════════════════════════

    #[tool(description = "Create a WebSocket endpoint.")]
    async fn websocket_endpoint_create(
        &self,
        Parameters(p): Parameters<WebSocketEndpointCreateParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.websocket_endpoint_create(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Broadcast a message to all WebSocket clients connected on a port.")]
    async fn websocket_broadcast(
        &self,
        Parameters(p): Parameters<WebSocketBroadcastParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.websocket_broadcast(&p.port, &p.message).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // WS Endpoint CRUD (Tier 2)
    // ═══════════════════════════════════════════════════════════════

    #[tool(description = "Add a web service consumer endpoint.")]
    async fn ws_consumer_endpoint_add(
        &self,
        Parameters(p): Parameters<WsEndpointAddParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.ws_consumer_endpoint_add(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Add a web service provider endpoint.")]
    async fn ws_provider_endpoint_add(
        &self,
        Parameters(p): Parameters<WsEndpointAddParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        let s = match parse_json(&p.settings) {
            Ok(v) => v,
            Err(e) => return text_result(&e),
        };
        match c.ws_provider_endpoint_add(&s).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a web service consumer endpoint.")]
    async fn ws_consumer_endpoint_delete(
        &self,
        Parameters(p): Parameters<WsEndpointDeleteParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.ws_consumer_endpoint_delete(&p.endpoint_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Delete a web service provider endpoint.")]
    async fn ws_provider_endpoint_delete(
        &self,
        Parameters(p): Parameters<WsEndpointDeleteParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.ws_provider_endpoint_delete(&p.endpoint_name).await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }

    #[tool(description = "Refresh all web service connectors.")]
    async fn ws_connector_refresh(
        &self,
        Parameters(p): Parameters<InstanceOnlyParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let c = self.get_client(&p.instance)?;
        match c.ws_connector_refresh().await {
            Ok(v) => json_result(&v),
            Err(e) => text_result(&format!("Failed: {e}")),
        }
    }
}

impl ServerHandler for WmServer {
    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        let all_tools = self.tool_router.list_all();
        let filtered: Vec<Tool> = if self.scopes.is_empty() {
            all_tools
        } else {
            all_tools
                .into_iter()
                .filter(|t| crate::scopes::is_tool_allowed(&t.name, &self.scopes))
                .collect()
        };
        std::future::ready(Ok(ListToolsResult {
            tools: filtered,
            ..Default::default()
        }))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        if !self.scopes.is_empty() && !crate::scopes::is_tool_allowed(&request.name, &self.scopes) {
            return Err(ErrorData {
                code: ErrorCode::INVALID_PARAMS,
                message: Cow::Owned(format!(
                    "Tool '{}' is not available in the current scope(s): {:?}",
                    request.name, self.scopes
                )),
                data: None,
            });
        }
        let ctx = ToolCallContext::new(self, request, context);
        self.tool_router.call(ctx).await
    }
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
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
                "CRITICAL: Before creating complex flow services, READ the resource 'wm://docs/flow-language-reference' for WmPath format\n",
                "and 'wm://docs/putnode-examples' for tested working JSON patterns. Read 'wm://docs/builtin-services' for service signatures.\n\n",
                "Key rules:\n",
                "- Services are identified by \"folder.subfolder:serviceName\" paths\n",
                "- put_node is the core API for creating/updating flow services with full logic\n",
                "- sig_in/sig_out MUST have javaclass:\"com.wm.util.Values\"\n",
                "- WmPath: /fieldName;type;dim where type: 1=String, 2=Record, 3=Object, 4=RecordRef\n",
                "- LOOP over record arrays: MAPCOPY MUST use type 4 (RecordRef) with doc type qualifier:\n",
                "  from:\"/arrayVar;4;0;pkg.folder:docType/nestedField;1;0\" (dim=0 = current iteration)\n",
                "- MAPSET for record arrays: field:\"/var;4;1;pkg.folder:docType\" with <array name=\"xml\" type=\"record\" depth=\"1\"> data\n",
                "- Create document types BEFORE using RecordRef fields\n",
                "- Use MAPDELETE after LOOP to clean up temp arrays from output\n\n",
                "MARKETPLACE: Use marketplace_search/install to find and install packages from packages.webmethods.io.\n",
                "JAR INSTALLER: Use install_jars to download JARs from Maven Central and install into IS.\n",
                "FLOW DEBUGGING: Use flow_debug_start/execute/close to step through services.\n",
                "UNIT TESTING: Use test_run to execute test suites, mock_load to mock services.\n",
            ))
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourcesResult, ErrorData>> + Send + '_ {
        std::future::ready(Ok(ListResourcesResult {
            resources: crate::resources::list(),
            ..Default::default()
        }))
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ReadResourceResult, ErrorData>> + Send + '_ {
        std::future::ready(
            crate::resources::read(&request.uri).ok_or_else(|| ErrorData {
                code: ErrorCode::INVALID_PARAMS,
                message: Cow::Owned(format!("Unknown resource: {}", request.uri)),
                data: None,
            }),
        )
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
