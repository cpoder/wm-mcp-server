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
    #[schemars(
        description = "Namespace path of the connection node: \"folder.subFolder:name\". NEVER prefix it with the package name -- that goes in package_name and would create a bogus folder. For package PetstoreAPI, whose root folder is petstoreapi: correct = \"petstoreapi.connections:petstore\", wrong = \"PetstoreAPI.connections:petstore\"."
    )]
    pub connection_alias: String,
    #[schemars(description = "Package that will own the connection node, e.g. \"PetstoreAPI\"")]
    pub package_name: String,
    #[schemars(
        description = "Adapter type name from adapter_type_list -- NOT the package name. Use \"JDBCAdapter\" (the package is WmJDBCAdapter, which is REJECTED here), \"WmSAP\", \"WmOPCAdapter\", \"wmMQAdapter\"."
    )]
    pub adapter_type: String,
    #[schemars(
        description = "Connection factory class. JDBC: \"com.wm.adapter.wmjdbc.connection.JDBCConnectionFactory\""
    )]
    pub connection_factory_type: String,
    #[schemars(
        description = "JSON object (as a string) mapping each property's systemName to its value -- get the exact names from adapter_connection_metadata, do not invent them. JDBC example: {\"transactionType\":\"LOCAL_TRANSACTION\",\"driverType\":\"Default\",\"datasourceClass\":\"com.wm.dd.jdbcx.postgresql.PostgreSQLDataSource\",\"serverName\":\"localhost\",\"portNumber\":\"5432\",\"databaseName\":\"petstore\",\"user\":\"postgres\",\"password\":\"secret\",\"networkProtocol\":\"\",\"otherProperties\":\"\"}. There is no url/dbUrl/uid/pwd/host/driverClass property -- those all fail."
    )]
    pub connection_settings: String,
    #[schemars(description = "Min pool size (default 1)")]
    pub pool_min: Option<i32>,
    #[schemars(description = "Max pool size (default 10)")]
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
