use rmcp::schemars;
use serde::Deserialize;

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
        description = "JSON string with the full node definition following IS Values serialization format (node_nsName, node_pkg, node_type, svc_sig, flow ...). The flow tree is validated before writing: only step types the IS flow compiler knows are accepted (INVOKE, MAP, MAPSET, MAPCOPY, MAPDELETE, MAPINVOKE, BRANCH, SEQUENCE, LOOP, RETRY, EXIT, ...); the repeat step is type \"RETRY\" (keys count, backoff, repeat-on), a BRANCH on expressions uses \"evaluate-labels\": \"true\"."
    )]
    pub node_data: String,
    #[schemars(
        description = "Read the node back after writing and compare step counts per type with what was sent (default true). A deficit is reported as an error because IS drops unknown constructs silently."
    )]
    pub verify: Option<bool>,
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
    #[schemars(
        description = "Per-call HTTP timeout in seconds for long-running services (overrides the instance timeout, WM_IS_TIMEOUT / config \"timeout\", default 30)."
    )]
    pub timeout_secs: Option<u64>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
