#!/usr/bin/env python3
"""End-to-end regression suite for the 2.12.0 fixes (silent-loss protections):
put_node shell creation + step-count verification, RETRY / evaluate-labels validation,
node_get flow tree, structured service_invoke errors, idempotent folder/doc-type creation,
adapter_service_create existence check + empty-array stripping, nested lookup values,
jdbc_custom_sql_create / jdbc_batch_insert_create, server_log tail.

Runs target/release/wm-mcp-server over stdio against a live IS. Needs:
  - a JDBC connection LOCAL_TRANSACTION on the PostgreSQL database holding dwh.dim_customer /
    dwh.fact_sales (WM_E2E_CONN_DWH, default star.connections:dwh) and one on the same database
    for read-only CustomSQL (WM_E2E_CONN_LOG, default star.connections:dwhLog);
  - the Petstore connection (WM_E2E_CONN_PET, default petstoreapi.connections:postgresql) whose
    public.tag table receives and loses one row (id 990002).
The throwaway package McpFeedbackTest is created and deleted by the script.

Usage: python3 test_e2e_flow_adapters.py   (env WM_MCP_BIN / WM_IS_URL / WM_IS_USER / WM_IS_PASSWORD)
"""
import itertools, json, os, subprocess, sys

BIN = os.environ.get("WM_MCP_BIN", os.path.join(os.path.dirname(os.path.abspath(__file__)), "target/release/wm-mcp-server"))
ENV = dict(os.environ,
           WM_IS_URL=os.environ.get("WM_IS_URL", "http://localhost:5555"),
           WM_IS_USER=os.environ.get("WM_IS_USER", "Administrator"),
           WM_IS_PASSWORD=os.environ.get("WM_IS_PASSWORD", "manage"),
           WM_IS_TIMEOUT=os.environ.get("WM_IS_TIMEOUT", "600"))


class Mcp:
    def __init__(self):
        self.p = subprocess.Popen([BIN], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  stderr=open(os.devnull, "w"), env=ENV, text=True, bufsize=1)
        self.ids = itertools.count(1)
        self.req("initialize", {"protocolVersion": "2024-11-05", "capabilities": {},
                                "clientInfo": {"name": "mcpcli", "version": "0.1"}})
        self.notify("notifications/initialized", {})

    def notify(self, method, params):
        self.p.stdin.write(json.dumps({"jsonrpc": "2.0", "method": method, "params": params}) + "\n")
        self.p.stdin.flush()

    def req(self, method, params):
        i = next(self.ids)
        self.p.stdin.write(json.dumps({"jsonrpc": "2.0", "id": i, "method": method, "params": params}) + "\n")
        self.p.stdin.flush()
        while True:
            line = self.p.stdout.readline()
            if not line:
                raise RuntimeError("server closed stdout")
            try:
                msg = json.loads(line)
            except json.JSONDecodeError:
                continue
            if msg.get("id") == i:
                if "error" in msg:
                    raise RuntimeError(json.dumps(msg["error"]))
                return msg["result"]

    def call(self, tool, args):
        r = self.req("tools/call", {"name": tool, "arguments": args})
        texts = [c.get("text", "") for c in r.get("content", []) if c.get("type") == "text"]
        out = "\n".join(texts)
        if r.get("isError"):
            raise RuntimeError(out)
        return out

    def close(self):
        try:
            self.p.stdin.close()
            self.p.wait(timeout=5)
        except Exception:
            self.p.kill()




PKG = "McpFeedbackTest"
ROOT = "mcpfeedbacktest"
CONN_LOG = os.environ.get("WM_E2E_CONN_LOG", "star.connections:dwhLog")   # read-only CustomSQL
CONN_DWH = os.environ.get("WM_E2E_CONN_DWH", "star.connections:dwh")      # LOCAL_TRANSACTION (BatchInsert lookups)
m = Mcp()
results = []

