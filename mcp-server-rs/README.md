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

## Tools Reference (167 tools)

### Server & Instances (3)
| Tool | Description |
|------|-------------|
| `list_instances` | List configured IS instances and which is default |
| `is_status` | Check if IS is running |
| `is_shutdown` | Shutdown or restart IS (bounce=true for restart) |

### Packages (5)
| Tool | Description |
|------|-------------|
| `package_list` | List all packages |
| `package_create` | Create and activate a package |
| `package_reload` | Reload a package |
| `package_enable` / `package_disable` | Enable or disable a package |

### Namespace (4)
| Tool | Description |
|------|-------------|
| `node_list` | List services, folders, documents in a package |
| `node_get` | Get full definition of any node |
| `node_delete` | Delete a node |
| `folder_create` | Create a namespace folder |

### Flow Services (4)
| Tool | Description |
|------|-------------|
| `flow_service_create` | Create empty flow service shell |
| `put_node` | **Core API** -- create/update flow service with full logic and signature |
| `service_invoke` | Execute/test any service |
| `document_type_create` | Create a document type |
| `mapset_value` | Helper: encode value for MAPSET data field |

### Ports / Listeners (8)
| Tool | Description |
|------|-------------|
| `port_list` / `port_factory_list` | List ports and available types |
| `port_get` / `port_add` / `port_update` | Get, create, update ports |
| `port_enable` / `port_disable` / `port_delete` | Manage port lifecycle |

### Adapter Connections (7)
| Tool | Description |
|------|-------------|
| `adapter_type_list` | List adapter types (JDBC, SAP, OPC, ...) |
| `adapter_connection_list` / `adapter_connection_create` | List/create connections |
| `adapter_connection_enable` / `adapter_connection_disable` | Toggle connection state |
| `adapter_connection_metadata` | Get connection factory parameters |
| `adapter_connection_state` | Query connection state |

### Adapter Services & Metadata (8)
| Tool | Description |
|------|-------------|
| `adapter_service_create` | Create an adapter service (Select, Insert, CustomSQL, ...) |
| `adapter_service_get` / `adapter_service_update` | Get/update adapter service config |
| `adapter_service_template_list` | List available service templates for a connection |
| `adapter_service_template_metadata` | Get full parameter metadata for a template |
| `adapter_resource_domain_lookup` | **Browse database objects** -- catalogs, schemas, tables, columns from a live connection |
| `adapter_listener_list` / `adapter_listener_create` | List/create adapter listeners |
| `adapter_listener_enable` / `adapter_listener_disable` | Toggle listener state |

### Adapter Notifications (3)
| Tool | Description |
|------|-------------|
| `adapter_notification_list` | List polling notifications |
| `adapter_notification_create_polling` | Create polling notification (JDBC insert/update/delete detection) |
| `adapter_notification_create_listener_based` | Create listener-based notification (SAP IDoc, OPC events) |

### Streaming / Kafka (15)
| Tool | Description |
|------|-------------|
| `streaming_connection_list` / `streaming_connection_create` | List/create Kafka connection aliases |
| `streaming_connection_enable` / `streaming_connection_disable` / `streaming_connection_delete` | Manage lifecycle |
| `streaming_connection_test` | Test connectivity to Kafka broker |
| `streaming_provider_list` | List available streaming providers |
| `streaming_event_source_list` / `streaming_event_source_create` / `streaming_event_source_delete` | Manage topic mappings |
| `streaming_trigger_list` / `streaming_trigger_enable` / `streaming_trigger_disable` / `streaming_trigger_suspend` | Manage triggers |

### JNDI Providers (6)
| Tool | Description |
|------|-------------|
| `jndi_alias_list` / `jndi_alias_get` / `jndi_alias_set` / `jndi_alias_delete` | CRUD for JNDI provider aliases |
| `jndi_test_lookup` | Test a JNDI lookup (e.g., ConnectionFactory) |
| `jndi_template_list` | List available JNDI provider templates |

### JMS Messaging (14)
| Tool | Description |
|------|-------------|
| `jms_connection_list` / `jms_connection_create` / `jms_connection_update` / `jms_connection_delete` | Connection CRUD |
| `jms_connection_enable` / `jms_connection_disable` | Toggle connection |
| `jms_trigger_report` / `jms_trigger_create` / `jms_trigger_update` / `jms_trigger_delete` | Trigger CRUD |
| `jms_trigger_enable` / `jms_trigger_disable` / `jms_trigger_suspend` | Trigger lifecycle |
| `jms_destination_list` | List queues/topics on a connection |

