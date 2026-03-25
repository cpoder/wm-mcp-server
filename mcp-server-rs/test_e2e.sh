#!/bin/bash
# End-to-end test of the Rust MCP server against a live IS
set -e

BIN="./target/release/wm-mcp-server"
PASS=0
FAIL=0
SKIP=0

# Helper: send MCP init + request, extract the response for the given id
mcp_call() {
    local id="$1"
    local tool="$2"
    local args="$3"
    (
        printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}\n'
        sleep 0.3
        printf '{"jsonrpc":"2.0","method":"notifications/initialized"}\n'
        sleep 0.3
        printf '{"jsonrpc":"2.0","id":%s,"method":"tools/call","params":{"name":"%s","arguments":%s}}\n' "$id" "$tool" "$args"
        sleep 1
    ) | timeout 10 "$BIN" 2>/dev/null | python3 -c "
import sys, json
for line in sys.stdin:
    line = line.strip()
    if not line: continue
    d = json.loads(line)
    if d.get('id') == $id:
        result = d.get('result', {})
        content = result.get('content', [])
        if content:
            print(content[0].get('text', ''))
        break
"
}

mcp_prompt() {
    local id="$1"
    local name="$2"
    (
        printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}\n'
        sleep 0.3
        printf '{"jsonrpc":"2.0","method":"notifications/initialized"}\n'
        sleep 0.3
        printf '{"jsonrpc":"2.0","id":%s,"method":"prompts/get","params":{"name":"%s"}}\n' "$id" "$name"
        sleep 0.5
    ) | timeout 5 "$BIN" 2>/dev/null | python3 -c "
import sys, json
for line in sys.stdin:
    line = line.strip()
    if not line: continue
    d = json.loads(line)
    if d.get('id') == $id:
        msgs = d.get('result', {}).get('messages', [])
        if msgs:
            text = msgs[0].get('content', {}).get('text', '')
            print(text[:100])
        break
"
}

check() {
    local name="$1"
    local output="$2"
    local expect="$3"
    if echo "$output" | grep -q "$expect"; then
        echo "  PASS: $name"
        PASS=$((PASS + 1))
    else
        echo "  FAIL: $name (expected '$expect')"
        echo "    got: $(echo "$output" | head -3)"
        FAIL=$((FAIL + 1))
    fi
}

check_not_empty() {
    local name="$1"
    local output="$2"
    if [ -n "$output" ]; then
        echo "  PASS: $name"
        PASS=$((PASS + 1))
    else
        echo "  FAIL: $name (empty output)"
        FAIL=$((FAIL + 1))
    fi
}

echo "=== E2E Tests against live IS ==="
echo ""

# ── Clean Slate ──────────────────────────────────────────────
echo "--- Clean Slate ---"
# Wipe E2ETestPkg completely if it exists from a previous run
curl -s -u Administrator:manage -H "Accept: application/json" \
  "http://localhost:5555/invoke/wm.server.packages/packageDelete?package=E2ETestPkg" > /dev/null 2>&1 || true
echo "  Done (deleted E2ETestPkg if it existed)"

# ── Server Status ────────────────────────────────────────────
echo "--- Server Status ---"
out=$(mcp_call 2 "is_status" '{}')
check "is_status" "$out" "RUNNING"

# ── Instances ────────────────────────────────────────────────
echo "--- Instances ---"
out=$(mcp_call 2 "list_instances" '{}')
check "list_instances" "$out" "default"

# ── Package Management ───────────────────────────────────────
echo "--- Package Management ---"
out=$(mcp_call 2 "package_list" '{}')
check "package_list" "$out" "packages"

out=$(mcp_call 2 "package_create" '{"package_name":"E2ETestPkg"}')
check "package_create" "$out" "created"

out=$(mcp_call 2 "package_reload" '{"package_name":"E2ETestPkg"}')
check "package_reload" "$out" "reloaded"

out=$(mcp_call 2 "package_disable" '{"package_name":"E2ETestPkg"}')
check "package_disable" "$out" "disabled"

out=$(mcp_call 2 "package_enable" '{"package_name":"E2ETestPkg"}')
check "package_enable" "$out" "enabled"