def check(name, cond, detail=""):
    results.append((name, bool(cond)))
    print(("OK  " if cond else "KO  ") + name + (f"  -- {detail}" if detail else ""))

def call(tool, args):
    """returns (is_error, text)"""
    try:
        return False, m.call(tool, args)
    except RuntimeError as e:
        return True, str(e)

def sig(fields_in, fields_out):
    def rec(fields):
        return {"node_type": "record", "field_type": "record", "field_dim": "0", "nillable": "true",
                "javaclass": "com.wm.util.Values",
                "rec_fields": [{"node_type": "field", "field_name": f, "field_type": "string", "field_dim": "0", "nillable": "true"} for f in fields]}
    return {"sig_in": rec(fields_in), "sig_out": rec(fields_out)}

def service(name, nodes, fin=(), fout=("n",)):
    return {"node_nsName": f"{ROOT}:{name}", "node_pkg": PKG, "node_type": "service", "svc_type": "flow",
            "svc_subtype": "default", "svc_sigtype": "java 3.5", "stateless": "yes", "pipeline_option": 1,
            "svc_sig": sig(fin, fout), "flow": {"type": "ROOT", "version": "3.0", "cleanup": "true", "nodes": nodes}}

INNER = [
    {"type": "MAP", "mode": "STANDALONE", "nodes": [
        {"type": "MAPSET", "field": "/n;1;0", "overwrite": "true", "d_enc": "XMLValues", "mapseti18n": "true",
         "data": "<Values version=\"2.0\"><value name=\"xml\">1</value></Values>"}]},
    {"type": "BRANCH", "evaluate-labels": "true", "nodes": [
        {"type": "EXIT", "label": "%n% == 1", "from": "$loop", "signal": "SUCCESS"}]},
]

# setup: throwaway package + root folder (package_create auto-activates)
try:
    m.call("package_create", {"package_name": PKG})
except RuntimeError as e:
    print("package_create:", str(e)[:120])
print("folder_create:", call("folder_create", {"package": PKG, "folder_path": ROOT})[1][:80].replace("\n", " "))

# T1 tool inventory
tools = [t["name"] for t in m.req("tools/list", {})["tools"]]
check("T1 tool count is 345", len(tools) == 345, f"got {len(tools)}")
check("T1 new jdbc tools registered", "jdbc_custom_sql_create" in tools and "jdbc_batch_insert_create" in tools)

# T2 REPEAT rejected before writing
err, txt = call("put_node", {"node_data": json.dumps(service("e2eRepeat", [
    {"type": "REPEAT", "count": "3", "repeat-interval": "1", "repeat-on": "SUCCESS", "nodes": INNER}]))})
check("T2 REPEAT is refused with isError", err and "RETRY" in txt and "repeat-interval" in txt, txt[:160].replace("\n", " "))
err, txt = call("node_get", {"name": f"{ROOT}:e2eRepeat"})
check("T2 nothing was written for the refused node", not err and json.loads(txt).get("node") is None)

# fresh nodes every run so "created" is meaningful
for n in ("e2eRetry", "e2eDoc", "e2eBadCopy", "e2eOne", "e2eBad", "e2eSelectCustomers", "e2eJoin", "e2eInsertCustomers",
          "e2ePetTag", "e2ePetTagDel", "e2ePetDel2"):
    call("node_delete", {"name": f"{ROOT}:{n}"})

# T3 RETRY on a NEW service: shell auto-created, verified, runs
err, txt = call("put_node", {"node_data": json.dumps(service("e2eRetry", [
    {"type": "RETRY", "count": "3", "backoff": "1", "repeat-on": "SUCCESS", "nodes": INNER}]))})
