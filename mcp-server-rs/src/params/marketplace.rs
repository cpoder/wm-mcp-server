use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MarketplaceSearchParam {
    #[schemars(
        description = "Search filter (package name substring). Leave empty to list all packages."
    )]
    pub filter: Option<String>,
    #[schemars(description = "Filter by category (e.g., \"utility\", \"connector\", \"example\")")]
    pub category: Option<String>,
    #[schemars(description = "Registry name (default: \"public\")")]
    pub registry: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MarketplacePackageParam {
    #[schemars(description = "Package name on the registry")]
    pub package_name: String,
    #[schemars(description = "Registry name (default: \"public\")")]
    pub registry: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MarketplaceInstallParam {
    #[schemars(description = "Package name on the registry")]
    pub package_name: String,
    #[schemars(description = "Tag/version to install (e.g., \"v2.1.0\"). Omit for latest.")]
    pub tag: Option<String>,
    #[schemars(description = "Registry name (default: \"public\")")]
    pub registry: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