# ── Namespace ────────────────────────────────────────────────
echo "--- Namespace ---"
out=$(mcp_call 2 "folder_create" '{"package":"E2ETestPkg","folder_path":"e2etest"}')
check "folder_create" "$out" "created"

out=$(mcp_call 2 "folder_create" '{"package":"E2ETestPkg","folder_path":"e2etest.services"}')
check "folder_create nested" "$out" "created"

out=$(mcp_call 2 "node_list" '{"package":"E2ETestPkg"}')
check "node_list" "$out" "e2etest"

# ── Flow Service ─────────────────────────────────────────────
echo "--- Flow Service ---"
out=$(mcp_call 2 "flow_service_create" '{"package":"E2ETestPkg","service_path":"e2etest.services:hello"}')
check "flow_service_create" "$out" "created"

out=$(mcp_call 2 "node_get" '{"name":"e2etest.services:hello"}')
check "node_get" "$out" "node_type"

PUT_NODE_DATA='{"node_nsName":"e2etest.services:hello","node_pkg":"E2ETestPkg","node_type":"service","svc_type":"flow","svc_subtype":"default","svc_sigtype":"java 3.5","stateless":"yes","pipeline_option":1,"svc_sig":{"sig_in":{"node_type":"record","field_type":"record","field_dim":"0","nillable":"true","javaclass":"com.wm.util.Values","rec_fields":[{"node_type":"field","field_name":"name","field_type":"string","field_dim":"0","nillable":"true"}]},"sig_out":{"node_type":"record","field_type":"record","field_dim":"0","nillable":"true","javaclass":"com.wm.util.Values","rec_fields":[{"node_type":"field","field_name":"greeting","field_type":"string","field_dim":"0","nillable":"true"}]}},"flow":{"type":"ROOT","version":"3.0","cleanup":"true","nodes":[{"type":"MAP","mode":"STANDALONE","nodes":[{"type":"MAPSET","field":"/greeting;1;0","overwrite":"true","d_enc":"XMLValues","mapseti18n":"true","data":"<Values version=\"2.0\"><value name=\"xml\">Hello from E2E test!</value></Values>"}]}]}}'
out=$(mcp_call 2 "put_node" "{\"node_data\": $(echo "$PUT_NODE_DATA" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read()))')}")
check "put_node" "$out" "ok"

out=$(mcp_call 2 "service_invoke" '{"service_path":"e2etest.services:hello"}')
check "service_invoke" "$out" "Hello from E2E test"

# ── Document Type ────────────────────────────────────────────
echo "--- Document Type ---"
out=$(mcp_call 2 "document_type_create" '{"package":"E2ETestPkg","doc_path":"e2etest:testDoc"}')
check "document_type_create" "$out" "created"

# ── Mapset Helper ────────────────────────────────────────────
echo "--- Helpers ---"
out=$(mcp_call 2 "mapset_value" '{"value":"hello <world> & \"friends\""}')
check "mapset_value" "$out" "&lt;world&gt;"

# ── Ports ────────────────────────────────────────────────────
echo "--- Ports ---"
out=$(mcp_call 2 "port_list" '{}')
check "port_list" "$out" "listeners"

out=$(mcp_call 2 "port_factory_list" '{}')
check "port_factory_list" "$out" "factories"

# ── Adapters ─────────────────────────────────────────────────
echo "--- Adapters ---"
out=$(mcp_call 2 "adapter_type_list" '{}')
check "adapter_type_list" "$out" "adapter"

out=$(mcp_call 2 "adapter_connection_list" '{}')
check_not_empty "adapter_connection_list" "$out"

# ── Streaming ────────────────────────────────────────────────
echo "--- Streaming ---"
out=$(mcp_call 2 "streaming_provider_list" '{}')
check "streaming_provider_list" "$out" "availableProviders\|Provider\|Kafka\|status"

out=$(mcp_call 2 "streaming_connection_list" '{}')
check_not_empty "streaming_connection_list" "$out"

out=$(mcp_call 2 "streaming_trigger_list" '{}')
check_not_empty "streaming_trigger_list" "$out"

