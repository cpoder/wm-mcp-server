//! MCP prompt definitions for interactive adapter/connector setup wizards.

use rmcp::model::*;

fn instance_arg() -> PromptArgument {
    PromptArgument::new("instance")
        .with_description("Target IS instance name (omit for default)")
        .with_required(false)
}

fn prompt(name: &str, desc: &str) -> Prompt {
    Prompt::new(name, Some(desc), Some(vec![instance_arg()]))
}

/// Build the list of all available prompts.
pub fn list() -> Vec<Prompt> {
    vec![
        prompt(
            "setup_kafka_streaming",
            "Interactive setup wizard for a Kafka streaming connection alias, event specification, and trigger.",
        ),
        prompt(
            "setup_jdbc_connection",
            "Interactive setup wizard for a JDBC adapter connection and adapter service.",
        ),
        prompt(
            "setup_sap_connection",
            "Interactive setup wizard for an SAP adapter connection.",
        ),
        prompt(
            "setup_jms_connection",
            "Interactive setup wizard for a JMS connection alias with JNDI provider and trigger.",
        ),
        prompt(
            "setup_mqtt_connection",
            "Interactive setup wizard for an MQTT connection alias and trigger.",
        ),
        prompt(
            "setup_scheduled_task",
            "Interactive setup wizard to schedule a service for execution.",
        ),
        prompt(
            "setup_rest_api",
            "Interactive setup wizard to expose IS services as a REST API via OpenAPI.",
        ),
        prompt(
            "setup_user_management",
            "Interactive wizard to create users, groups, and configure access control.",
        ),
        prompt(
            "setup_oauth",
            "Interactive setup wizard for OAuth 2.0 client registration and scopes.",
        ),
    ]
}

