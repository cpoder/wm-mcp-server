use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OAuthClientRegisterParam {
    #[schemars(
        description = "JSON string with client settings: name, version, type (confidential/public). Grant types: authorization_code_allowed, implicit_allowed, client_credentials_allowed, owner_credentials_allowed (true/false). Optional: redirect_uris, scopes, token_lifetime, enabled."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OAuthClientIdParam {
    #[schemars(description = "OAuth client ID")]
    pub client_id: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OAuthScopeAddParam {
    #[schemars(
        description = "JSON string with scope settings: name, description, values (array of scope values/service paths)."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OAuthScopeNameParam {
    #[schemars(description = "Scope name")]
    pub name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OAuthSettingsUpdateParam {
    #[schemars(
        description = "JSON string with OAuth settings: requireHTTPS, requirePKCE, authCodeLifetime, accessTokenLifetime, etc."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