d = json.loads(txt) if not err else {}
check("T3 put_node creates a new service (status created)", not err and d.get("status") == "created", txt[:200].replace("\n", " "))
v = d.get("verification", {})
check("T3 verification ok with RETRY stored", v.get("status") == "ok" and v.get("stored", {}).get("RETRY") == 1, json.dumps(v)[:200])
err, txt = call("service_invoke", {"service_path": f"{ROOT}:e2eRetry", "inputs": "{}", "timeout_secs": 60})
check("T3 RETRY + evaluate-labels runs and exits the loop", not err and json.loads(txt).get("n") == "1", txt[:120])

# T4 second write = update
err, txt = call("put_node", {"node_data": json.dumps(service("e2eRetry", [
    {"type": "RETRY", "count": "3", "backoff": "1", "repeat-on": "SUCCESS", "nodes": INNER}]))})
check("T4 second put_node reports updated", not err and json.loads(txt).get("status") == "updated", txt[:120])

# T5 document type via put_node alone
err, txt = call("put_node", {"node_data": json.dumps({
    "node_nsName": f"{ROOT}:e2eDoc", "node_pkg": PKG, "node_type": "record", "field_type": "record", "field_dim": "0",
    "nillable": "true", "rec_fields": [
        {"node_type": "field", "field_name": "id", "field_type": "string", "field_dim": "0", "nillable": "true"},
        {"node_type": "record", "field_name": "lines", "field_type": "record", "field_dim": "1", "nillable": "true", "rec_fields": [
            {"node_type": "field", "field_name": "sku", "field_type": "string", "field_dim": "0", "nillable": "true"}]}]})})
d = json.loads(txt) if not err else {}
check("T5 put_node creates a document type directly", not err and d.get("status") == "created" and d.get("verification", {}).get("fields_stored") == 3, txt[:200].replace("\n", " "))

# T6 node_get returns the real tree
err, txt = call("node_get", {"name": f"{ROOT}:e2eRetry"})
d = json.loads(txt) if not err else {}
nodes = d.get("node", {}).get("flow", {}).get("nodes")
check("T6 node_get flow.nodes is a list with the RETRY step", isinstance(nodes, list) and nodes and nodes[0].get("type") == "RETRY" and isinstance(nodes[0].get("nodes"), list), str(nodes)[:120])

# T7 structured invoke error
err, txt = call("service_invoke", {"service_path": f"{ROOT}:doesNotExist", "inputs": "{}"})
try:
    d = json.loads(txt)
except Exception:
    d = {}
check("T7 failed invoke has isError and structured JSON", err and d.get("httpStatus") == 404 and "errorType" in d and "error" in d, txt[:200].replace("\n", " "))

# T8 flow-compiler error keeps the IS message and the throwing frame
err, txt = call("put_node", {"node_data": json.dumps(service("e2eBadCopy", [
    {"type": "INVOKE", "service": "pub.string:concat", "validate-in": "$none", "validate-out": "$none", "nodes": [
        {"type": "MAP", "mode": "INPUT", "nodes": [{"type": "MAPCOPY", "from": "/x;1;0", "to": "/inString1;1;0"}]}]}]))})
check("T8 valid tree with INVOKE passes verification", not err and json.loads(txt).get("verification", {}).get("status") == "ok", txt[:160].replace("\n", " "))

# T9/T10 idempotent creates
err, txt = call("folder_create", {"package": PKG, "folder_path": ROOT})
check("T9 folder_create on an existing folder answers exists", not err and json.loads(txt).get("status") == "exists", txt[:120])
err, txt = call("document_type_create", {"package": PKG, "doc_path": f"{ROOT}:e2eDoc"})
check("T10 document_type_create on an existing doc type answers exists", not err and json.loads(txt).get("status") == "exists", txt[:120])