/// Handle a get_prompt request by name.
pub fn get(name: &str) -> Option<GetPromptResult> {
    let text = match name {
        "setup_kafka_streaming" => KAFKA_WIZARD,
        "setup_jdbc_connection" => JDBC_WIZARD,
        "setup_sap_connection" => SAP_WIZARD,
        "setup_jms_connection" => JMS_WIZARD,
        "setup_mqtt_connection" => MQTT_WIZARD,
        "setup_scheduled_task" => SCHEDULER_WIZARD,
        "setup_rest_api" => REST_API_WIZARD,
        "setup_user_management" => USER_MGMT_WIZARD,
        "setup_oauth" => OAUTH_WIZARD,
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

Then ask if I want to create an **adapter service** for this connection. If yes:
1. Use adapter_service_template_list to show available service types (Select, Insert, CustomSQL, etc.)
2. Ask which type they want
3. For Select/Insert/Update/Delete: use adapter_resource_domain_lookup to browse the database interactively:
   a. List catalogs: resource_domain_name=\"catalogNames\"
   b. List schemas: resource_domain_name=\"schemaNames\", values=[\"catalog\"]
   c. List tables: resource_domain_name=\"tableNames\", values=[\"catalog\",\"schema\"]
   d. List columns: resource_domain_name=\"columnInfo\", values=[\"catalog\",\"schema\",\"table\"]
   Show the user the available tables and let them pick. Then show columns and let them pick.
4. For CustomSQL: ask the user for the SQL query directly
5. Create the service with adapter_service_create using the collected parameters";

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

const JMS_WIZARD: &str = "\
Guide me through setting up a JMS connection on webMethods Integration Server. \
Ask me for each of the following parameters one at a time.

**Step 1: JNDI Provider** (required for third-party JMS providers like ActiveMQ, RabbitMQ)
1. **JNDI alias name** -- identifier for the JNDI provider
2. **Initial context factory** -- Java class name (e.g., org.apache.activemq.jndi.ActiveMQInitialContextFactory)
3. **Provider URL** -- JNDI provider URL (e.g., tcp://activemq-host:61616)
4. **Security principal** / **credentials** -- if authentication is needed

IMPORTANT: The JMS provider's client JARs must be in IS classpath (WmART/code/jars/static/). \
IS must be restarted after adding JARs. Tell the user this before proceeding.

Use jndi_alias_set to create the JNDI alias, then jndi_test_lookup with lookupName=\"ConnectionFactory\" to verify.

**Step 2: JMS Connection Alias**
1. **Alias name** -- identifier for the JMS connection
2. **JNDI alias** -- reference to the JNDI alias from step 1
3. **Connection factory lookup name** -- typically \"ConnectionFactory\"
4. **Client ID** -- unique client identifier
5. **Transaction type** -- 0 (none), 1 (local), 2 (XA)

Use jms_connection_create, then jms_connection_enable.

**Step 3 (optional): JMS Trigger**
Ask if the user wants to create a trigger to consume messages from a queue/topic.";

const MQTT_WIZARD: &str = "\
Guide me through setting up an MQTT connection on webMethods Integration Server. \
Ask me for each of the following parameters one at a time.

1. **Package name** -- which IS package should own this connection?
2. **Alias name** -- identifier for the MQTT connection
3. **Broker URL** -- MQTT broker address (e.g., tcp://mosquitto-host:1883, ssl://host:8883)
4. **Client ID** -- unique MQTT client identifier
5. **Username** / **Password** -- if broker requires authentication (optional)
6. **Clean session** -- start with clean session (default: true)
7. **Keep alive interval** -- seconds between pings (default: 60)
8. **Connection timeout** -- seconds to wait for connection (default: 30)

Use mqtt_connection_create to create, mqtt_connection_enable to connect.

Then ask if the user wants to create a **trigger** to subscribe to an MQTT topic:
1. **Trigger name** -- full namespace path (e.g., mypkg.triggers:mqttHandler)
2. **Topic name** -- MQTT topic filter (supports +/# wildcards)
3. **QoS** -- 0 (at most once), 1 (at least once), 2 (exactly once)
4. **Service** -- flow service to invoke when a message arrives

Use mqtt_trigger_create to create the trigger.";

const SCHEDULER_WIZARD: &str = "\
Guide me through scheduling a service for execution on webMethods Integration Server. \
Ask me for each of the following parameters.

1. **Service to schedule** -- full service path (e.g., mypkg.services:myService). List available services with node_list if needed.
2. **Schedule type** -- one of:
   - **once** -- run once at a specific date/time
   - **repeat** -- run repeatedly at a fixed interval
   - **complex** -- cron-like schedule
3. **For once**: start date (MM/dd/yyyy) and start time (HH:mm:ss)
4. **For repeat**: interval in milliseconds (e.g., 300000 for 5 minutes), start date/time, optional end date/time
5. **Target** -- which server to run on (\"$any\" for any available server, or a specific server name)
6. **Description** -- what this scheduled task does
7. **Service inputs** -- JSON string of input parameters for the service (optional)

Use scheduler_task_add to create the task. The response includes the task OID.
Use scheduler_task_get to verify the task was created correctly.
Use scheduler_task_list to see all scheduled tasks.";

const REST_API_WIZARD: &str = "\
Guide me through exposing IS services as a REST API on webMethods Integration Server.

**Option A: Generate from existing services**
1. List available services with node_list
2. For each service to expose, verify its input/output signature with node_get
3. The IS REST descriptor framework automatically maps services to REST endpoints

**Option B: Import from OpenAPI specification**
1. **Package name** -- which IS package should contain the generated services
2. **Folder name** -- namespace folder for the generated artifacts
3. **REST API descriptor name** -- identifier for this API
4. **OpenAPI source** -- either a URL (sourceUri) or inline JSON/YAML content (openapiContent)
5. **Group by tag** -- whether to organize services by OpenAPI tags (default: false)

Use openapi_generate_provider to generate IS services from the spec, or \
openapi_generate_consumer to generate client connectors for calling an external API.

Use rest_resource_list to see all REST API descriptors, and \
openapi_doc_get to retrieve the generated OpenAPI document.";

const USER_MGMT_WIZARD: &str = "\
Guide me through setting up users and access control on webMethods Integration Server. \
Ask me what I need to do:

**Create a user:**
1. **Username** -- ASCII letters and digits only
2. **Password** -- initial password
Use user_add to create, then optionally add to groups.

**Create a group:**
1. **Group name** -- identifier for the group
Use group_add to create. Then use group_change to set members.

**View current configuration:**
- user_list -- show all users and their group memberships
- group_list -- show all groups and their members
- acl_list -- show all ACLs with allow/deny groups
- account_locking_get -- show account locking policy

**Manage access:**
- acl_add -- create a new ACL with allow/deny group lists
- user_set_disabled -- enable or disable a user account

Ask me which of these operations I need.";

const OAUTH_WIZARD: &str = "\
Guide me through setting up OAuth 2.0 on webMethods Integration Server. \
Ask me what I need:

**Register an OAuth client:**
1. **Client name** and **version**
2. **Client type** -- confidential (server-side app) or public (SPA/mobile)
3. **Grant types** -- which OAuth flows to allow:
   - client_credentials (machine-to-machine, most common for APIs)
   - authorization_code (interactive login with redirect)
   - implicit (legacy browser flow)
   - owner_credentials (direct username/password)
4. **Redirect URIs** -- required for authorization_code and implicit grants
5. **Enabled** -- whether the client is active

Use oauth_client_register. It returns the client_id and client_secret -- tell the user to save these securely.

**Create an OAuth scope:**
1. **Scope name** -- identifier (e.g., \"read\", \"admin\")
2. **Description** -- what the scope grants access to
3. **Values** -- array of IS service paths that this scope authorizes

Use oauth_scope_add.

**View current configuration:**
- oauth_settings_get -- show OAuth server settings
- oauth_client_list -- show all registered clients
- oauth_scope_list -- show all scopes
- oauth_token_list -- show active access tokens";
