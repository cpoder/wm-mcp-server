use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NodeListParam {
    #[schemars(description = "Package name")]
    pub package: String,
    #[schemars(
        description = "Optional folder path (e.g., \"services.utils\"). Empty = package root."
    )]
    pub folder: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NodeNameParam {
    #[schemars(description = "Full namespace path (e.g., \"claudedemo.services:helloWorld\")")]
    pub name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FolderCreateParam {
    #[schemars(description = "Package name")]
    pub package: String,
    #[schemars(
        description = "Dot-separated folder path, WITHOUT the package name. The root folder of a package is the package name in lowercase, and everything nests under it: \"petstoreapi\", then \"petstoreapi.api\", then \"petstoreapi.api.pets\". Create each level with its own call -- parents are not created implicitly."
    )]
    pub folder_path: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
