use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JndiAliasCreateParam {
    #[schemars(
        description = "JSON string with JNDI settings: jndiAliasName, description, initialContextFactory, providerURL. Optional: securityPrincipal, securityCredentials, otherProperties."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JndiAliasNameParam {
    #[schemars(description = "JNDI provider alias name")]
    pub jndi_alias_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JndiTestLookupParam {
    #[schemars(description = "JNDI provider alias name")]
    pub jndi_alias_name: String,
    #[schemars(
        description = "JNDI name to look up (e.g., \"ConnectionFactory\", \"dynamicQueues/myQueue\")"
    )]
    pub lookup_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
