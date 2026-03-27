use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FlatFileSchemaSaveParam {
    #[schemars(description = "XML content of the flat file schema")]
    pub xml_content: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "Schema name")]
    pub schema_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FlatFileDictionaryParam {
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "Dictionary name")]
    pub dictionary_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FlatFileSchemaParam {
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "Schema name")]
    pub schema_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