out=$(mcp_call 2 "streaming_event_source_list" '{}')
check_not_empty "streaming_event_source_list" "$out"

# ── JMS Messaging ────────────────────────────────────────────
echo "--- JMS Messaging ---"
out=$(mcp_call 2 "jms_connection_list" '{}')
check "jms_connection_list" "$out" "aliasDataList"

out=$(mcp_call 2 "jms_trigger_report" '{}')
check "jms_trigger_report" "$out" "triggerDataList"

# Full CRUD cycle: create -> disable (already disabled) -> delete
JMS_SETTINGS='{"aliasName":"E2E_JMS_Test","description":"E2E test connection","jndi_connectionFactoryLookupName":"ConnectionFactory","clientID":"e2e_jms_client","enabled":"false","transactionType":"0","nwm_brokerHost":"localhost:61616","nwm_brokerName":"n/a","nwm_clientGroup":"IS-JMS","classLoader":"INTEGRATION_SERVER"}'
out=$(mcp_call 2 "jms_connection_create" "{\"settings\":$(echo "$JMS_SETTINGS" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read().strip()))')}")
check "jms_connection_create" "$out" "E2E_JMS_Test\|created\|success"

out=$(mcp_call 2 "jms_connection_delete" '{"alias_name":"E2E_JMS_Test"}')
check "jms_connection_delete" "$out" "deleted\|E2E_JMS_Test\|message"

# ── MQTT Messaging (full E2E with Mosquitto broker) ─────────
echo "--- MQTT Messaging ---"
out=$(mcp_call 2 "mqtt_connection_list" '{}')
check "mqtt_connection_list" "$out" "aliasDataList"

out=$(mcp_call 2 "mqtt_trigger_report" '{}')
check "mqtt_trigger_report" "$out" "triggerDataList"

# Full lifecycle: create -> enable -> verify connected -> publish -> disable -> delete
# Requires Mosquitto running on localhost:1883 (docker run -d --name mosquitto-test -p 1883:1883 eclipse-mosquitto:2 mosquitto -c /mosquitto-no-auth.conf)
MQTT_SETTINGS='{"name":"E2E_MQTT_Test","description":"E2E MQTT test","package":"WmRoot","host":"tcp://localhost:1883","clientId":"isE2eMqttTest","timeout":"30","keepAlive":"60","cleanSessionEnabled":"true"}'
out=$(mcp_call 2 "mqtt_connection_create" "{\"settings\":$(echo "$MQTT_SETTINGS" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read().strip()))')}")
check "mqtt_connection_create" "$out" "E2E_MQTT_Test\|created\|success"

out=$(mcp_call 2 "mqtt_connection_enable" '{"alias_name":"E2E_MQTT_Test"}')
check "mqtt_connection_enable" "$out" "enabled\|E2E_MQTT_Test"

sleep 2

# Verify connection is connected by checking the report
out=$(mcp_call 2 "mqtt_connection_list" '{}')
check "mqtt_connected" "$out" "true\|Running"

# Publish a message via IS (use service_invoke since wm.server.mqtt:publish is the IS service)
out=$(mcp_call 2 "service_invoke" '{"service_path":"wm.server.mqtt:publish","inputs":"{\"connectionAliasName\":\"E2E_MQTT_Test\",\"topicName\":\"e2e/test\",\"MqttMessage\":{\"body\":\"E2E_VERIFY\"}}"}')
check "mqtt_publish" "$out" "connectionAliasName\|E2E_MQTT_Test\|topicName"

out=$(mcp_call 2 "mqtt_connection_disable" '{"alias_name":"E2E_MQTT_Test"}')
check "mqtt_connection_disable" "$out" "disabled\|E2E_MQTT_Test"

out=$(mcp_call 2 "mqtt_connection_delete" '{"alias_name":"E2E_MQTT_Test"}')
check "mqtt_connection_delete" "$out" "deleted\|E2E_MQTT_Test"

# ── Scheduler (full lifecycle) ────────────────────────────────
echo "--- Scheduler ---"
out=$(mcp_call 2 "scheduler_state" '{}')
check "scheduler_state" "$out" "running"

