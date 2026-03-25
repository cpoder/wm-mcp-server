use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterTypeParam {
    #[schemars(description = "Adapter type (e.g., \"WmSAP\", \"WmOPCAdapter\", \"JDBCAdapter\")")]
    pub adapter_type: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterConnectionMetadataParam {
    #[schemars(
        description = "Adapter type name (e.g., \"JDBCAdapter\", \"WmSAP\", \"WmOPCAdapter\")"
    )]
    pub adapter_type: String,
    #[schemars(description = "Factory class name")]
    pub connection_factory_type: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterConnectionCreateParam {
    #[schemars(description = "Alias like \"mypkg.connections:mydb\"")]
    pub connection_alias: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "\"WmJDBCAdapter\", \"WmSAP\", \"WmOPCAdapter\"")]
    pub adapter_type: String,
    #[schemars(
        description = "Factory class name, e.g. \"com.wm.adapter.wmjdbc.connection.JDBCConnectionFactory\""
    )]
    pub connection_factory_type: String,
    #[schemars(description = "JSON string of connection properties")]
    pub connection_settings: String,
    #[schemars(description = "Min pool size")]
    pub pool_min: Option<i32>,
    #[schemars(description = "Max pool size")]
    pub pool_max: Option<i32>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ConnectionAliasParam {
    #[schemars(description = "Connection alias (e.g., \"demosap:connNode_sap\")")]
    pub connection_alias: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterListenerCreateParam {
    #[schemars(description = "Alias like \"mypkg.listeners:sapListener\"")]
    pub listener_alias: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "\"WmSAP\", \"WmOPCAdapter\", etc.")]
    pub adapter_type: String,
    #[schemars(description = "Connection alias this listener uses")]
    pub connection_alias: String,
    #[schemars(description = "JSON string of listener properties")]
    pub listener_settings: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListenerAliasParam {
    #[schemars(description = "Listener alias")]
    pub listener_alias: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterServiceCreateParam {
    #[schemars(description = "Full name like \"mypkg.services:queryDb\"")]
    pub service_name: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "Connection to use (e.g., \"mypkg.connections:sqlserver\")")]
    pub connection_alias: String,
    #[schemars(
        description = "Full template class name (e.g., \"com.wm.adapter.wmjdbc.services.CustomSQL\")"
    )]
    pub service_template: String,
    #[schemars(description = "JSON string of service-specific settings")]
    pub adapter_service_settings: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterNotificationPollingParam {
    #[schemars(description = "Full name like \"mypkg.notifications:onInsert\"")]
    pub notification_name: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "Connection to use")]
    pub connection_alias: String,
    #[schemars(description = "Full template class name")]
    pub notification_template: String,
    #[schemars(description = "JSON string of properties")]
    pub notification_settings: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterNotificationListenerParam {
    #[schemars(description = "Full name like \"mypkg.notifications:onSAPEvent\"")]
    pub notification_name: String,
    #[schemars(description = "Package name")]
    pub package_name: String,
    #[schemars(description = "Listener this notification is bound to")]
    pub listener_alias: String,
    #[schemars(description = "Full template class name")]
    pub notification_template: String,
    #[schemars(description = "JSON string of properties")]
    pub notification_settings: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterServiceTemplateListParam {
    #[schemars(description = "Adapter connection alias (e.g., \"mypkg.connections:sqlserver\")")]
    pub connection_alias: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterServiceTemplateMetadataParam {
    #[schemars(description = "Adapter connection alias")]
    pub connection_alias: String,
    #[schemars(
        description = "Service template class (e.g., \"com.wm.adapter.wmjdbc.services.Select\")"
    )]
    pub service_template: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterResourceDomainLookupParam {
    #[schemars(description = "Adapter connection alias")]
    pub connection_alias: String,
    #[schemars(description = "Service template class")]
    pub service_template: String,
    #[schemars(
        description = "Resource domain name (e.g., \"catalogNames\", \"schemaNames\", \"tableNames\", \"columnInfo\")"
    )]
    pub resource_domain_name: String,
    #[schemars(
        description = "Dependent parameter values as a JSON array of strings. E.g., for tableNames: [\"catalogName\",\"schemaName\"]. For columnInfo: [\"catalog\",\"schema\",\"table\"]. Omit for top-level domains like catalogNames."
    )]
    pub values: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterServiceGetParam {
    #[schemars(description = "Full adapter service name (e.g., \"mypkg.services:queryDb\")")]
    pub service_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdapterServiceUpdateParam {
    #[schemars(description = "Full adapter service name")]
    pub service_name: String,
    #[schemars(
        description = "JSON string of settings to update (connectionAlias, adapterServiceSettings)"
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
