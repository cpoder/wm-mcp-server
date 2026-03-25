//! MCP prompt definitions for interactive adapter/connector setup wizards.

use rmcp::model::*;

/// Build the list of all available prompts.
pub fn list() -> Vec<Prompt> {
    vec![
        Prompt::new(
            "setup_kafka_streaming",
            Some(
                "Interactive setup wizard for a Kafka streaming connection alias, event specification, and trigger on webMethods IS.",
            ),
            Some(vec![
                PromptArgument::new("instance")
                    .with_description("Target IS instance name (omit for default)")
                    .with_required(false),
            ]),
        ),
        Prompt::new(
            "setup_jdbc_connection",
            Some(
                "Interactive setup wizard for a JDBC adapter connection and adapter service on webMethods IS.",
            ),
            Some(vec![
                PromptArgument::new("instance")
                    .with_description("Target IS instance name (omit for default)")
                    .with_required(false),
            ]),
        ),
        Prompt::new(
            "setup_sap_connection",
            Some("Interactive setup wizard for an SAP adapter connection on webMethods IS."),
            Some(vec![
                PromptArgument::new("instance")
                    .with_description("Target IS instance name (omit for default)")
                    .with_required(false),
            ]),
        ),
    ]
}

/// Handle a get_prompt request by name.
pub fn get(name: &str) -> Option<GetPromptResult> {
    let text = match name {
        "setup_kafka_streaming" => KAFKA_WIZARD,
        "setup_jdbc_connection" => JDBC_WIZARD,
        "setup_sap_connection" => SAP_WIZARD,
        _ => return None,
    };

    Some(GetPromptResult::new(vec![PromptMessage::new_text(
        PromptMessageRole::User,
        text,
    )]))
}

const KAFKA_WIZARD: &str = "\
Guide me through setting up a Kafka streaming connection on webMethods Integration Server. \
Ask me for each of the following parameters one at a time (or in small groups). \
Use sensible defaults where possible and tell me what the default is.

1. **Package name** -- which IS package should own this connection? (list available packages first with package_list)
2. **Connection alias base name** -- a short identifier (letters, digits, underscores, must start with a letter)
3. **Description** -- what this connection is for
4. **Provider URI** -- Kafka bootstrap servers (e.g., localhost:9092)
5. **Client prefix** -- client ID prefix for Kafka (auto-generated default is fine)
6. **Security protocol** -- one of: None, SSL, SASL_SSL, SASL_PLAINTEXT
7. **Configuration parameters** -- any extra Kafka properties as name=value pairs (optional)

After collecting all parameters, use streaming_connection_create to create the connection, \
then streaming_connection_enable to enable it, and streaming_connection_test to verify it works.

Then ask if I want to create an **event specification** (topic mapping) for this connection:
1. **Event specification name** -- identifier for this spec
2. **Topic name** -- Kafka topic
3. **Key type** -- none, RAW, STRING, JSON, XML, Double, Float, Integer, Long
4. **Value type** -- same options as key type
5. **Document type name** -- if JSON or XML type, the IS document type to map to (optional)

Use streaming_event_source_create to create it.";

const JDBC_WIZARD: &str = "\
Guide me through setting up a JDBC adapter connection on webMethods Integration Server. \
Ask me for each of the following parameters one at a time (or in small groups). \
Use sensible defaults where possible and tell me what the default is.

1. **Package name** -- which IS package should own this connection?
2. **Connection alias** -- full path like \"mypkg.connections:mydb\"
3. **Database type** -- SQL Server, PostgreSQL, Oracle, MySQL, etc. (this determines the datasourceClass)
4. **Server hostname** -- database server address
5. **Port number** -- database port (default depends on DB type: 1433 for SQL Server, 5432 for PostgreSQL, 1521 for Oracle, 3306 for MySQL)
6. **Database name** -- the database/schema to connect to
7. **Username** -- database login
8. **Password** -- database password
9. **Transaction type** -- NO_TRANSACTION, LOCAL_TRANSACTION, or XA_TRANSACTION (default: NO_TRANSACTION)
10. **Additional properties** -- any extra JDBC properties (e.g., encrypt=false for SQL Server)

Common datasourceClass values:
- SQL Server: com.microsoft.sqlserver.jdbc.SQLServerDataSource
- PostgreSQL: org.postgresql.ds.PGSimpleDataSource
- Oracle: oracle.jdbc.pool.OracleDataSource
- MySQL: com.mysql.cj.jdbc.MysqlDataSource

After collecting all parameters, use adapter_connection_create with adapter_type=\"JDBCAdapter\" \
and connection_factory_type=\"com.wm.adapter.wmjdbc.connection.JDBCConnectionFactory\" to create it, \
then adapter_connection_enable to enable it.

Then ask if I want to create an **adapter service** (CustomSQL query) for this connection.";

const SAP_WIZARD: &str = "\
Guide me through setting up an SAP adapter connection on webMethods Integration Server. \
Ask me for each of the following parameters one at a time. \
Use sensible defaults where possible and tell me what the default is.

1. **Package name** -- which IS package should own this connection?
2. **Connection alias** -- full path like \"mypkg.connections:sap\"
3. **SAP application server** -- hostname of the SAP system
4. **System number** -- SAP system number (e.g., 00)
5. **Client** -- SAP client number (e.g., 001)
6. **Username** -- SAP login user
7. **Password** -- SAP password
8. **Language** -- SAP logon language (default: en)
9. **Pool size** -- min and max connection pool size (default: 1-10)

After collecting all parameters, use adapter_connection_create with adapter_type=\"WmSAP\" \
and connection_factory_type=\"com.wm.adapter.sap.spi.SAPConnectionFactory\" to create it, \
then adapter_connection_enable to enable it.

Then ask if I want to set up an **SAP listener** (for RFC or IDoc events).";
