use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SamlIssuerAddParam {
    #[schemars(
        description = "JSON string with SAML issuer settings: issuer (entity ID), certificate, etc."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SamlIssuerParam {
    #[schemars(description = "SAML issuer entity ID")]
    pub issuer: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