# T11 adapter_service_create strips empty arrays and verifies existence
settings = {"sql": "SELECT 1 AS one", "sqlFieldType": "java.lang.String", "colInfo": "0;one;INTEGER;OUT;\n",
            "inputColIndexes": [], "inputExpression": [], "inputJDBCType": [], "inputFieldType": [], "inputField": [], "realInputFields": [],
            "outputColIndexes": ["0"], "outputExpression": ["one"], "outputJDBCType": ["INTEGER"], "outputFieldType": ["java.lang.String"],
            "outputField": ["one"], "resultField": ["results[].one"], "resultFieldType": ["java.lang.String[]"], "realOutputField": ["results[].one"],
            "maxRow": "0", "queryTimeOut": "-1", "resultRowField": "", "resultRowFieldType": "", "designTimeLocale": "en",
            "userid": "overrideCredentials.$dbUser", "useridType": "java.lang.String", "inputUseridSign": "overrideCredentials.$dbUser",
            "password": "overrideCredentials.$dbPassword", "passwordType": "java.lang.String", "inputPasswordSign": "overrideCredentials.$dbPassword"}
call("node_delete", {"name": f"{ROOT}:e2eOne"})
err, txt = call("adapter_service_create", {"service_name": f"{ROOT}:e2eOne", "package_name": PKG, "connection_alias": CONN_LOG,
                                           "service_template": "com.wm.adapter.wmjdbc.services.CustomSQL", "adapter_service_settings": json.dumps(settings)})
d = json.loads(txt) if not err else {}
check("T11 empty arrays stripped and node verified", not err and d.get("verified") is True and "inputField" in d.get("omitted_empty_arrays", []), txt[:200].replace("\n", " "))
err, txt = call("service_invoke", {"service_path": f"{ROOT}:e2eOne", "inputs": "{}"})
check("T11 the service runs", not err and json.loads(txt).get("e2eOneOutput", {}).get("results", [{}])[0].get("one") == "1", txt[:160])

# T12 ART refusal is detected (wrong Java type for an array property)
bad = dict(settings); bad["realInputFields"] = "notAnArray"; bad["inputField"] = "notAnArray"
err, txt = call("adapter_service_create", {"service_name": f"{ROOT}:e2eBad", "package_name": PKG, "connection_alias": CONN_LOG,
                                           "service_template": "com.wm.adapter.wmjdbc.services.CustomSQL", "adapter_service_settings": json.dumps(bad)})
check("T12 ART refusal (wrong Java type) is an error naming the ART cause", err and "ART.117.4030" in txt and "argument type mismatch" in txt, txt[:220].replace("\n", " "))
err, txt = call("node_get", {"name": f"{ROOT}:e2eBad"})
check("T12 no node left behind", not err and json.loads(txt).get("node") is None)

# T13 nested lookup values for *-dependencies
err, txt = call("adapter_resource_domain_lookup", {"connection_alias": CONN_DWH, "service_template": "com.wm.adapter.wmjdbc.services.BatchInsert",
                                                   "resource_domain_name": "columnInfo", "values": json.dumps(["winfarm", "dwh", "dim_customer"])})
colinfo = json.loads(txt)["resourceDomainValues"][0]["values"][0]["name"] if not err else ""
check("T13 columnInfo lookup works", not err and "customer_code" in colinfo)
err, txt = call("adapter_resource_domain_lookup", {"connection_alias": CONN_DWH, "service_template": "com.wm.adapter.wmjdbc.services.BatchInsert",
                                                   "resource_domain_name": "updateJDBCTypes", "values": json.dumps([[colinfo]])})
d = json.loads(txt) if not err else {}
names = {x.get("resourceDomainName"): [v["name"] for v in x.get("values", [])] for x in d.get("resourceDomainValues", [])}
check("T13 updateJDBCTypes with [[columnInfo]] answers per-column JDBC types", not err and names.get("updateJDBCTypes") and names.get("updateColumnNames"), (txt if err else json.dumps(names))[:240].replace("\n", " "))
err, txt = call("adapter_resource_domain_lookup", {"connection_alias": CONN_DWH, "service_template": "com.wm.adapter.wmjdbc.services.BatchInsert",
                                                   "resource_domain_name": "updateJDBCTypes", "values": json.dumps([colinfo])})
