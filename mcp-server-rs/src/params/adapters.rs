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
    #[schemars(
        description = "JSON string of the template properties (adapterServiceSettings). Rules that avoid a silent refusal by the Adapter Runtime: int/boolean properties are JSON STRINGS (\"0\", \"-1\", \"false\"); empty arrays are removed automatically (a JSON [] arrives as Object[] and kills the node); the node's existence is verified after the call and the server.log lines are returned when the ART refused it. For JDBC CustomSQL / BatchInsert prefer jdbc_custom_sql_create / jdbc_batch_insert_create, which build these settings."
    )]
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
        description = "Dependent parameter values as a JSON array, one entry per dependency of the domain, in order. Scalars are strings: tableNames -> [\"catalog\",\"schema\"], columnInfo -> [\"catalog\",\"schema\",\"table\"]. An ARRAY-valued dependency (the *-prefixed kind, e.g. updateColumnNames / updateColumnTypes / updateJDBCTypes depend on *tables.columnInfo) is passed as a nested JSON array: [[\"<columnInfo string>\"]] -- the server encodes it the way the ART expects (elements escaped and newline-joined). Omit for top-level domains like catalogNames."
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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JdbcCustomSqlCreateParam {
    #[schemars(description = "Full service name, e.g. \"winfarm.adapters:selectFactChunk\"")]
    pub service_name: String,
    #[schemars(description = "Package that owns the service")]
    pub package_name: String,
    #[schemars(description = "JDBC adapter connection alias, e.g. \"winfarm.connections:dwh\"")]
    pub connection_alias: String,
    #[schemars(
        description = "SQL statement with ? bind markers (SELECT, INSERT, UPDATE, DELETE, function call ...). Vendor syntax is fine: the adapter only needs the column list, which it either parses itself or takes from `outputs`."
    )]
    pub sql: String,
    #[schemars(
        description = "JSON array of the bind parameters IN ORDER, one per ?: [{\"name\":\"from_id\",\"jdbc_type\":\"BIGINT\"}, ...]. `name` becomes a field of <service>Input; `jdbc_type` is the JDBC type name (BIGINT, INTEGER, VARCHAR, NUMERIC, TIMESTAMP, DATE, BOOLEAN ...), taken from the adapter's SQL analysis when omitted; `java_type` defaults to java.lang.String, which works for every JDBC type. Required when the SQL contains ?."
    )]
    pub inputs: Option<String>,
    #[schemars(
        description = "JSON array of the result columns [{\"name\":\"order_id\",\"jdbc_type\":\"VARCHAR\"}, ...] -> <service>Output/results[]/<name>. Omit to let the adapter analyse the SQL (customSQLcolInfo); that analysis returns -1 for joins with aliases, subqueries, functions or ||, in which case this list is required."
    )]
    pub outputs: Option<String>,
    #[schemars(
        description = "Name of an extra output field receiving the affected-row count (useful for INSERT/UPDATE/DELETE), e.g. \"rowCount\"."
    )]
    pub result_row_field: Option<String>,
    #[schemars(description = "Maximum rows returned (\"0\" = unlimited, default)")]
    pub max_row: Option<String>,
    #[schemars(description = "Query timeout in seconds (\"-1\" = driver default)")]
    pub query_timeout: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JdbcBatchInsertCreateParam {
    #[schemars(description = "Full service name, e.g. \"winfarm.adapters:insertCustomers\"")]
    pub service_name: String,
    #[schemars(description = "Package that owns the service")]
    pub package_name: String,
    #[schemars(description = "JDBC adapter connection alias")]
    pub connection_alias: String,
    #[schemars(
        description = "Catalog (database) name; omit to use the first catalog the connection reports (PostgreSQL: the database name)."
    )]
    pub catalog: Option<String>,
    #[schemars(description = "Schema name, e.g. \"dwh\"")]
    pub schema: String,
    #[schemars(description = "Table name, e.g. \"dim_customer\"")]
    pub table: String,
    #[schemars(
        description = "JSON array of column names NOT to insert (serial/identity keys, defaulted timestamps), e.g. [\"customer_key\"]"
    )]
    pub exclude_columns: Option<String>,
    #[schemars(
        description = "JSON array restricting the insert to these columns (default: every column not excluded)"
    )]
    pub include_columns: Option<String>,
    #[schemars(description = "Query timeout in seconds (\"-1\" = driver default)")]
    pub query_timeout: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
