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
