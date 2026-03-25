use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JdbcPoolAddParam {
    #[schemars(
        description = "JSON string with pool settings: pool (name), description, drivers (driver alias), url (JDBC URL), uid (username), pwd (password), mincon (min connections), maxcon (max connections)."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JdbcPoolNameParam {
    #[schemars(description = "JDBC connection pool alias name")]
    pub pool: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JdbcPoolTestParam {
    #[schemars(
        description = "JSON string with pool settings to test (same as addPoolAlias: pool, drivers, url, uid, pwd, mincon, maxcon)"
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
