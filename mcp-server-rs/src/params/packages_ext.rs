use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DocTypeGenJsonParam {
    #[schemars(description = "JSON string to generate a document type from")]
    pub json_string: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "Folder/interface name (e.g., \"mypkg.doctypes\")")]
    pub ifc_name: String,
    #[schemars(description = "Document type name")]
    pub record_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DocTypeGenXsdParam {
    #[schemars(description = "XSD source content (XML string)")]
    pub xsd_source: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "Folder/interface name")]
    pub ifc_name: String,
    #[schemars(description = "Document type name")]
    pub record_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DocTypeGenXmlParam {
    #[schemars(description = "XML sample string to infer structure from")]
    pub xml_string: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "Folder/interface name")]
    pub ifc_name: String,
    #[schemars(description = "Document type name")]
    pub record_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DocTypeGenJsonSchemaParam {
    #[schemars(description = "JSON Schema string")]
    pub json_schema: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "Folder/interface name")]
    pub ifc_name: String,
    #[schemars(description = "Document type name")]
    pub record_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UrlAliasAddParam {
    #[schemars(
        description = "JSON string with alias settings: alias (name, no leading /), urlPath (e.g., \"invoke/mypkg.svc:name\"), package. Optional: portList, association."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UrlAliasNameParam {
    #[schemars(description = "URL alias name (no leading /)")]
    pub alias: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SapIDocDocTypeParam {
    #[schemars(description = "SAP system ID (connection alias name)")]
    pub system_id: String,
    #[schemars(description = "IDoc type name (e.g., \"MATMAS05\")")]
    pub idoc_type: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "Folder name")]
    pub folder_name: String,
    #[schemars(description = "Document type name to create")]
    pub document_type_name: String,
    #[schemars(description = "CIM type (optional)")]
    pub cim_type: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SapRfcDocTypeParam {
    #[schemars(description = "SAP system ID (connection alias name)")]
    pub system_id: String,
    #[schemars(description = "RFC function module or structure name")]
    pub struct_name: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "Folder name")]
    pub folder_name: String,
    #[schemars(description = "Document type name to create")]
    pub document_type_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
