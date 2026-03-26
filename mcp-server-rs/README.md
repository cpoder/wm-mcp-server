# webMethods IS MCP Server -- Technical Reference

## Installation

```bash
cargo install wm-mcp-server          # from crates.io
# or download from https://github.com/cpoder/wm-mcp-server/releases
```

## Configuration

### Single instance (environment variables)

| Variable | Default | Description |
|----------|---------|-------------|
| `WM_IS_URL` | `http://localhost:5555` | IS base URL |
| `WM_IS_USER` | `Administrator` | IS username |
| `WM_IS_PASSWORD` | `manage` | IS password |
| `WM_IS_TIMEOUT` | `30` | HTTP request timeout (seconds) |

### Multiple instances (config file)

Set `WM_CONFIG=/path/to/config.json`:

```json
{
  "instances": {
    "dev": { "url": "http://dev:5555", "user": "Administrator", "password": "manage" },
    "prod": { "url": "http://prod:5555", "user": "Administrator", "password": "secret", "timeout": 60 }
  },
  "default": "dev"
}
```

## Tools Reference (222 tools)

### Server & Instances (3)
`list_instances`, `is_status`, `is_shutdown`

### Packages & Namespace (13)
`package_list`, `package_create`, `package_reload`, `package_enable`, `package_disable`, `package_delete`, `package_info`, `package_dependencies`, `package_jar_list`, `node_list`, `node_get`, `node_delete`, `folder_create`

### Flow Services (5)
`flow_service_create`, `put_node`, `service_invoke`, `document_type_create`, `mapset_value`

### Flow Debugging (7)
`flow_debug_start`, `flow_debug_execute` (stepOver/stepIn/stepOut/resume/stop), `flow_debug_close`, `flow_debug_insert_breakpoints`, `flow_debug_remove_all_breakpoints`, `flow_debug_set_pipeline`, `flow_debug_stop_service`

### Unit Testing & Mocking (10)
`test_run`, `test_check_status`, `test_text_report`, `test_junit_report`, `mock_load`, `mock_clear`, `mock_clear_all`, `mock_list`, `mock_suspend`, `mock_resume`

### Document Type Generation (4 + 2 SAP)
`doctype_gen_from_json`, `doctype_gen_from_json_schema`, `doctype_gen_from_xsd`, `doctype_gen_from_xml`, `sap_idoc_doctype_create`, `sap_rfc_doctype_create`

### Package Marketplace (7)
`marketplace_search`, `marketplace_package_info`, `marketplace_package_tags`, `marketplace_package_git`, `marketplace_categories`, `marketplace_registries`, `marketplace_install`

### Ports / Listeners (8)
`port_list`, `port_factory_list`, `port_get`, `port_add`, `port_update`, `port_enable`, `port_disable`, `port_delete`

### URL Aliases (4)
`url_alias_list`, `url_alias_add`, `url_alias_get`, `url_alias_delete`

### Adapter Connections (7)
`adapter_type_list`, `adapter_connection_list`, `adapter_connection_create`, `adapter_connection_enable`, `adapter_connection_disable`, `adapter_connection_metadata`, `adapter_connection_state`

### Adapter Services & Metadata (8)
`adapter_service_create`, `adapter_service_get`, `adapter_service_update`, `adapter_service_template_list`, `adapter_service_template_metadata`, `adapter_resource_domain_lookup`, `adapter_listener_list`, `adapter_listener_create`, `adapter_listener_enable`, `adapter_listener_disable`

### Adapter Notifications (3)
`adapter_notification_list`, `adapter_notification_create_polling`, `adapter_notification_create_listener_based`

### Streaming / Kafka (14)
`streaming_connection_list`, `streaming_connection_create`, `streaming_connection_enable`, `streaming_connection_disable`, `streaming_connection_delete`, `streaming_connection_test`, `streaming_provider_list`, `streaming_event_source_list`, `streaming_event_source_create`, `streaming_event_source_delete`, `streaming_trigger_list`, `streaming_trigger_enable`, `streaming_trigger_disable`, `streaming_trigger_suspend`

### JNDI Providers (6)
`jndi_alias_list`, `jndi_alias_get`, `jndi_alias_set`, `jndi_alias_delete`, `jndi_test_lookup`, `jndi_template_list`

### JMS Messaging (14)
`jms_connection_list`, `jms_connection_create`, `jms_connection_update`, `jms_connection_delete`, `jms_connection_enable`, `jms_connection_disable`, `jms_trigger_report`, `jms_trigger_create`, `jms_trigger_update`, `jms_trigger_delete`, `jms_trigger_enable`, `jms_trigger_disable`, `jms_trigger_suspend`, `jms_destination_list`