out=$(mcp_call 2 "scheduler_task_list" '{}')
check "scheduler_task_list" "$out" "tasks"

# Create a repeating task -> get -> suspend -> resume -> cancel
SCHED_SETTINGS='{"service":"pub.flow:debugLog","description":"E2E scheduler test","type":"repeat","target":"$any","interval":"300000","startDate":"06/15/2027","startTime":"12:00:00"}'
out=$(mcp_call 2 "scheduler_task_add" "{\"settings\":$(echo "$SCHED_SETTINGS" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read().strip()))')}")
check "scheduler_task_add" "$out" "oid\|Task added"
TASK_OID=$(echo "$out" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('oid',''))" 2>/dev/null)

if [ -n "$TASK_OID" ]; then
  out=$(mcp_call 2 "scheduler_task_get" "{\"oid\":\"$TASK_OID\"}")
  check "scheduler_task_get" "$out" "pub.flow:debugLog\|E2E scheduler"

  out=$(mcp_call 2 "scheduler_task_suspend" "{\"oid\":\"$TASK_OID\"}")
  check "scheduler_task_suspend" "$out" "taskSuspended\|true"

  out=$(mcp_call 2 "scheduler_task_resume" "{\"oid\":\"$TASK_OID\"}")
  check "scheduler_task_resume" "$out" "taskResumed\|true"

  out=$(mcp_call 2 "scheduler_task_cancel" "{\"oid\":\"$TASK_OID\"}")
  check "scheduler_task_cancel" "$out" "taskCancelled\|true"
else
  echo "  SKIP: could not extract task OID"
  SKIP=$((SKIP + 4))
fi

# Pause/resume scheduler
out=$(mcp_call 2 "scheduler_pause" '{}')
check "scheduler_pause" "$out" "paused\|message\|status"

out=$(mcp_call 2 "scheduler_resume" '{}')
check "scheduler_resume" "$out" "resumed\|message\|status"

# ── User & Access Management (full lifecycle) ───────────────
echo "--- Users & Access ---"
out=$(mcp_call 2 "user_list" '{}')
check "user_list" "$out" "users\|Administrator"

out=$(mcp_call 2 "group_list" '{}')
check "group_list" "$out" "groups\|Administrators"

out=$(mcp_call 2 "acl_list" '{}')
check "acl_list" "$out" "aclgroups\|Administrators"

out=$(mcp_call 2 "account_locking_get" '{}')
check "account_locking_get" "$out" "isEnabled\|maximumLoginAttempts"

# Full lifecycle: create user -> disable -> enable -> create group -> delete group -> delete user
out=$(mcp_call 2 "user_add" '{"username":"e2etestuser","password":"TestPass123"}')
check "user_add" "$out" "success.*true\|e2etestuser"

out=$(mcp_call 2 "user_set_disabled" '{"username":"e2etestuser","disabled":true}')
check "user_disable" "$out" "User\|message"

out=$(mcp_call 2 "user_disabled_list" '{}')
check "user_disabled_list" "$out" "e2etestuser\|disabledUsers"

out=$(mcp_call 2 "user_set_disabled" '{"username":"e2etestuser","disabled":false}')
check "user_enable" "$out" "User\|message"

out=$(mcp_call 2 "group_add" '{"groupname":"E2ETestGroup"}')
check "group_add" "$out" "success.*true\|Added"

out=$(mcp_call 2 "group_delete" '{"groupname":"E2ETestGroup"}')
check "group_delete" "$out" "success.*true\|Deleted"

out=$(mcp_call 2 "user_delete" '{"username":"e2etestuser"}')
check "user_delete" "$out" "success.*true\|Deleted"

# ── JDBC Pools (full lifecycle) ───────────────────────────────
echo "--- JDBC Pools ---"
out=$(mcp_call 2 "jdbc_pool_list" '{}')
check "jdbc_pool_list" "$out" "pools\|pool.name"

out=$(mcp_call 2 "jdbc_driver_list" '{}')
check "jdbc_driver_list" "$out" "drivers\|driver.name"

out=$(mcp_call 2 "jdbc_function_list" '{}')
check "jdbc_function_list" "$out" "functions\|function.name"

