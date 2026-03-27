use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WsEndpointNameParam {
    #[schemars(description = "Web service descriptor name or endpoint name")]
    pub name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OpenApiDocParam {
    #[schemars(description = "REST API descriptor name (radName)")]
    pub rad_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OpenApiGenerateParam {
    #[schemars(
        description = "JSON string with settings: packageName, folderName, radName (output name). Provide either sourceUri (URL to OpenAPI spec) or openapiContent (inline JSON/YAML spec). Optional: isGroupByTag (true/false)."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WsEndpointAddParam {
    #[schemars(
        description = "JSON string with endpoint settings: endpointAlias, transport (HTTP/JMS), url, etc."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WsEndpointDeleteParam {
    #[schemars(description = "Endpoint name to delete")]
    pub endpoint_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