### MQTT Messaging (12)
| Tool | Description |
|------|-------------|
| `mqtt_connection_list` / `mqtt_connection_create` / `mqtt_connection_update` / `mqtt_connection_delete` | Connection CRUD |
| `mqtt_connection_enable` / `mqtt_connection_disable` | Toggle connection |
| `mqtt_trigger_report` / `mqtt_trigger_create` / `mqtt_trigger_delete` | Trigger CRUD |
| `mqtt_trigger_enable` / `mqtt_trigger_disable` / `mqtt_trigger_suspend` | Trigger lifecycle |

### Scheduler (10)
| Tool | Description |
|------|-------------|
| `scheduler_state` | Get scheduler state (running/paused) |
| `scheduler_task_list` / `scheduler_task_get` | List/get tasks |
| `scheduler_task_add` / `scheduler_task_update` / `scheduler_task_cancel` | Task CRUD |
| `scheduler_task_suspend` / `scheduler_task_resume` | Pause/resume individual tasks |
| `scheduler_pause` / `scheduler_resume` | Pause/resume entire scheduler |

### Users & Access (14)
| Tool | Description |
|------|-------------|
| `user_list` / `user_add` / `user_delete` | User CRUD |
| `user_set_disabled` / `user_disabled_list` | Enable/disable users |
| `group_list` / `group_add` / `group_delete` / `group_change` | Group CRUD + membership |
| `acl_list` / `acl_add` / `acl_delete` | ACL management |
| `account_locking_get` | Account locking policy |

### JDBC Connection Pools (8)
| Tool | Description |
|------|-------------|
| `jdbc_pool_list` / `jdbc_pool_add` / `jdbc_pool_update` / `jdbc_pool_delete` | Pool CRUD |
| `jdbc_pool_test` / `jdbc_pool_restart` | Test and restart pools |
| `jdbc_driver_list` / `jdbc_function_list` | List drivers and functional aliases |

### Global Variables (5)
| Tool | Description |
|------|-------------|
| `global_var_list` / `global_var_get` | List/get variables |
| `global_var_add` / `global_var_edit` / `global_var_remove` | CRUD |

### Server Monitoring (11)
| Tool | Description |
|------|-------------|
| `server_health` | Adapter connections, triggers, messaging status |
| `server_stats` | Uptime, memory, thread counts |
| `server_settings` / `server_extended_settings` | Server and watt.* properties |
| `server_service_stats` | Service execution statistics |
| `server_thread_dump` | JVM thread dump for diagnostics |
| `server_session_list` | Active HTTP sessions |
| `server_license_info` | Licensed features |
| `server_log` | Server log (full or last N lines) |
| `server_circuit_breaker_stats` | Circuit breaker state |

### Remote Servers (4)
| Tool | Description |
|------|-------------|
| `remote_server_list` / `remote_server_add` / `remote_server_delete` | CRUD |
| `remote_server_test` | Test connectivity |

### Auditing (5)
| Tool | Description |
|------|-------------|
| `audit_logger_list` / `audit_logger_get` / `audit_logger_update` | Logger CRUD |
| `audit_logger_enable` / `audit_logger_disable` | Toggle loggers |

### OAuth (9)
| Tool | Description |
|------|-------------|
| `oauth_settings_get` / `oauth_settings_update` | OAuth server settings |
| `oauth_client_list` / `oauth_client_register` / `oauth_client_delete` | Client CRUD |
| `oauth_scope_list` / `oauth_scope_add` / `oauth_scope_remove` | Scope CRUD |
| `oauth_token_list` | Active access tokens |

### Web Services / REST / OpenAPI (8)
| Tool | Description |
|------|-------------|
| `ws_provider_endpoint_list` / `ws_consumer_endpoint_list` | List SOAP endpoints |
| `ws_wsdl_get` | Get WSDL document |
| `rest_resource_list` | List REST API descriptors |
| `openapi_doc_get` | Get OpenAPI document for a REST descriptor |
| `openapi_generate_provider` / `openapi_generate_consumer` | Generate IS services from OpenAPI spec |
| `openapi_refresh_provider` | Refresh REST descriptor from source |

### Security & Keystore (4)
| Tool | Description |
|------|-------------|
| `keystore_list` / `truststore_list` | List keystores and truststores |
| `security_settings_get` / `security_settings_update` | SSL/TLS settings |

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
| OPC UA adapters | OPC UA JARs installed |
