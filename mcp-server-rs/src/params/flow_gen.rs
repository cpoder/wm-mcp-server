use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DslValidateParam {
    #[schemars(
        description = "DSL source text to validate (FSL is the only DSL currently supported)"
    )]
    pub dsl_text: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FslDeployParam {
    #[schemars(description = "FSL source text to compile and deploy")]
    pub fsl_string: String,
    #[schemars(
        description = "Target package name. Must already exist on IS (use package_create first)."
    )]
    pub package_name: String,
    #[schemars(description = "Folder path within the package (e.g. \"services.utils\")")]
    pub ifc_name: String,
    #[schemars(description = "Unqualified service name to create/update")]
    pub flow_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FslExtractParam {
    #[schemars(
        description = "Full namespace path of an existing flow service (e.g. \"folder:serviceName\")"
    )]
    pub service_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