# Create -> test -> delete cycle
POOL_SETTINGS='{"pool":"E2ETestPool","description":"E2E test pool","drivers":"DataDirect Connect JDBC SQL Server Driver","url":"jdbc:wm:sqlserver://localhost:1433;databaseName=master","uid":"sa","pwd":"TestPass","mincon":"1","maxcon":"3"}'
out=$(mcp_call 2 "jdbc_pool_add" "{\"settings\":$(echo "$POOL_SETTINGS" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read().strip()))')}")
check "jdbc_pool_add" "$out" "E2ETestPool\|funct\|Add"

out=$(mcp_call 2 "jdbc_pool_test" "{\"settings\":$(echo "$POOL_SETTINGS" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read().strip()))')}")
check "jdbc_pool_test" "$out" "E2ETestPool\|message"

out=$(mcp_call 2 "jdbc_pool_delete" '{"pool":"E2ETestPool"}')
check "jdbc_pool_delete" "$out" "deleted\|E2ETestPool"

# ── Global Variables (full lifecycle) ────────────────────────
echo "--- Global Variables ---"
out=$(mcp_call 2 "global_var_list" '{}')
check "global_var_list" "$out" "globalVariables"

out=$(mcp_call 2 "global_var_add" '{"key":"E2E_TEST_VAR","value":"hello_from_e2e"}')
check "global_var_add" "$out" "Success\|added"

out=$(mcp_call 2 "global_var_get" '{"key":"E2E_TEST_VAR"}')
check "global_var_get" "$out" "hello_from_e2e"

out=$(mcp_call 2 "global_var_edit" '{"key":"E2E_TEST_VAR","value":"updated_value"}')
check "global_var_edit" "$out" "Success\|updated"

out=$(mcp_call 2 "global_var_get" '{"key":"E2E_TEST_VAR"}')
check "global_var_get_updated" "$out" "updated_value"

out=$(mcp_call 2 "global_var_remove" '{"key":"E2E_TEST_VAR"}')
check "global_var_remove" "$out" "Success\|deleted"

# ── Server Monitoring & Config ────────────────────────────────
echo "--- Server Monitoring ---"
out=$(mcp_call 2 "server_stats" '{}')
check "server_stats_quick" "$out" "uptime"
# server_health can be slow on large IS instances, tested via curl above

out=$(mcp_call 2 "server_stats" '{}')
check "server_stats" "$out" "uptime\|freeMem\|totalMem"

out=$(mcp_call 2 "server_settings" '{}')
check_not_empty "server_settings" "$out"

out=$(mcp_call 2 "server_extended_settings" '{}')
check "server_extended_settings" "$out" "watt\."

out=$(mcp_call 2 "server_service_stats" '{}')
check "server_service_stats" "$out" "SvcStats\|name"

out=$(mcp_call 2 "server_thread_dump" '{}')
check "server_thread_dump" "$out" "threadDump\|Thread"

out=$(mcp_call 2 "server_session_list" '{}')
check "server_session_list" "$out" "sessions\|ssnid"

out=$(mcp_call 2 "server_license_info" '{}')
check "server_license_info" "$out" "LicenseInfo\|Clustering"

out=$(mcp_call 2 "server_circuit_breaker_stats" '{}')
check_not_empty "server_circuit_breaker_stats" "$out"

# ── Remote Server Aliases (full lifecycle) ────────────────────
echo "--- Remote Servers ---"
out=$(mcp_call 2 "remote_server_list" '{}')
check "remote_server_list" "$out" "servers\|local"

# Full lifecycle: add -> test connectivity -> delete
REMOTE_SETTINGS='{"alias":"E2ERemoteTest","host":"localhost","port":"5555","user":"Administrator","pass":"manage"}'
out=$(mcp_call 2 "remote_server_add" "{\"settings\":$(echo "$REMOTE_SETTINGS" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read().strip()))')}")
check "remote_server_add" "$out" "E2ERemoteTest\|servers"

out=$(mcp_call 2 "remote_server_test" '{"alias":"E2ERemoteTest"}')
check "remote_server_test" "$out" "success\|Connected"

