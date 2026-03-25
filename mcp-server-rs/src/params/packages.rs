use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PackageNameParam {
    #[schemars(description = "Package name in PascalCase (e.g., \"MyNewPackage\")")]
    pub package_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
