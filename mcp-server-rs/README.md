# webMethods IS MCP Server -- Technical Reference

## Installation

```bash
npx wm-mcp-server                    # run via npx (no install needed)
npm install -g wm-mcp-server         # install globally via npm
cargo install wm-mcp-server          # from crates.io (requires Rust)
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

## Tools Reference (336 tools)

### Server & Instances (3)
`list_instances`, `is_status`, `is_shutdown`

### Packages & Namespace (21)
`package_list`, `package_create`, `package_reload`, `package_enable`, `package_disable`, `package_delete`, `package_info`, `package_dependencies`, `package_jar_list`, `package_settings`, `package_compile`, `package_add_depend`, `package_del_depend`, `package_add_startup_service`, `package_remove_startup_service`, `package_jar_delete`, `node_list`, `node_get`, `node_delete`, `folder_create`, `install_jars`

### Flow Services (5)
`flow_service_create`, `put_node`, `service_invoke`, `document_type_create`, `mapset_value`

### Flow Debugging (7)
`flow_debug_start`, `flow_debug_execute` (stepOver/stepIn/stepOut/resume/stop), `flow_debug_close`, `flow_debug_insert_breakpoints`, `flow_debug_remove_all_breakpoints`, `flow_debug_set_pipeline`, `flow_debug_stop_service`

### Unit Testing & Mocking (10)
`test_run`, `test_check_status`, `test_text_report`, `test_junit_report`, `mock_load`, `mock_clear`, `mock_clear_all`, `mock_list`, `mock_suspend`, `mock_resume`

### Document Type Generation (5 + 2 SAP)
`doctype_gen_from_json`, `doctype_gen_from_json_schema`, `doctype_gen_from_xsd`, `doctype_gen_from_xml`, `doctype_gen_from_dtd`, `sap_idoc_doctype_create`, `sap_rfc_doctype_create`

### Namespace Dependencies (6)
`ns_dep_get_dependents`, `ns_dep_get_references`, `ns_dep_get_unresolved`, `ns_dep_search`, `ns_dep_refactor_preview`, `ns_dep_refactor`

### Flat File Schemas (4)
`flatfile_schema_save`, `flatfile_dictionary_create`, `flatfile_schema_get`, `flatfile_schema_delete`

### Package Marketplace (7)
`marketplace_search`, `marketplace_package_info`, `marketplace_package_tags`, `marketplace_package_git`, `marketplace_categories`, `marketplace_registries`, `marketplace_install`

### Ports / Listeners (8)
`port_list`, `port_factory_list`, `port_get`, `port_add`, `port_update`, `port_enable`, `port_disable`, `port_delete`

### URL Aliases (5)
`url_alias_list`, `url_alias_add`, `url_alias_get`, `url_alias_delete`, `url_alias_update`

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

### Messaging Connections & Publish (11)
`messaging_connection_list`, `messaging_connection_create`, `messaging_connection_delete`, `messaging_connection_enable`, `messaging_connection_disable`, `messaging_publishable_doctypes`, `messaging_csq_count`, `messaging_csq_clear`, `messaging_publish`, `messaging_publish_and_wait`, `messaging_deliver`

### Scheduler (10)
`scheduler_state`, `scheduler_task_list`, `scheduler_task_get`, `scheduler_task_add`, `scheduler_task_update`, `scheduler_task_cancel`, `scheduler_task_suspend`, `scheduler_task_resume`, `scheduler_pause`, `scheduler_resume`

### Users & Access (21)
`user_list`, `user_add`, `user_delete`, `user_set_disabled`, `user_disabled_list`, `group_list`, `group_add`, `group_delete`, `group_change`, `acl_list`, `acl_add`, `acl_delete`, `acl_assign`, `acl_get_nodes`, `acl_get_default_access`, `acl_set_default_access`, `account_locking_get`, `account_locking_update`, `account_locking_reset`, `account_locked_list`, `account_unlock`

### JDBC Connection Pools (8)
`jdbc_pool_list`, `jdbc_pool_add`, `jdbc_pool_update`, `jdbc_pool_delete`, `jdbc_pool_test`, `jdbc_pool_restart`, `jdbc_driver_list`, `jdbc_function_list`

### Global Variables (5)
`global_var_list`, `global_var_get`, `global_var_add`, `global_var_edit`, `global_var_remove`

### Server Monitoring & Admin (15)
`server_health`, `server_stats`, `server_settings`, `server_extended_settings`, `server_service_stats`, `server_thread_dump`, `server_session_list`, `server_license_info`, `server_log`, `server_circuit_breaker_stats`, `server_thread_interrupt`, `server_thread_kill`, `server_session_kill`, `server_ssl_cache_clear`, `ip_access_change_type`

### Remote Servers (4)
`remote_server_list`, `remote_server_add`, `remote_server_delete`, `remote_server_test`

### Auditing (5)
`audit_logger_list`, `audit_logger_get`, `audit_logger_update`, `audit_logger_enable`, `audit_logger_disable`

### OAuth (9)
`oauth_settings_get`, `oauth_settings_update`, `oauth_client_list`, `oauth_client_register`, `oauth_client_delete`, `oauth_scope_list`, `oauth_scope_add`, `oauth_scope_remove`, `oauth_token_list`

### Web Services / REST / OpenAPI (13)
`ws_provider_endpoint_list`, `ws_consumer_endpoint_list`, `ws_wsdl_get`, `ws_consumer_endpoint_add`, `ws_provider_endpoint_add`, `ws_consumer_endpoint_delete`, `ws_provider_endpoint_delete`, `ws_connector_refresh`, `rest_resource_list`, `openapi_doc_get`, `openapi_generate_provider`, `openapi_generate_consumer`, `openapi_refresh_provider`

### SFTP (9)
`sftp_server_list`, `sftp_server_get`, `sftp_server_create`, `sftp_server_delete`, `sftp_test_connection`, `sftp_user_list`, `sftp_user_get`, `sftp_user_create`, `sftp_user_delete`

### HTTP Proxy (6)
`proxy_list`, `proxy_get`, `proxy_create`, `proxy_delete`, `proxy_enable`, `proxy_disable`

### JWT (6)
`jwt_issuer_list`, `jwt_issuer_get`, `jwt_issuer_add`, `jwt_issuer_delete`, `jwt_settings_get`, `jwt_settings_update`

### Quiesce Mode (3)
`quiesce_status`, `quiesce_enable`, `quiesce_disable`

### Health Indicators (3)
`health_indicators_list`, `health_indicator_get`, `health_indicator_change`

### Alerts (3)
`alert_status`, `alert_enable`, `alert_disable`

### IP Access Control (4)
`ip_access_list`, `ip_access_add`, `ip_access_delete`, `ip_access_change_type`

### Password Policy (2)
`password_policy_get`, `password_policy_update`

### Security & Keystore (4)
`keystore_list`, `truststore_list`, `security_settings_get`, `security_settings_update`

### Cache Manager (6)
`cache_manager_list`, `cache_manager_get`, `cache_manager_create`, `cache_manager_update`, `cache_manager_delete`, `cache_reset`

### SAML (3)
`saml_issuer_list`, `saml_issuer_add`, `saml_issuer_delete`

### LDAP (4)
`ldap_settings_get`, `ldap_server_add`, `ldap_server_edit`, `ldap_server_delete`

### Logger Configuration (5)
`logger_list`, `logger_get`, `logger_update`, `logger_server_config_get`, `logger_server_config_update`

### Outbound Passwords (3)
`outbound_password_store`, `outbound_password_retrieve`, `outbound_password_remove`

### Port Access Control (6)
`port_access_list`, `port_access_get`, `port_access_add_nodes`, `port_access_delete_node`, `port_access_set_type`, `port_access_reset`

### Enterprise Gateway (7)
`egw_rules_list`, `egw_dos_get`, `egw_dos_update`, `egw_denied_ip_list`, `egw_rule_add`, `egw_rule_delete`, `egw_rule_update`

### WebSocket (4)
`websocket_sessions_by_port`, `websocket_close_session`, `websocket_endpoint_create`, `websocket_broadcast`

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