check("T13 raw string dependency still fails as before (documented trap)", err and "ART.114.243" in txt, txt[:120].replace("\n", " "))

# T14 jdbc_custom_sql_create with automatic output analysis
call("node_delete", {"name": f"{ROOT}:e2eSelectCustomers"})
err, txt = call("jdbc_custom_sql_create", {"service_name": f"{ROOT}:e2eSelectCustomers", "package_name": PKG, "connection_alias": CONN_LOG,
                                           "sql": "SELECT customer_key, customer_code, customer_name FROM dwh.dim_customer WHERE customer_key <= ?",
                                           "inputs": json.dumps([{"name": "maxKey", "jdbc_type": "INTEGER"}])})
d = json.loads(txt) if not err else {}
check("T14 jdbc_custom_sql_create (outputs from customSQLcolInfo)", not err and d.get("verified") and [o["name"] for o in d.get("outputs", [])] == ["customer_key", "customer_code", "customer_name"], txt[:240].replace("\n", " "))
err, txt = call("service_invoke", {"service_path": f"{ROOT}:e2eSelectCustomers", "inputs": json.dumps({"e2eSelectCustomersInput": {"maxKey": "3"}})})
rows = json.loads(txt).get("e2eSelectCustomersOutput", {}).get("results", []) if not err else []
check("T14 the CustomSQL service runs (rows depend on the teammate's data)", not err and isinstance(rows, list) and len(rows) <= 3, txt[:160].replace("\n", " "))

# T15 join: analysis fails -> explicit outputs required
call("node_delete", {"name": f"{ROOT}:e2eJoin"})
join_sql = ("SELECT f.order_line_id, c.customer_name FROM dwh.fact_sales f JOIN dwh.dim_customer c ON c.customer_key = f.customer_key "
            "WHERE f.order_line_id <= ? ORDER BY f.order_line_id")
err, txt = call("jdbc_custom_sql_create", {"service_name": f"{ROOT}:e2eJoin", "package_name": PKG, "connection_alias": CONN_LOG,
                                           "sql": join_sql, "inputs": json.dumps([{"name": "maxId", "jdbc_type": "BIGINT"}])})
check("T15 join without outputs is refused with the -1 explanation", err and "-1" in txt and "outputs" in txt, txt[:200].replace("\n", " "))
err, txt = call("jdbc_custom_sql_create", {"service_name": f"{ROOT}:e2eJoin", "package_name": PKG, "connection_alias": CONN_LOG,
                                           "sql": join_sql, "inputs": json.dumps([{"name": "maxId", "jdbc_type": "BIGINT"}]),
                                           "outputs": json.dumps([{"name": "order_line_id", "jdbc_type": "BIGINT"}, {"name": "customer_name", "jdbc_type": "VARCHAR"}])})
check("T15 join with explicit outputs is created", not err and json.loads(txt).get("verified") is True, txt[:160].replace("\n", " "))
err, txt = call("service_invoke", {"service_path": f"{ROOT}:e2eJoin", "inputs": json.dumps({"e2eJoinInput": {"maxId": "2"}})})
check("T15 the join service runs (0..2 rows)", not err and isinstance(json.loads(txt).get("e2eJoinOutput", {}).get("results"), list), txt[:160].replace("\n", " "))

# T16 jdbc_batch_insert_create (create only, never invoked)
call("node_delete", {"name": f"{ROOT}:e2eInsertCustomers"})
err, txt = call("jdbc_batch_insert_create", {"service_name": f"{ROOT}:e2eInsertCustomers", "package_name": PKG, "connection_alias": CONN_DWH,
                                             "schema": "dwh", "table": "dim_customer", "exclude_columns": json.dumps(["customer_key"])})
