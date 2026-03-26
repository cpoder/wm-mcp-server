use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JwtIssuerCreateParam {
    #[schemars(
        description = "JSON string with JWT issuer settings: issuerName, jwksUri, audience, claims, etc."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JwtIssuerNameParam {
    #[schemars(description = "JWT issuer name")]
    pub issuer_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JwtSettingsUpdateParam {
    #[schemars(
        description = "JSON string with JWT global settings: enableJwtAuth, tokenLifetime, etc."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
