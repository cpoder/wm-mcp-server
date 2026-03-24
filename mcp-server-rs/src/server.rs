//! MCP tool definitions for webMethods Integration Server.

use crate::is_client::ISClient;
use rmcp::{
    ServerHandler, handler::server::router::tool::ToolRouter, handler::server::wrapper::Parameters,
    model::*, schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;
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

// ═══════════════════════════════════════════════════════════════════════
// Parameter structs -- every struct includes an optional `instance` field
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InstanceOnlyParam {
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PackageNameParam {
    #[schemars(description = "Package name in PascalCase (e.g., \"MyNewPackage\")")]
    pub package_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NodeListParam {
    #[schemars(description = "Package name")]
    pub package: String,
    #[schemars(
        description = "Optional folder path (e.g., \"services.utils\"). Empty = package root."
    )]
    pub folder: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NodeNameParam {
    #[schemars(description = "Full namespace path (e.g., \"claudedemo.services:helloWorld\")")]
    pub name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FolderCreateParam {
    #[schemars(description = "Package name")]
    pub package: String,
    #[schemars(description = "Dot-separated path (e.g., \"services\" or \"services.utils\")")]
    pub folder_path: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FlowServiceCreateParam {
    #[schemars(description = "Package name")]
    pub package: String,
    #[schemars(description = "Path as \"folder:serviceName\" (e.g., \"services:helloWorld\")")]
    pub service_path: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PutNodeParam {
    #[schemars(
        description = "JSON string with the full node definition following IS Values serialization format"
    )]
    pub node_data: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DocumentTypeCreateParam {
    #[schemars(description = "Package name")]
    pub package: String,
    #[schemars(description = "Document path as \"folder.docTypes:docName\"")]
    pub doc_path: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ServiceInvokeParam {
    #[schemars(description = "Service path (e.g., \"claudedemo.services:helloWorld\")")]
    pub service_path: String,
    #[schemars(description = "JSON string of input parameters")]
    pub inputs: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ShutdownParam {
    #[schemars(description = "If true, restart the server instead of stopping it")]
    pub bounce: Option<bool>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PortKeyPkgParam {
    #[schemars(description = "Listener key from port_list (e.g., \"HTTPListener@5555\")")]
    pub port_key: String,
    #[schemars(description = "Package that owns the listener (e.g., \"WmRoot\")")]
    pub pkg: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PortAddParam {
    #[schemars(
        description = "JSON string with listener configuration including \"factoryKey\" and \"pkg\""
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PortUpdateParam {
    #[schemars(description = "Listener key (e.g., \"HTTPListener@5555\")")]
    pub port_key: String,
    #[schemars(description = "Package that owns the listener (e.g., \"WmRoot\")")]
    pub pkg: String,
    #[schemars(description = "JSON string with properties to update")]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterTypeParam {
    #[schemars(description = "Adapter type (e.g., \"WmSAP\", \"WmOPCAdapter\", \"JDBCAdapter\")")]
    pub adapter_type: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterConnectionMetadataParam {
    #[schemars(
        description = "Adapter type name (e.g., \"JDBCAdapter\", \"WmSAP\", \"WmOPCAdapter\")"
    )]
    pub adapter_type: String,
    #[schemars(description = "Factory class name")]
    pub connection_factory_type: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterConnectionCreateParam {
    #[schemars(description = "Alias like \"mypkg.connections:mydb\"")]
    pub connection_alias: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "\"WmJDBCAdapter\", \"WmSAP\", \"WmOPCAdapter\"")]
    pub adapter_type: String,
    #[schemars(
        description = "Factory class name, e.g. \"com.wm.adapter.wmjdbc.connection.JDBCConnectionFactory\""
    )]
    pub connection_factory_type: String,
    #[schemars(description = "JSON string of connection properties")]
    pub connection_settings: String,
    #[schemars(description = "Min pool size")]
    pub pool_min: Option<i32>,
    #[schemars(description = "Max pool size")]
    pub pool_max: Option<i32>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ConnectionAliasParam {
    #[schemars(description = "Connection alias (e.g., \"demosap:connNode_sap\")")]
    pub connection_alias: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterListenerCreateParam {
    #[schemars(description = "Alias like \"mypkg.listeners:sapListener\"")]
    pub listener_alias: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "\"WmSAP\", \"WmOPCAdapter\", etc.")]
    pub adapter_type: String,
    #[schemars(description = "Connection alias this listener uses")]
    pub connection_alias: String,
    #[schemars(description = "JSON string of listener properties")]
    pub listener_settings: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListenerAliasParam {
    #[schemars(description = "Listener alias")]
    pub listener_alias: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterServiceCreateParam {
    #[schemars(description = "Full name like \"mypkg.services:queryDb\"")]
    pub service_name: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "Connection to use (e.g., \"mypkg.connections:sqlserver\")")]
    pub connection_alias: String,
    #[schemars(
        description = "Full template class name (e.g., \"com.wm.adapter.wmjdbc.services.CustomSQL\")"
    )]
    pub service_template: String,
    #[schemars(description = "JSON string of service-specific settings")]
    pub adapter_service_settings: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterNotificationPollingParam {
    #[schemars(description = "Full name like \"mypkg.notifications:onInsert\"")]
    pub notification_name: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "Connection to use")]
    pub connection_alias: String,
    #[schemars(description = "Full template class name")]
    pub notification_template: String,
    #[schemars(description = "JSON string of properties")]
    pub notification_settings: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterNotificationListenerParam {
    #[schemars(description = "Full name like \"mypkg.notifications:onSAPEvent\"")]
    pub notification_name: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "Listener this notification is bound to")]
    pub listener_alias: String,
    #[schemars(description = "Full template class name")]
    pub notification_template: String,
    #[schemars(description = "JSON string of properties")]
    pub notification_settings: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MapsetValueParam {
    #[schemars(description = "The string value to encode")]
    pub value: String,
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
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                "webmethods-is",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(concat!(
                "MCP server for managing one or more webMethods Integration Server instances via pure HTTP API.\n\n",
                "MULTI-INSTANCE: Use list_instances to see available servers. Pass 'instance' parameter to target a specific one (omit for default).\n\n",
                "KEY CONCEPTS:\n",
                "- Packages contain services, document types, and adapter configurations\n",
                "- Services are identified by \"folder.subfolder:serviceName\" paths\n",
                "- Flow services have steps: INVOKE, MAP, BRANCH, LOOP, SEQUENCE, EXIT\n",
                "- The putNode API is the core for creating/updating services with full flow logic\n",
                "- Adapter connections link IS to external systems (SAP, JDBC, OPC)\n\n",
                "FLOW STEP TYPES AND THEIR JSON KEYS:\n",
                "- INVOKE: {type:\"INVOKE\", service:\"pub.string:concat\", validate-in:\"$none\", validate-out:\"$none\", nodes:[input_map, output_map]}\n",
                "- MAP (standalone): {type:\"MAP\", mode:\"STANDALONE\", nodes:[MAPSET/MAPCOPY/MAPDELETE nodes]}\n",
                "- MAP (input): {type:\"MAP\", mode:\"INPUT\", nodes:[...]} -- goes inside INVOKE's nodes array\n",
                "- MAP (output): {type:\"MAP\", mode:\"OUTPUT\", nodes:[...]} -- goes inside INVOKE's nodes array\n",
                "- MAPCOPY: {type:\"MAPCOPY\", from:\"/srcField;1;0\", to:\"/dstField;1;0\"}\n",
                "- MAPSET: {type:\"MAPSET\", field:\"/field;1;0\", overwrite:\"true\", d_enc:\"XMLValues\", mapseti18n:\"true\", data:\"<Values version=\\\"2.0\\\"><value name=\\\"xml\\\">theValue</value></Values>\"}\n",
                "- MAPDELETE: {type:\"MAPDELETE\", field:\"/field;1;0\"}\n",
                "- BRANCH: {type:\"BRANCH\", switch:\"/field\", nodes:[SEQUENCE children with label names]}\n",
                "- LOOP: {type:\"LOOP\", in-array:\"/arrayField\", out-array:\"/outArray\", nodes:[child steps]}\n",
                "- SEQUENCE: {type:\"SEQUENCE\", label:\"name\", exit-on:\"FAILURE\", nodes:[child steps]}\n",
                "- EXIT: {type:\"EXIT\", from:\"$flow\", signal:\"FAILURE\"}\n\n",
                "WMPATH FORMAT for field references: /fieldName;type;dim\n",
                "- type: 1=String, 2=Record, 3=Object, 4=RecordRef\n",
                "- dim: 0=scalar, 1=array\n",
                "Example: /myString;1;0 (scalar string), /myList;1;1 (string array), /myDoc;2;0 (record)",
            ))
    }
}