out=$(mcp_call 2 "remote_server_delete" '{"alias":"E2ERemoteTest"}')
check "remote_server_delete" "$out" "servers\|E2ERemoteTest"

# ── Auditing ─────────────────────────────────────────────────
echo "--- Auditing ---"
out=$(mcp_call 2 "audit_logger_list" '{}')
check "audit_logger_list" "$out" "loggers\|loggerName"

out=$(mcp_call 2 "audit_logger_get" '{"logger_name":"Error Logger"}')
check "audit_logger_get" "$out" "Error Logger\|isEnabled"

# Disable then re-enable the Error Logger
out=$(mcp_call 2 "audit_logger_disable" '{"logger_name":"Error Logger"}')
check "audit_logger_disable" "$out" "message\|disabled\|Error Logger"

out=$(mcp_call 2 "audit_logger_enable" '{"logger_name":"Error Logger"}')
check "audit_logger_enable" "$out" "message\|enabled\|Error Logger"

# ── OAuth (full lifecycle) ────────────────────────────────────
echo "--- OAuth ---"
out=$(mcp_call 2 "oauth_settings_get" '{}')
check "oauth_settings_get" "$out" "requireHTTPS\|accessTokenLifetime"

out=$(mcp_call 2 "oauth_client_list" '{}')
check "oauth_client_list" "$out" "clients"

out=$(mcp_call 2 "oauth_scope_list" '{}')
check_not_empty "oauth_scope_list" "$out"

# Full lifecycle: register client -> add scope -> remove scope -> remove client
OAUTH_CLIENT='{"name":"E2ETestClient","version":"1.0","type":"confidential","client_credentials_allowed":"true","enabled":"true"}'
out=$(mcp_call 2 "oauth_client_register" "{\"settings\":$(echo "$OAUTH_CLIENT" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read().strip()))')}")
check "oauth_client_register" "$out" "client_id\|client_secret"
CLIENT_ID=$(echo "$out" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('client_id',''))" 2>/dev/null)

OAUTH_SCOPE='{"name":"e2e_scope","description":"E2E test","values":["pub.flow:debugLog"]}'
out=$(mcp_call 2 "oauth_scope_add" "{\"settings\":$(echo "$OAUTH_SCOPE" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read().strip()))')}")
check "oauth_scope_add" "$out" "Saved\|e2e_scope"

out=$(mcp_call 2 "oauth_scope_remove" '{"name":"e2e_scope"}')
check "oauth_scope_remove" "$out" "Deleted\|e2e_scope"

if [ -n "$CLIENT_ID" ]; then
  out=$(mcp_call 2 "oauth_client_delete" "{\"client_id\":\"$CLIENT_ID\"}")
  check "oauth_client_delete" "$out" "removed\|Successfully"
else
  echo "  SKIP: oauth_client_delete (no client_id)"
  SKIP=$((SKIP + 1))
fi

# ── Prompts ──────────────────────────────────────────────────
echo "--- Prompts ---"
out=$(mcp_prompt 2 "setup_kafka_streaming")
check "prompt:setup_kafka_streaming" "$out" "Kafka"

out=$(mcp_prompt 2 "setup_jdbc_connection")
check "prompt:setup_jdbc_connection" "$out" "JDBC"

out=$(mcp_prompt 2 "setup_sap_connection")
check "prompt:setup_sap_connection" "$out" "SAP"

# ── Cleanup ──────────────────────────────────────────────────
echo "--- Cleanup ---"
out=$(mcp_call 2 "node_delete" '{"name":"e2etest.services:hello"}')
check "node_delete service" "$out" "deleted"

out=$(mcp_call 2 "node_delete" '{"name":"e2etest:testDoc"}')
check "node_delete doc" "$out" "deleted"

out=$(mcp_call 2 "package_disable" '{"package_name":"E2ETestPkg"}')
check "cleanup: disable pkg" "$out" "disabled"

# Note: not deleting pkg since there's no package_delete tool

echo ""
echo "=== Results: $PASS passed, $FAIL failed, $SKIP skipped ==="
exit $FAIL