### MQTT Messaging (12)
`mqtt_connection_list`, `mqtt_connection_create`, `mqtt_connection_update`, `mqtt_connection_delete`, `mqtt_connection_enable`, `mqtt_connection_disable`, `mqtt_trigger_report`, `mqtt_trigger_create`, `mqtt_trigger_delete`, `mqtt_trigger_enable`, `mqtt_trigger_disable`, `mqtt_trigger_suspend`

### Pub/Sub Triggers (9)
`trigger_report`, `trigger_create`, `trigger_delete`, `trigger_get_properties`, `trigger_set_properties`, `trigger_suspend`, `trigger_processing_status`, `trigger_retrieval_status`, `trigger_stats`

### Messaging Connections (8)
`messaging_connection_list`, `messaging_connection_create`, `messaging_connection_delete`, `messaging_connection_enable`, `messaging_connection_disable`, `messaging_publishable_doctypes`, `messaging_csq_count`, `messaging_csq_clear`

### Scheduler (10)
`scheduler_state`, `scheduler_task_list`, `scheduler_task_get`, `scheduler_task_add`, `scheduler_task_update`, `scheduler_task_cancel`, `scheduler_task_suspend`, `scheduler_task_resume`, `scheduler_pause`, `scheduler_resume`

### Users & Access (13)
`user_list`, `user_add`, `user_delete`, `user_set_disabled`, `user_disabled_list`, `group_list`, `group_add`, `group_delete`, `group_change`, `acl_list`, `acl_add`, `acl_delete`, `account_locking_get`

### JDBC Connection Pools (8)
`jdbc_pool_list`, `jdbc_pool_add`, `jdbc_pool_update`, `jdbc_pool_delete`, `jdbc_pool_test`, `jdbc_pool_restart`, `jdbc_driver_list`, `jdbc_function_list`

### Global Variables (5)
`global_var_list`, `global_var_get`, `global_var_add`, `global_var_edit`, `global_var_remove`

### Server Monitoring (10)
`server_health`, `server_stats`, `server_settings`, `server_extended_settings`, `server_service_stats`, `server_thread_dump`, `server_session_list`, `server_license_info`, `server_log`, `server_circuit_breaker_stats`

### Remote Servers (4)
`remote_server_list`, `remote_server_add`, `remote_server_delete`, `remote_server_test`

### Auditing (5)
`audit_logger_list`, `audit_logger_get`, `audit_logger_update`, `audit_logger_enable`, `audit_logger_disable`

### OAuth (9)
`oauth_settings_get`, `oauth_settings_update`, `oauth_client_list`, `oauth_client_register`, `oauth_client_delete`, `oauth_scope_list`, `oauth_scope_add`, `oauth_scope_remove`, `oauth_token_list`

### Web Services / REST / OpenAPI (8)
`ws_provider_endpoint_list`, `ws_consumer_endpoint_list`, `ws_wsdl_get`, `rest_resource_list`, `openapi_doc_get`, `openapi_generate_provider`, `openapi_generate_consumer`, `openapi_refresh_provider`

### Security & Keystore (4)
`keystore_list`, `truststore_list`, `security_settings_get`, `security_settings_update`

## Prompts Reference (9 interactive wizards)

| Prompt | What it guides you through |
|---|---|
| `setup_jdbc_connection` | JDBC connection + interactive table/column browsing + adapter service creation |
| `setup_kafka_streaming` | Kafka connection alias + event specification + trigger |
| `setup_jms_connection` | JNDI provider + JMS connection + trigger (including JAR installation guidance) |
| `setup_mqtt_connection` | MQTT connection + trigger subscription |
| `setup_sap_connection` | SAP adapter connection + optional listener |
| `setup_scheduled_task` | Schedule a service (once, repeating, or complex) |
| `setup_rest_api` | Expose services via OpenAPI or import an external API spec |
| `setup_user_management` | Create users, groups, ACLs, manage access control |
| `setup_oauth` | Register OAuth clients, create scopes, configure OAuth settings |

## Prerequisites for specific features

| Feature | Requirement |
|---|---|
| JMS (ActiveMQ, etc.) | Provider client JARs in `WmART/code/jars/static/` + IS restart |
| Kafka streaming | Kafka client JARs in `WmStreaming/code/jars/static/` (included in IS 11.x) |
| MQTT | No additional JARs needed (built into IS) |
| JDBC adapters | JDBC driver JARs in IS classpath (IS ships with DataDirect drivers) |
| SAP adapters | SAP JCo libraries installed |
| SAP doc type gen | Active SAP connection required |
| OPC UA adapters | OPC UA JARs installed |
| Marketplace install | MCP server needs filesystem access to IS packages directory |
| Unit testing | `WmUnitTestManager` package installed and enabled |
