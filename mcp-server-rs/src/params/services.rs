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