d = json.loads(txt) if not err else {}
cols = [c["name"] for c in d.get("columns", [])]
check("T16 jdbc_batch_insert_create builds and verifies the node", not err and d.get("verified") and "customer_code" in cols and "customer_key" not in cols and d.get("table", "").endswith(".dwh.dim_customer") and d.get("catalog") == "<current catalog>", txt[:260].replace("\n", " "))
err, txt = call("adapter_service_get", {"service_name": f"{ROOT}:e2eInsertCustomers"})
check("T16 adapter_service_get sees BatchInsert template", not err and "BatchInsert" in txt)
err, txt = call("jdbc_batch_insert_create", {"service_name": f"{ROOT}:e2eInsertNoTable", "package_name": PKG, "connection_alias": CONN_DWH,
                                             "schema": "dwh", "table": "no_such_table"})
check("T16 unknown table is a clear error", err and "no columns found" in txt, txt[:160].replace("\n", " "))

# T18 Petstore (own demo DB): <current catalog> default really inserts; row read back and deleted
PET = os.environ.get("WM_E2E_CONN_PET", "petstoreapi.connections:postgresql")
err, txt = call("jdbc_batch_insert_create", {"service_name": f"{ROOT}:e2ePetTag", "package_name": PKG, "connection_alias": PET, "schema": "public", "table": "tag"})
d = json.loads(txt) if not err else {}
check("T18 batch insert on Petstore tag built with <current catalog>", not err and d.get("catalog") == "<current catalog>" and d.get("verified"), txt[:160].replace("\n", " "))
err, txt = call("service_invoke", {"service_path": f"{ROOT}:e2ePetTag", "inputs": json.dumps({"e2ePetTagInput": {"inputs": [{"id": "990002", "name": "mcp-e2e"}]}})})
check("T18 one row inserted (updateCount [1])", not err and json.loads(txt).get("e2ePetTagOutput", {}).get("updateCount") == ["1"], txt[:160].replace("\n", " "))
# T19 row-less statement: the -1 analysis is tolerated when inputs are typed
err, txt = call("jdbc_custom_sql_create", {"service_name": f"{ROOT}:e2ePetTagDel", "package_name": PKG, "connection_alias": PET,
                                           "sql": "DELETE FROM public.tag WHERE id = ?", "inputs": json.dumps([{"name": "id", "jdbc_type": "BIGINT"}]), "result_row_field": "rowCount"})
d = json.loads(txt) if not err else {}
check("T19 DELETE without outputs accepted despite -1 analysis", not err and d.get("verified") and "-1" in d.get("note", ""), txt[:200].replace("\n", " "))
err, txt = call("service_invoke", {"service_path": f"{ROOT}:e2ePetTagDel", "inputs": json.dumps({"e2ePetTagDelInput": {"id": "990002"}})})
check("T19 the DELETE removed the test row (rowCount 1)", not err and json.loads(txt).get("e2ePetTagDelOutput", {}).get("rowCount") == "1", txt[:160].replace("\n", " "))
err, txt = call("jdbc_custom_sql_create", {"service_name": f"{ROOT}:e2ePetDel2", "package_name": PKG, "connection_alias": PET,
                                           "sql": "SELECT id, name FROM public.tag WHERE id = ?", "inputs": json.dumps([{"name": "id", "jdbc_type": "BIGINT"}])})
check("T19 SELECT with failed analysis still asks for outputs", err and "-1" in txt and "outputs" in txt, txt[:160].replace("\n", " "))

# T17 server_log tail
err, txt = call("server_log", {"num_lines": "5"})
d = json.loads(txt) if not err else {}
check("T17 server_log returns the last lines", not err and len(d.get("lines", [])) == 5, txt[:160].replace("\n", " "))

print("cleanup:", call("package_delete", {"package_name": PKG})[1][:60].replace("\n", " "))
m.close()
failed = [n for n, ok in results if not ok]
print(f"\n{len(results) - len(failed)}/{len(results)} checks passed" + (f"; FAILED: {failed}" if failed else ""))
sys.exit(1 if failed else 0)
