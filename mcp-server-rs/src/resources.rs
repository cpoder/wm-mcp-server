//! MCP resource definitions -- embedded reference documentation for flow service development.

use rmcp::model::*;

pub struct DocResource {
    pub uri: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub content: &'static str,
}

pub const RESOURCES: &[DocResource] = &[
    DocResource {
        uri: "wm://docs/flow-language-reference",
        name: "Flow Language Reference",
        description: "Complete reference for webMethods flow service development via putNode API: how put_node creates and verifies nodes, step types and their exact keys (RETRY, evaluate-labels), WmPath format, mapping rules, LOOP patterns, and working examples.",
        content: FLOW_LANGUAGE_REF,
    },
    DocResource {
        uri: "wm://docs/putnode-examples",
        name: "putNode Working Examples",
        description: "Tested, working putNode JSON examples for common flow service patterns: simple service, INVOKE with mappings, LOOP over records, BRANCH, record array handling.",
        content: PUTNODE_EXAMPLES,
    },
    DocResource {
        uri: "wm://docs/adapter-connection-reference",
        name: "Adapter Connection Configuration Reference",
        description: "How to create an adapter connection (JDBC and others) with adapter_connection_create: discovering the real adapter type name and the connectionSettings systemNames, a verified PostgreSQL/SQL Server/Oracle example, the connection-alias namespace rule, enabling the node, and an error-to-cause table.",
        content: ADAPTER_CONNECTION_REF,
    },
    DocResource {
        uri: "wm://docs/adapter-service-reference",
        name: "Adapter Service Configuration Reference",
        description: "How to create JDBC adapter services: jdbc_custom_sql_create / jdbc_batch_insert_create, verified Select / Insert / CustomSQL / BatchInsert adapter_service_settings JSON, the colInfo and columnInfo formats, array-valued resource-domain lookups ([[...]]), why the ART refuses nodes silently (empty arrays, numbers) and how the tools detect it, transactions.",
        content: ADAPTER_SERVICE_REF,
    },
    DocResource {
        uri: "wm://docs/flow-steps-reference",
        name: "Flow Steps Reference (IBM Docs)",
        description: "Official IBM documentation for all webMethods flow step types: INVOKE, BRANCH, LOOP, MAP, SEQUENCE, REPEAT (putNode type RETRY), EXIT. Properties, behavior rules, failure conditions, and data mapping concepts.",
        content: FLOW_STEPS_REF,
    },
    DocResource {
        uri: "wm://docs/builtin-services",
        name: "Built-In Services Reference",
        description: "Input/output signatures for commonly used IS built-in services: pub.string (concat, replace, substring, etc.), pub.math (addInts, multiplyFloats, etc.), pub.list (appendToDocumentList, etc.), pub.date (formatDate, etc.), pub.flow (debugLog, getLastError, etc.).",
        content: BUILTIN_SERVICES_REF,
    },
    DocResource {
        uri: "wm://docs/onprem-provisioning",
        name: "On-Prem Provisioning, Database & JDBC Setup",
        description: "How to install add-on products (Trading Networks, EDIINT/AS2, EDI) with the IBM Installer in CLI mode, apply fixes with Update Manager (SUM), create database components with the Database Configurator (DCC) for PostgreSQL, and wire JDBC pools + functional aliases. Captures non-obvious gotchas: the PTY-required installer password prompt, the updateFunctionalAlias isolationlevel requirement, and the IS-restart-required rule for Trading Networks.",
        content: ONPREM_PROVISIONING_REF,
    },
    DocResource {
        uri: "wm://docs/fsl-language-reference",
        name: "FSL (Flow Service Language) Reference",
        description: "Text-based DSL syntax for authoring flow services with dsl_validate/fsl_deploy/fsl_extract: when FSL is safe and when to use put_node instead (document lists, WHILE, pub.date are compiled wrong on IS 12.1), interface/service/properties structure, copy vs set, semicolon placement rules, EXIT/TRY-CATCH, and a complete worked example.",
        content: FSL_LANGUAGE_REF,
    },
    DocResource {
        uri: "wm://docs/unit-test-reference",
        name: "Unit Test Framework Reference",
        description: "How IS unit tests work today (Unit Test Framework, ex-WmTestSuite: Designer plugin + WmUnitTestManager package) and how to author, run and read them with test_suite_create / test_suite_list / test_run / test_*_report / mock_*: suite JSON spec with examples, generated file layout and XML, comparison semantics, mocks, environment prerequisites and gotchas.",
        content: UNIT_TEST_REF,
    },
];

const ONPREM_PROVISIONING_REF: &str = r#"# On-Prem Provisioning, Database & JDBC Configuration

Operational runbook for standing up Integration Server features that are NOT
exposed as MCP tools (they run via the platform's own CLIs). The gotchas below
are easy to miss and cost hours.

## 1. Install add-on products (Trading Networks, EDIINT/AS2, EDI)

- Use the IBM webMethods **Installer** (e.g. `IBM_webM_Install_Linux_x64.bin`),
  NOT Update Manager. Update Manager only applies FIXES to already-installed
  products; the Installer adds NEW products.
- Run it in console mode: `installer.bin -console` (launches the SAG DistMan
  installer `com.wm.distman.install.DistManInstallMain`; the bin forwards args).
- CRITICAL: the credential prompt uses a no-echo reader that needs a REAL TTY.
  A plain pipe/FIFO yields an empty password ("Enter a user name and password"
  loop). Drive it under a pseudo-TTY:
  `script -qefc 'installer.bin -console' typescript.log` and feed stdin via a
  FIFO kept open by a background writer (so the reader never sees EOF).
- Auth: `empowerUser` = your IBM/Empower account (often already stored in
  `<sumdir>/UpdateManager/conf/preferences` as `empowerUser=...`); password =
  the entitlement key (an IBM Marketplace JWT).
- Choose the EXISTING install dir; pick "Install packages on existing instance".
- Selecting EDIINT auto-selects "Trading Networks Server" (dependency).
- Selecting the EDI module auto-selects ALL standard schema libraries (~6 GB).
  Toggle OFF the "Schemas" node to keep only EDI Program Files (~260 MB total).
- "Use sudo? N" for headless (daemon registration is optional if IS runs as the
  install user). Finish with `F`.
- Result packages: WmTN, WmEDIINT, WmEDIforTN, WmEDI.

## 2. Apply fixes (Update Manager / SUM) -- do this BEFORE the DB step

Fixes can update the DB component scripts, so patch before running DCC.

- `<sumdir>/bin/UpdateManagerCMD.sh -installDir <dir> -empowerUser <u> -empowerPass <entitlementKey>`
- Cheap validation of token + network without installing:
  `-action viewAvailableFixes -installDir <dir> -empowerUser <u> -empowerPass <key>`.
- Interactive install path: Install and Uninstall Fixes -> Install fixes from
  Passport Advantage Online (PAO) -> "Create script? N" -> select "All fixes"
  (item 1) -> continue past the shutdown warning. IS/MWS/UM must be DOWN.
- Drive it under a PTY too (same `script` technique) if a no-echo prompt appears,
  though passing `-empowerPass` as an arg avoids the password prompt.
- A "Post Install Messages" note about fixes for products you don't have (e.g.
  Optimize InfrastructureDC) is safe to ignore.

## 3. Create database components (Database Configurator / DCC)

- Tool: `<install>/common/db/bin/dbConfigurator.sh`.
  Discovery flags: `-pd` (DB types), `-pp` (products), `-pc` (components),
  `-pa` (actions).
- PostgreSQL is supported (`-d postgresql`). webMethods BUNDLES its own
  DataDirect driver at `<install>/common/lib/ext/dd-cjdbc.jar` (already on the
  DCC and IS classpath) -- do NOT add an external PostgreSQL driver.
- Use the DataDirect/webMethods JDBC URL format, NOT the open-source one:
    jdbc:wm:postgresql://<host>:<port>;databaseName=<db>
  (the open-source `jdbc:postgresql://host:port/db` is only for ad-hoc tooling.)
- The database itself must already exist; DCC creates the component TABLES in it,
  not the database. Create a dedicated DB (e.g. `wm12`) so it does not collide
  with an older install's schema.
- Create per product at the latest level:
    dbConfigurator.sh -a create -d postgresql -pr TN  -v latest \
      -l "jdbc:wm:postgresql://localhost:5432;databaseName=wm12" \
      -u <user> -p <pwd> -au <admin> -ap <adminPwd>
  Products: `TN` (Trading Networks), `IS` (Integration Server),
  `MWS` (My webMethods Server). Look for "status : complete".

## 4. Wire the database to IS (JDBC pool + functional aliases)

- Create a pool (MCP `jdbc_pool_add`):
  drivers = "DataDirect Connect JDBC PostgreSQL Driver",
  url = "jdbc:wm:postgresql://host:5432;databaseName=<db>".
  Validate first with `jdbc_pool_test`.
- Map functional aliases to the pool via the service
  `wm.server.jdbcpool:updateFunctionalAlias` (no dedicated MCP tool).
  CRITICAL GOTCHA: passing only {function, pool} is SILENTLY IGNORED -- the
  association does not persist. You MUST also pass `isolationlevel`.
  Working call:
    service_invoke wm.server.jdbcpool:updateFunctionalAlias
      {"function":"TN","pool":"<pool>","isolationlevel":"-1"}
  Verify with `wm.server.jdbcpool:getFunctionalAlias {"function":"TN"}` ->
  `function.pool` must be set. The on-disk proof is
  `config/jdbc/function/<Alias>.xml`.
- For Trading Networks map the `TN` alias; also map `ISCoreAudit`,
  `DocumentHistory`, and `Xref` (used by TN/EDI) to the same pool.

## 5. Restart IS after configuring the TN alias -- a reload is NOT enough

Trading Networks reads its pool only at package init. If IS first started
without the alias, TN failed and left `JobMgrFactory.registry` / the ehcache
`CacheManager` null. A `package_reload WmTN` then keeps failing with
"Trading Networks database pool is not configured". Only a FULL Integration
Server restart inits TN cleanly against the DB. (A harmless UM
"Realm is currently not reachable" error at startup is fine -- AS2/TN use HTTP,
not Universal Messaging.)

## Known issue: TN datastore init fails on PostgreSQL ("could not retrieve data")

Symptom: WmTN loads cleanly (hundreds of services, 0 load errors) but at startup TN
logs `DatastoreException: Trading Networks could not retrieve data from your database`
then `NullPointerException ... ehcache CacheManager ... is null`, and TN is disabled.
The thrown `com.wm.util.BasisSQLException` has an EMPTY message.

Diagnosis (traced with DataDirect spy logging -- set `spyenabled=true` in
config/jdbc/pool/<pool>.xml, restart, read logs/spy/<pool>.log): TN's
`Datastore.getDBMetaData()` opens the connection and the JDBC metadata calls all
SUCCEED (`getMetaData`/`getDatabaseProductName` -> "PostgreSQL", driver 6.0.0, DB
16.14), but the method throws the empty SQLException BEFORE issuing any SQL -- there
is NO prepareStatement/executeQuery in the spy trace and NO error in the PostgreSQL
server log. The throw originates inside TN's `SQLStatements.getSql("version.select")`.

Ruled out (don't waste time re-checking):
- DB schema: DCC `-pr TN/IS/MWS` completes; 203 tables incl. bizdoc, is_datastore.
- Connectivity: `jdbc_pool_test` and the admin "Test" both succeed.
- PostgreSQL version: fails identically on PG16 and PG17 (the DataDirect "This driver
  is locked for use with embedded applications" message only appears for
  EXTERNAL/standalone use of the OEM driver -- expected, NOT the IS-side cause).
- The TNModelVersion row: `version.select` reads it, but inserting it
  (`INSERT INTO TNModelVersion(MajorVersion,MinorVersion) VALUES (85,0)`) does NOT fix
  startup -- TN throws UPSTREAM of that query. DCC never populates this row (no SQL
  script inserts it; TN writes it itself via `Datastore.setVersion`, only during a
  config import).
- Jar conflicts: the two tncore.jar (install-level + instance-level) are
  byte-identical and both define the `version.select` key.
- Package load: WmTN itself loads with 0 errors.

Separately, WmEDIforTN and WmEDI may fail to load ("circular dependencies" / "system
package depends on non-system package") when older EDI/EDIINT module versions are
added to a 12.1 install -- a different problem affecting EDI-over-TN, not the core
TN datastore.

Status: unresolved/environment-specific. Next steps: open an IBM support case with the
spy trace, or point the TN functional alias at the embedded (Derby) datastore to
isolate whether the defect is DataDirect/PostgreSQL-specific.
"#;

pub fn list() -> Vec<Resource> {
    RESOURCES
        .iter()
        .map(|r| {
            let raw = RawResource::new(r.uri, r.name)
                .with_description(r.description)
                .with_mime_type("text/markdown");
            Annotated::new(raw, None)
        })
        .collect()
}

pub fn read(uri: &str) -> Option<ReadResourceResult> {
    RESOURCES
        .iter()
        .find(|r| r.uri == uri)
        .map(|r| ReadResourceResult::new(vec![ResourceContents::text(r.content, r.uri)]))
}

// ═══════════════════════════════════════════════════════════════════════
// Embedded documentation
// ═══════════════════════════════════════════════════════════════════════

const FLOW_LANGUAGE_REF: &str = r#"# Flow Language Reference for putNode API

## Node naming (CRITICAL -- common 500 cause)

A service is named by its **folder path**: `folder.subfolder:serviceName`. This is NOT the package.
NEVER prefix `node_nsName` with the package name -- the package is passed separately in `node_pkg`.
Folders and packages are different concepts: a node lives *in* a package but is *named* by its folder path.

- Correct:     `node_nsName = "mypackage.orders.api:create"`, `node_pkg = "MyPackage"`
- WRONG (500): `node_nsName = "MyPackage.orders.api:create"` (or `"MyPackage:orders.api:create"`)

Why the second form fails: every segment of the path is a **folder that must already exist**.
`MyPackage` (PascalCase) is a package, not a folder, so putNode reports
`[ISS.0081.9001] Node MyPackage.orders.api:create does not exist`. The first form works because
`mypackage` is a real folder you created with `folder_create` -- see the namespace layout below.

The same applies to `folder_create`, `document_type_create` and `node_delete`: the path never contains the package name.

## How put_node writes a node (IS 12.1, `watt.server.ns.lockingMode=full`)

`wm.server.ns:putNode` LOCKS the node before writing, so a node that does not
exist yet fails with `[ISS.0081.9001] Node x:y does not exist` at
`nsimpl.lockNode`. The `put_node` tool handles this: when the node is absent
it first creates the shell (`serviceAdd` for a flow service, `makeNode` for a
document type or any other `node_type`), then writes the full definition. One
`put_node` call is therefore enough; `flow_service_create` and
`document_type_create` are only needed to reserve a name. Parent FOLDERS are
not created for document types -- `folder_create` them first (a flow service
shell does create its folder, but do not rely on it: keep the layout below).

After writing, `put_node` reads the node back (as XML -- the only encoding in
which IS returns the step tree) and compares the number of steps per type
with what was sent; the answer carries `"verification": {"status": "ok",
"sent": {...}, "stored": {...}}`. A deficit is an error: IS drops any step
whose `type` or keys it does not know without a message (see RETRY and
`evaluate-labels` below), and this check is what catches it. `verify: false`
skips the readback.

`node_get` uses the same XML readback, so `flow.nodes` is the real nested
tree. (IS's own JSON encoding renders it as the string `"[INVOKE]"`; that is
what `fsl_deploy`'s `node` field and raw `getNode` calls show.)

Nodes written this way stay locked by the MCP user (Designer shows a lock
icon); that is the normal state of a node being edited under `full` locking.

## Namespace layout (create the folders before anything else)

The IS namespace is **shared by all packages** -- a folder path identifies a node globally, and
the package only says who owns it. So a package must not plant generic folders at the top level:
two packages both defining `api` or `util` end up interleaved in the same namespace.

Convention: **one root folder per package, the package name in lowercase, everything nested under it.**

```
package PetstoreAPI          package WxEdiAddon
  petstoreapi                  wx.edi.addon
    petstoreapi.api              wx.edi.addon.inbound
    petstoreapi.adapter          wx.edi.addon.outbound
    petstoreapi.connections      wx.edi.addon.connections
```

- Right: `folder_create("PetstoreAPI", "petstoreapi")` then `folder_create("PetstoreAPI", "petstoreapi.api")`
- Wrong: `folder_create("PetstoreAPI", "api")` -- top-level generic folder, collides across packages

A multi-word package name is split into dotted lowercase segments (`WxEdiAddon` -> `wx.edi.addon`),
which also groups every package sharing a prefix under one visible tree.

`folder_create` does NOT create intermediate folders: call it once per level, parents first.

## WmPath Format

All field references in flow services use WmPath format: `/fieldName;type;dim[;docTypeRef][/nestedPath]`

### Type values
- `1` = String
- `2` = Record (anonymous document)
- `3` = Object (Java object)
- `4` = RecordRef (typed document reference -- CRITICAL for LOOP mappings)

### Dimension values
- `0` = scalar
- `1` = array
- `2` = table (2D array)

### Examples
- `/myString;1;0` -- scalar string
- `/myList;1;1` -- string array
- `/myDoc;2;0` -- anonymous record (scalar)
- `/myDocs;2;1` -- anonymous record array
- `/accounts;4;1;mypackage.doctypes:account` -- typed record array (RecordRef to doc type)
- `/accounts;4;0;mypackage.doctypes:account/customerName;1;0` -- field inside current LOOP iteration element

### Indexed access into a list (`results[0]`)

A record list or string list element is addressed with the index right after
the field name, BEFORE the `;type;dim` suffix, and the dim stays the list's:

```
/selectPetOutput;2;0/results[0];2;1/name;1;0     -> name of the first row
/ICValues;2;0/UNB;2;0/UNG[0];2;1/UNG06;1;0        -> Designer-generated example
```

`results;2;0` (dim 0) does NOT mean "first element": it reads the list as if
it were a single record and silently yields nothing. Typical JDBC pattern:
INVOKE the Select, `pub.list:sizeOfList` on `results`, BRANCH on `size`
(`0` -> EXIT $parent FAILURE into the CATCH), else MAPCOPY from `results[0]`.

## Flow Step Types

### INVOKE
Call another service with optional input/output pipeline mappings.
```json
{
  "type": "INVOKE",
  "service": "pub.string:concat",
  "validate-in": "$none",
  "validate-out": "$none",
  "nodes": [
    {"type": "MAP", "mode": "INPUT", "nodes": [/* MAPCOPY/MAPSET */]},
    {"type": "MAP", "mode": "OUTPUT", "nodes": [/* MAPCOPY/MAPSET */]}
  ]
}
```

### MAP
Manipulate pipeline variables. Three modes:
- `STANDALONE` -- independent mapping step
- `INPUT` -- maps pipeline to service input (inside INVOKE nodes array)
- `OUTPUT` -- maps service output back to pipeline (inside INVOKE nodes array)

### MAPCOPY
Copy value between pipeline fields.
```json
{"type": "MAPCOPY", "from": "/sourceField;1;0", "to": "/targetField;1;0"}
```

### MAPSET
Set a constant value.
```json
{
  "type": "MAPSET",
  "field": "/fieldName;1;0",
  "overwrite": "true",
  "d_enc": "XMLValues",
  "mapseti18n": "true",
  "data": "<Values version=\"2.0\"><value name=\"xml\">theValue</value></Values>"
}
```

#### MAPSET for record arrays (CRITICAL)
Use type 4 (RecordRef) and `<array>` element in data:
```json
{
  "type": "MAPSET",
  "field": "/accounts;4;1;mypackage.doctypes:account",
  "overwrite": "true",
  "d_enc": "XMLValues",
  "mapseti18n": "true",
  "data": "<Values version=\"2.0\"><array name=\"xml\" type=\"record\" depth=\"1\"><record javaclass=\"com.wm.util.Values\"><value name=\"field1\">val1</value></record><record javaclass=\"com.wm.util.Values\"><value name=\"field1\">val2</value></record></array></Values>"
}
```

### MAPDELETE
Remove a field from the pipeline.
```json
{"type": "MAPDELETE", "field": "/fieldName;1;0"}
```

### LOOP
Iterate over an array. **CRITICAL: use type 4 RecordRef paths for field access.**

```json
{
  "type": "LOOP",
  "in-array": "/inputArray",
  "out-array": "/outputArray",
  "nodes": [/* child steps executed per iteration */]
}
```

Inside the LOOP body, access the current iteration element's fields using:
`/arrayName;4;0;pkg.folder:docType/fieldName;1;0`

Where:
- `4` = RecordRef type (MUST be 4, not 2)
- `0` = scalar dimension (current iteration element, not the array)
- `pkg.folder:docType` = the document type reference for the record
- `/fieldName;1;0` = the nested field to access

### BRANCH
Conditional execution based on a field value.
```json
{
  "type": "BRANCH",
  "switch": "/fieldName",
  "nodes": [
    {"type": "SEQUENCE", "label": "value1", "nodes": [/* steps */]},
    {"type": "SEQUENCE", "label": "$null", "nodes": [/* null handling */]},
    {"type": "SEQUENCE", "label": "$default", "nodes": [/* default steps */]}
  ]
}
```
Special labels: `$null` (value is null), `$default` (fallback), blank (empty string match).
Without a `$null` case a null switch value falls into `$default` (verified IS 12.1).

#### BRANCH with label expressions (expression-based branching)
The key is **`evaluate-labels`** (flow.xml `LABELEXPRESSIONS="true"`). The
spelling `label-expressions` is silently ignored: the BRANCH is then written
without switch or expressions and fails at run time with `[ISC.0049.9009]
Missing required property switch at 'unlabeled BRANCH'`. No `switch` key is
needed in expression mode.
```json
{
  "type": "BRANCH",
  "evaluate-labels": "true",
  "nodes": [
    {"type": "SEQUENCE", "label": "code = 200", "nodes": [/* success */]},
    {"type": "SEQUENCE", "label": "code = 400", "nodes": [/* bad request */]},
    {"type": "SEQUENCE", "label": "name != null && status != null", "nodes": [/* both set */]},
    {"type": "SEQUENCE", "label": "$default", "nodes": [/* fallback */]}
  ]
}
```
Expressions support: `=`, `!=`, `null`, `$null`, `&&` (use `&amp;&amp;` in XML), `||`, regex (`/^pattern/`).

#### BRANCH with expression-guarded EXIT (conditional loop break)
```json
{
  "type": "BRANCH", "evaluate-labels": "true",
  "nodes": [
    {"type": "EXIT", "label": "%count% >= %limit%", "from": "$loop", "signal": "SUCCESS"}
  ]
}
```

### REPEAT (putNode type `RETRY`)
Retry/polling step -- shown as REPEAT in Designer, but its putNode/Values
type is **`RETRY`** (`com.wm.lang.flow.FlowRetry`, flow.xml `<RETRY COUNT=
BACK-OFF= LOOP-ON=>`). A node typed `"REPEAT"` is unknown to the flow
compiler and is dropped WITH ITS WHOLE SUBTREE, silently (HTTP 200, no log
line); put_node now refuses it before writing.
```json
{
  "type": "RETRY",
  "count": "3",
  "backoff": "5",
  "repeat-on": "FAILURE",
  "nodes": [/* steps to retry */]
}
```
- `repeat-on`: `FAILURE` (retry when a child fails) or `SUCCESS` (repeat while
  children succeed -- polling / batch loop, leave with `EXIT from="$loop"`)
- `count`: max re-executions (`-1` = unlimited). Supports `%variable%`.
- `backoff`: seconds between iterations (Designer's "Repeat interval").
  `repeat-interval` and `back-off` are NOT keys -- ignored silently.
- `timeout`: optional, seconds (generic step key)
- `$retries` holds the current iteration inside the loop.

### SEQUENCE
Group steps with a label and exit condition.
```json
{"type": "SEQUENCE", "label": "myLabel", "exit-on": "FAILURE", "nodes": [/* steps */]}
```

Exit-on values: `FAILURE` (stop on first failure, default), `SUCCESS` (stop on first success), `DONE` (run all regardless).

Optional `scope` attribute restricts pipeline visibility:
```json
{"type": "SEQUENCE", "scope": "$myScope", "exit-on": "FAILURE", "nodes": [/* steps */]}
```

#### Try-alternatives pattern (SEQUENCE EXIT-ON="SUCCESS")
Older alternative to FORM="TRY"/"CATCH". The outer SEQUENCE exits on first success — if the try block succeeds, skip catch:
```json
{
  "type": "SEQUENCE", "exit-on": "SUCCESS",
  "nodes": [
    {"type": "SEQUENCE", "label": "try", "exit-on": "FAILURE", "nodes": [/* business logic */]},
    {"type": "INVOKE", "label": "catch", "service": "pub.flow:getLastError"}
  ]
}
```

### TRY/CATCH (CRITICAL for production services)

TRY/CATCH is implemented using SEQUENCE elements with a `form` attribute. The TRY and CATCH are **sibling** elements (NOT nested).

```json
{
  "type": "SEQUENCE", "exit-on": "FAILURE", "form": "TRY",
  "nodes": [
    {"type": "SEQUENCE", "exit-on": "FAILURE", "comment": "business logic",
     "nodes": [
       {"type": "INVOKE", "service": "my.svc:doWork"},
       {"type": "BRANCH", "switch": "/statusCode", "nodes": [
         {"type": "SEQUENCE", "label": "200", "exit-on": "FAILURE", "nodes": []},
         {"type": "SEQUENCE", "label": "$default", "exit-on": "FAILURE", "nodes": [
           {"type": "MAP", "mode": "STANDALONE", "nodes": [
             {"type": "MAPSET", "field": "/error/status;1;0", "overwrite": "true",
              "d_enc": "XMLValues", "mapseti18n": "true",
              "data": "<Values version=\"2.0\"><value name=\"xml\">500</value></Values>"}
           ]},
           {"type": "EXIT", "from": "$parent", "signal": "FAILURE", "failure-message": "Operation failed"}
         ]}
       ]}
     ]
    }
  ]
},
{
  "type": "SEQUENCE", "exit-on": "FAILURE", "form": "CATCH",
  "nodes": [
    {"type": "BRANCH", "switch": "/error/status", "nodes": [
      {"type": "SEQUENCE", "label": "$null", "exit-on": "FAILURE", "nodes": [
        {"type": "MAP", "mode": "STANDALONE", "nodes": [
          {"type": "MAPSET", "field": "/error/status;1;0", "overwrite": "true",
           "d_enc": "XMLValues", "mapseti18n": "true",
           "data": "<Values version=\"2.0\"><value name=\"xml\">500</value></Values>"},
          {"type": "MAPSET", "field": "/error/message;1;0", "overwrite": "true",
           "d_enc": "XMLValues", "mapseti18n": "true",
           "data": "<Values version=\"2.0\"><value name=\"xml\">Internal error</value></Values>"}
        ]},
        {"type": "INVOKE", "service": "pub.flow:getLastFailureCaught"},
        {"type": "INVOKE", "service": "pub.flow:debugLog"}
      ]}
    ]},
    {"type": "INVOKE", "service": "pub.flow:setHTTPResponse"}
  ]
}
```

**Rules:**
1. `form` attribute: `"TRY"` on the try wrapper, `"CATCH"` on the catch handler
2. Both MUST have `exit-on: "FAILURE"`
3. TRY and CATCH are **adjacent siblings** at the same level (both children of FLOW root or same parent)
4. Inside TRY: use `EXIT from="$parent" signal="FAILURE"` to trigger the catch
5. Inside CATCH: call `pub.flow:getLastFailureCaught` to get failure details (returns `failureMessage`, `failureName`, `failure`)
6. For transactions: call `pub.art.transaction:rollbackTransaction` in CATCH block

### EXIT
Exit from the current flow, loop, or sequence.
```json
{"type": "EXIT", "from": "$flow", "signal": "FAILURE", "failure-message": "Error message"}
```

`from` values:
- `$flow` -- exit the entire flow service
- `$parent` -- exit the parent SEQUENCE (used to trigger CATCH in TRY/CATCH)
- `$loop` -- exit the nearest LOOP
- `$iteration` -- exit the current LOOP iteration only

`signal` values: `FAILURE` (triggers catch/error), `SUCCESS` (clean exit)

Inside a `SEQUENCE form=TRY`: `EXIT from="$parent" signal="FAILURE"` triggers the adjacent
CATCH; `EXIT from="$flow"` ends the whole service immediately and BYPASSES the CATCH
(verified IS 12.1) -- use `$parent` when the CATCH must run (rollback, logging).

EXIT can be a direct child of BRANCH for value-matching:
```json
{"type": "EXIT", "label": "ERR_01", "from": "$flow", "signal": "FAILURE", "failure-message": "API Key invalid"}
```

### MAPINVOKE (inline service call within MAP)
Call a service inline during a MAP step (e.g., generate UUID, get current date):
```json
{
  "type": "MAP", "mode": "STANDALONE", "nodes": [
    {
      "type": "MAPINVOKE", "service": "pub.utils:generateUUID",
      "validate-in": "$none", "validate-out": "$none", "invoke-order": "0",
      "nodes": [
        {"type": "MAP", "mode": "INVOKEINPUT", "nodes": []},
        {"type": "MAP", "mode": "INVOKEOUTPUT", "nodes": [
          {"type": "MAPCOPY", "from": "/UUID;1;0", "to": "/contextId;1;0"}
        ]}
      ]
    }
  ]
}
```

### MAPSET with Variable Substitution
Pipeline variables use `%variableName%`, global variables also use `%GLOBAL_VAR%`:
```json
{
  "type": "MAPSET", "field": "/url;1;0", "overwrite": "true",
  "variables": "true", "globalvariables": "true",
  "d_enc": "XMLValues", "mapseti18n": "true",
  "data": "<Values version=\"2.0\"><value name=\"xml\">%SERVER_URL%/api/%resourceId%</value></Values>"
}
```

- `"variables": "true"` -- enables pipeline variable substitution (`%pipelineVar%`)
- `"globalvariables": "true"` -- enables IS global variable substitution (`%GLOBAL_VAR%`)

## Service Signature (svc_sig)

Every service needs input/output signatures:
```json
{
  "svc_sig": {
    "sig_in": {
      "node_type": "record",
      "field_type": "record",
      "field_dim": "0",
      "nillable": "true",
      "javaclass": "com.wm.util.Values",
      "rec_fields": [
        {"node_type": "field", "field_name": "myInput", "field_type": "string", "field_dim": "0", "nillable": "true"}
      ]
    },
    "sig_out": {
      "node_type": "record",
      "field_type": "record",
      "field_dim": "0",
      "nillable": "true",
      "javaclass": "com.wm.util.Values",
      "rec_fields": [
        {"node_type": "field", "field_name": "myOutput", "field_type": "string", "field_dim": "0", "nillable": "true"}
      ]
    }
  }
}
```

### Field types for rec_fields
- `"field_type": "string"` -- string field
- `"field_type": "record"` -- anonymous record (nested document)
- `"field_type": "recref"` -- typed record reference (use with `"rec_ref": "pkg.folder:docType"`)
- `"field_type": "object"` -- Java object

### Array fields
- `"field_dim": "0"` -- scalar
- `"field_dim": "1"` -- array (string list, record list)

### Record reference fields (for typed documents)
```json
{
  "node_type": "record",
  "field_name": "accounts",
  "field_type": "recref",
  "field_dim": "1",
  "nillable": "true",
  "rec_ref": "mypackage.doctypes:account",
  "rec_closed": "true"
}
```

## Document Type Creation

Before using RecordRef fields, create the document type:
```json
{
  "node_nsName": "mypackage.doctypes:account",
  "node_pkg": "MyPackage",
  "node_type": "record",
  "field_type": "record",
  "field_dim": "0",
  "nillable": "true",
  "rec_fields": [
    {"node_type": "field", "field_name": "accountId", "field_type": "string", "field_dim": "0", "nillable": "true"},
    {"node_type": "field", "field_name": "customerName", "field_type": "string", "field_dim": "0", "nillable": "true"}
  ]
}
```

## Common Mistakes

1. **LOOP MAPCOPY uses type 2 instead of type 4**: Inside LOOP, always use type 4 (RecordRef) with doc type qualifier
2. **Missing javaclass on svc_sig records**: Both sig_in and sig_out MUST have `"javaclass": "com.wm.util.Values"`
3. **Missing document type**: RecordRef (type 4) paths require the document type to exist first
4. **MAPSET record array wrong format**: Must use `<array name="xml" type="record" depth="1">` not `<value>`
5. **INVOKE without INPUT/OUTPUT maps**: The INVOKE nodes array should have MAP mode=INPUT and MAP mode=OUTPUT entries
6. **Type conversion via intermediary variable (STALE VALUE BUG)**: NEVER create an intermediary variable for type conversion (e.g., long→string). If the source is null/empty, the intermediary retains its previous value from an earlier mapping, producing stale data instead of null. ALWAYS use MAPINVOKE as an inline transformer directly in the MAP step. See correct pattern below.
7. **No null check before transformation**: When converting types (object→string, long→string, etc.), ALWAYS check for null first. A null value passed to a converter may produce unexpected results or exceptions.
8. **Whole-record copy then child write in the SAME MAP step**: `MAPCOPY /hdrs;2;0 -> /headers;2;0` followed in the same MAP by a MAPSET/MAPCOPY into `/headers;2;0/Accept;1;0` REPLACES the copied record -- only the child survives. Write the children into the SOURCE record in an earlier MAP, then copy the record alone (verified IS 12.1).
9. **MAPCOPY then MAPSET overwrite=false on the same target in one MAP step** is unpredictable (sometimes the default wins, sometimes the copy). Use two MAP steps: the copy, then the MAPSET with `overwrite: "false"`.
10. **Input-map copies stay in the CALLER's pipeline**: variables an INVOKE's INPUT map copies into the called service's inputs remain in the parent pipeline even when the callee drops them at its end. MAPDELETE them in the OUTPUT map of the INVOKE, otherwise secrets (client_secret, tokens) leak into the parent service's output.
11. **pub.list:appendToDocumentList stores a REFERENCE to `fromItem`**: reusing the same document variable for the next entry overwrites the previous one. MAPDELETE the document after every append (or MAPSET a fresh one).

### CORRECT: Type conversion with inline MAPINVOKE (no intermediary variable)

Convert a long field directly to a string target using `pub.string:objectToString` as an inline transformer.
**WRONG approach** (intermediary variable, stale value bug):
```
MAP: MAPCOPY /source/longField;3;0 -> /tempObject;3;0   ← intermediary!
INVOKE: pub.string:objectToString on /tempObject
MAP: MAPCOPY /string -> /target/stringField;1;0
```
If `longField` is null, `/tempObject` may still hold a value from a previous invocation → stale data.

**CORRECT approach** (inline MAPINVOKE, no intermediary):
```json
{"type": "MAP", "mode": "STANDALONE", "nodes": [
  {"type": "MAPINVOKE", "service": "pub.string:objectToString",
   "validate-in": "$none", "validate-out": "$none", "invoke-order": "0",
   "nodes": [
     {"type": "MAP", "mode": "INVOKEINPUT", "nodes": [
       {"type": "MAPCOPY", "from": "/longField;3;0", "to": "/object;3;0"}
     ]},
     {"type": "MAP", "mode": "INVOKEOUTPUT", "nodes": [
       {"type": "MAPCOPY", "from": "/string;1;0", "to": "/target/stringField;1;0"}
     ]}
   ]}
]}
```
The MAPINVOKE input/output are scoped to the transformer — no pipeline pollution, no stale values.

**CRITICAL: MAPINVOKE input paths must be FLAT (not deeply nested).** The INVOKEINPUT context does not resolve multi-level nested paths like `/parent/child/field;3;0`. If you need to convert a deeply nested field, first MAPCOPY it to a flat pipeline variable, then use MAPINVOKE on that flat variable:
```json
{"type": "MAP", "mode": "STANDALONE", "nodes": [
  {"type": "MAPCOPY", "from": "/selectOutput;2;0/results;2;0/ORDER_ID;3;0", "to": "/tempOrderId;3;0"},
  {"type": "MAPINVOKE", "service": "pub.string:objectToString",
   "validate-in": "$none", "validate-out": "$none", "invoke-order": "0",
   "nodes": [
     {"type": "MAP", "mode": "INVOKEINPUT", "nodes": [
       {"type": "MAPCOPY", "from": "/tempOrderId;3;0", "to": "/object;3;0"}
     ]},
     {"type": "MAP", "mode": "INVOKEOUTPUT", "nodes": [
       {"type": "MAPCOPY", "from": "/string;1;0", "to": "/200/orderId;1;0"}
     ]}
   ]},
  {"type": "MAPDELETE", "field": "/tempOrderId;3;0"}
]}
```

**EVEN BETTER: Null-safe type conversion with BRANCH guard:**
```json
{"type": "BRANCH", "switch": "/longField", "nodes": [
  {"type": "SEQUENCE", "label": "$null", "exit-on": "FAILURE", "nodes": []},
  {"type": "MAP", "label": "$default", "mode": "STANDALONE", "nodes": [
    {"type": "MAPINVOKE", "service": "pub.string:objectToString",
     "validate-in": "$none", "validate-out": "$none", "invoke-order": "0",
     "nodes": [
       {"type": "MAP", "mode": "INVOKEINPUT", "nodes": [
         {"type": "MAPCOPY", "from": "/longField;3;0", "to": "/object;3;0"}
       ]},
       {"type": "MAP", "mode": "INVOKEOUTPUT", "nodes": [
         {"type": "MAPCOPY", "from": "/string;1;0", "to": "/target/stringField;1;0"}
       ]}
     ]}
  ]}
]}
```
This checks for null first (`$null` → do nothing, preserving null in target), only transforms when value exists.

**Common type conversions — CORRECT input/output field names:**
- Long/Integer → String: `pub.string:objectToString` (in: `object;3;0`, out: **`string;1;0`**)
- Object → String: `pub.string:objectToString` (in: `object;3;0`, out: **`string;1;0`**)
- String → Integer: `pub.string:stringToInteger` (in: `inString;1;0`, out: `value;3;0`)
- Date → String: `pub.date:formatDate` (in: `date;3;0` + `pattern;1;0`, out: `value;1;0`)
- Any type mismatch in adapter output: use inline MAPINVOKE, never intermediary variables

**OAS/REST API services — success vs error response handling:**
- **Success (200):** Populate the output document directly (e.g., `/200/Order;2;0`). The OAS framework handles the HTTP 200 response automatically. Do NOT call `pub.flow:setHTTPResponse` for success.
- **Error (404, 500):** Call `pub.flow:setHTTPResponse` with responseCode and JSON error body. Set before EXIT.
- **CATCH block:** Always call `pub.flow:setHTTPResponse` with 500 + error JSON.

**Flow debugger limitation:** The debugger (`flow_debug_*` tools) cannot step into TRY/CATCH blocks (FORM="TRY"/"CATCH" sequences). To debug business logic inside TRY/CATCH, temporarily remove the TRY/CATCH wrapper or test the inner logic in a separate service.
"#;

const PUTNODE_EXAMPLES: &str = r#"# putNode Working Examples

All examples below have been tested and verified on IS 11.1 and 12.1. `put_node`
creates the node when it does not exist (no separate flow_service_create /
document_type_create call) and verifies the stored step counts afterwards.

## Example 1: Simple service with MAPSET default value

```json
{
  "node_nsName": "mypackage.services:greet",
  "node_pkg": "MyPackage",
  "node_type": "service",
  "svc_type": "flow",
  "svc_subtype": "default",
  "svc_sigtype": "java 3.5",
  "stateless": "yes",
  "pipeline_option": 1,
  "svc_sig": {
    "sig_in": {
      "node_type": "record", "field_type": "record", "field_dim": "0", "nillable": "true",
      "javaclass": "com.wm.util.Values",
      "rec_fields": [
        {"node_type": "field", "field_name": "name", "field_type": "string", "field_dim": "0", "nillable": "true"}
      ]
    },
    "sig_out": {
      "node_type": "record", "field_type": "record", "field_dim": "0", "nillable": "true",
      "javaclass": "com.wm.util.Values",
      "rec_fields": [
        {"node_type": "field", "field_name": "greeting", "field_type": "string", "field_dim": "0", "nillable": "true"}
      ]
    }
  },
  "flow": {
    "type": "ROOT", "version": "3.0", "cleanup": "true",
    "nodes": [
      {"type": "MAP", "mode": "STANDALONE", "nodes": [
        {"type": "MAPSET", "field": "/name;1;0", "overwrite": "false",
         "d_enc": "XMLValues", "mapseti18n": "true",
         "data": "<Values version=\"2.0\"><value name=\"xml\">World</value></Values>"}
      ]},
      {"type": "INVOKE", "service": "pub.string:concat",
       "validate-in": "$none", "validate-out": "$none",
       "nodes": [
        {"type": "MAP", "mode": "INPUT", "nodes": [
          {"type": "MAPSET", "field": "/inString1;1;0", "overwrite": "true",
           "d_enc": "XMLValues", "mapseti18n": "true",
           "data": "<Values version=\"2.0\"><value name=\"xml\">Hello, </value></Values>"},
          {"type": "MAPCOPY", "from": "/name;1;0", "to": "/inString2;1;0"}
        ]},
        {"type": "MAP", "mode": "OUTPUT", "nodes": [
          {"type": "MAPCOPY", "from": "/value;1;0", "to": "/greeting;1;0"}
        ]}
      ]}
    ]
  }
}
```

## Example 2: INVOKE + LOOP with RecordRef mapping (extract fields from record array)

This pattern: call a service that returns a record array, loop over it, extract a field into a string array.

**Prerequisites**: Create the document type first, then the mock service.

### Step 1: Document type
```json
{
  "node_nsName": "mypackage.doctypes:account",
  "node_pkg": "MyPackage",
  "node_type": "record",
  "field_type": "record",
  "field_dim": "0",
  "nillable": "true",
  "rec_fields": [
    {"node_type": "field", "field_name": "accountName", "field_type": "string", "field_dim": "0", "nillable": "true"},
    {"node_type": "field", "field_name": "customerName", "field_type": "string", "field_dim": "0", "nillable": "true"}
  ]
}
```

### Step 2: Mock service returning record array
```json
{
  "node_nsName": "mypackage.services:searchAccounts",
  "node_pkg": "MyPackage",
  "node_type": "service",
  "svc_type": "flow", "svc_subtype": "default", "svc_sigtype": "java 3.5",
  "stateless": "yes", "pipeline_option": 1,
  "svc_sig": {
    "sig_in": {"node_type":"record","field_type":"record","field_dim":"0","nillable":"true","javaclass":"com.wm.util.Values","rec_fields":[]},
    "sig_out": {"node_type":"record","field_type":"record","field_dim":"0","nillable":"true","javaclass":"com.wm.util.Values",
      "rec_fields":[
        {"node_type":"record","field_name":"accounts","field_type":"recref","field_dim":"1","nillable":"true","rec_ref":"mypackage.doctypes:account","rec_closed":"true"}
      ]
    }
  },
  "flow": {"type":"ROOT","version":"3.2","cleanup":"true","nodes":[
    {"type":"MAP","mode":"STANDALONE","nodes":[
      {"type":"MAPSET","field":"/accounts;4;1;mypackage.doctypes:account","overwrite":"true","d_enc":"XMLValues","mapseti18n":"true",
       "data":"<Values version=\"2.0\"><array name=\"xml\" type=\"record\" depth=\"1\"><record javaclass=\"com.wm.util.Values\"><value name=\"accountName\">acc1</value><value name=\"customerName\">Alice</value></record><record javaclass=\"com.wm.util.Values\"><value name=\"accountName\">acc2</value><value name=\"customerName\">Bob</value></record></array></Values>"}
    ]}
  ]}
}
```

### Step 3: Main service with LOOP
```json
{
  "node_nsName": "mypackage.services:getCustomers",
  "node_pkg": "MyPackage",
  "node_type": "service",
  "svc_type": "flow", "svc_subtype": "default", "svc_sigtype": "java 3.5",
  "stateless": "yes", "pipeline_option": 1,
  "svc_sig": {
    "sig_in": {"node_type":"record","field_type":"record","field_dim":"0","nillable":"true","javaclass":"com.wm.util.Values","rec_fields":[]},
    "sig_out": {"node_type":"record","field_type":"record","field_dim":"0","nillable":"true","javaclass":"com.wm.util.Values",
      "rec_fields":[
        {"node_type":"field","field_name":"customers","field_type":"string","field_dim":"1","nillable":"true"}
      ]
    }
  },
  "flow": {"type":"ROOT","version":"3.2","cleanup":"true","nodes":[
    {"type":"INVOKE","service":"mypackage.services:searchAccounts","validate-in":"$none","validate-out":"$none"},
    {"type":"LOOP","in-array":"/accounts","out-array":"/customers","nodes":[
      {"type":"MAP","mode":"STANDALONE","nodes":[
        {"type":"MAPCOPY","from":"/accounts;4;0;mypackage.doctypes:account/customerName;1;0","to":"/customers;1;0"}
      ]}
    ]},
    {"type":"MAP","mode":"STANDALONE","nodes":[
      {"type":"MAPDELETE","field":"/accounts;4;1;mypackage.doctypes:account"}
    ]}
  ]}
}
```

**Key points:**
- MAPCOPY from path uses type 4 (RecordRef): `/accounts;4;0;mypackage.doctypes:account/customerName;1;0`
- Dimension is 0 (current iteration element, not the array)
- Document type reference is required after the dimension
- MAPDELETE after LOOP cleans up the temporary array from pipeline output

## Example 3: BRANCH (conditional logic)

```json
{
  "flow": {"type":"ROOT","version":"3.0","cleanup":"true","nodes":[
    {"type":"BRANCH","switch":"/action","nodes":[
      {"type":"SEQUENCE","label":"create","exit-on":"FAILURE","nodes":[
        {"type":"INVOKE","service":"mypackage.services:createRecord","validate-in":"$none","validate-out":"$none"}
      ]},
      {"type":"SEQUENCE","label":"delete","exit-on":"FAILURE","nodes":[
        {"type":"INVOKE","service":"mypackage.services:deleteRecord","validate-in":"$none","validate-out":"$none"}
      ]},
      {"type":"SEQUENCE","label":"$default","exit-on":"FAILURE","nodes":[
        {"type":"MAP","mode":"STANDALONE","nodes":[
          {"type":"MAPSET","field":"/error;1;0","overwrite":"true","d_enc":"XMLValues","mapseti18n":"true",
           "data":"<Values version=\"2.0\"><value name=\"xml\">Unknown action</value></Values>"}
        ]}
      ]}
    ]}
  ]}
}
```

## Example 4: Nested record mapping (adapter service output extraction)

This pattern: call an adapter service, extract nested fields from the result into a flat output.
Uses type 2 (Record) with nested paths -- NO LOOP needed for single-record access.

```json
{
  "flow": {"type":"ROOT","version":"3.0","cleanup":"true","nodes":[
    {"type":"INVOKE","service":"mypackage.adapters:getAccountDetails","validate-in":"$none","validate-out":"$none",
     "nodes":[
       {"type":"MAP","mode":"INPUT","nodes":[
         {"type":"MAPCOPY","from":"/accountID;1;0","to":"/getAccountDetailsInput;2;0/EXTERNAL_ID_1;1;0"}
       ]},
       {"type":"MAP","mode":"OUTPUT","nodes":[
         {"type":"MAPDELETE","field":"/getAccountDetailsInput;2;0"}
       ]}
     ]},
    {"type":"MAP","mode":"STANDALONE","nodes":[
      {"type":"MAPCOPY","from":"/getAccountDetailsOutput;2;0/results;2;1/AccountName;1;0","to":"/accountName;1;0"},
      {"type":"MAPCOPY","from":"/getAccountDetailsOutput;2;0/results;2;1/AccountType;1;0","to":"/accountType;1;0"},
      {"type":"MAPCOPY","from":"/getAccountDetailsOutput;2;0/results;2;1/WebSite;1;0","to":"/website;1;0"},
      {"type":"MAPDELETE","field":"/getAccountDetailsOutput;2;0"}
    ]}
  ]}
}
```

**Key patterns for nested record access:**
- `from:"/parentRecord;2;0/childRecord;2;1/field;1;0"` -- chain type 2 (Record) paths
- type 2 dim 0 = single record, dim 1 = record array
- Can write to nested paths too: `to:"/inputRecord;2;0/field;1;0"` creates the structure
- Use `MAPDELETE` to clean up temporary records from the pipeline
- For adapter services: input goes to `/{serviceName}Input;2;0/field`, output comes from `/{serviceName}Output;2;0/results;2;1/field`

## Example 5: String concatenation with pub.string:concat

Build a search pattern by concatenating prefix + input + suffix.

```json
{"type":"INVOKE","service":"pub.string:concat","validate-in":"$none","validate-out":"$none","nodes":[
  {"type":"MAP","mode":"INPUT","nodes":[
    {"type":"MAPSET","field":"/inString1;1;0","overwrite":"true","d_enc":"XMLValues","mapseti18n":"true","data":"<Values version=\"2.0\"><value name=\"xml\">%</value></Values>"},
    {"type":"MAPCOPY","from":"/searchTerm;1;0","to":"/inString2;1;0"}
  ]},
  {"type":"MAP","mode":"OUTPUT","nodes":[
    {"type":"MAPCOPY","from":"/value;1;0","to":"/searchPattern;1;0"},
    {"type":"MAPDELETE","field":"/inString1;1;0"},
    {"type":"MAPDELETE","field":"/inString2;1;0"},
    {"type":"MAPDELETE","field":"/value;1;0"}
  ]}
]}
```

## Example 6: JDBC query result to typed document (listOrders pattern)

Map JDBC adapter output fields to a typed document. Each field uses full nested path.

```json
{"type":"MAP","mode":"STANDALONE","nodes":[
  {"type":"MAPCOPY","from":"/selectOrdersOutput;2;0/results;2;0/order_id;1;0","to":"/orders;4;0;mypackage.docTypes:OrderCanonical/id;1;0"},
  {"type":"MAPCOPY","from":"/selectOrdersOutput;2;0/results;2;0/order_date;1;0","to":"/orders;4;0;mypackage.docTypes:OrderCanonical/date;1;0"},
  {"type":"MAPCOPY","from":"/selectOrdersOutput;2;0/results;2;0/status;1;0","to":"/orders;4;0;mypackage.docTypes:OrderCanonical/status;1;0"},
  {"type":"MAPCOPY","from":"/selectOrdersOutput;2;0/results;2;0/customer_id;1;0","to":"/orders;4;0;mypackage.docTypes:OrderCanonical/customer;2;0/id;1;0"},
  {"type":"MAPCOPY","from":"/selectOrdersOutput;2;0/results;2;0/customer_name;1;0","to":"/orders;4;0;mypackage.docTypes:OrderCanonical/customer;2;0/name;1;0"},
  {"type":"MAPDELETE","field":"/selectOrdersOutput;2;0"}
]}
```
**Key:** Nested TO paths like `/orders;4;0;.../customer;2;0/name;1;0` create nested doc structures automatically.

## Example 7: JMS message processing (processOrder pattern)

Receive JMS message, extract body, convert to JSON, persist to DB.

```json
{"flow":{"type":"ROOT","version":"3.2","cleanup":"true","nodes":[
  {"type":"INVOKE","service":"pub.json:documentToJSON","validate-in":"$none","validate-out":"$none","nodes":[
    {"type":"MAP","mode":"INPUT","nodes":[
      {"type":"MAPCOPY","from":"/JMSMessage;4;0;pub.jms:JMSMessage/body;2;0/data;2;0","to":"/document;2;0"}
    ]},
    {"type":"MAP","mode":"OUTPUT","nodes":[
      {"type":"MAPDELETE","field":"/document;2;0"}
    ]}
  ]},
  {"type":"MAP","mode":"STANDALONE","nodes":[
    {"type":"MAPCOPY","from":"/JMSMessage;4;0;pub.jms:JMSMessage/body;2;0/data;2;0","to":"/order;4;0;mypackage.docTypes:OrderCanonical"},
    {"type":"MAPDELETE","field":"/JMSMessage;4;0;pub.jms:JMSMessage"}
  ]},
  {"type":"INVOKE","service":"mypackage.jdbc:createOrder","validate-in":"$none","validate-out":"$none","nodes":[
    {"type":"MAP","mode":"INPUT","nodes":[
      {"type":"MAPCOPY","from":"/order;4;0;mypackage.docTypes:OrderCanonical/id;1;0","to":"/createOrderInput;2;0/order_id;1;0"},
      {"type":"MAPCOPY","from":"/order;4;0;mypackage.docTypes:OrderCanonical/status;1;0","to":"/createOrderInput;2;0/status;1;0"}
    ]}
  ]}
]}}
```

## Example 8: HTTP response with JSON body (REST API pattern)

Set HTTP response code, content type, and JSON body for a REST endpoint.

```json
{"type":"INVOKE","service":"pub.flow:setHTTPResponse","validate-in":"$none","validate-out":"$none","nodes":[
  {"type":"MAP","mode":"INPUT","nodes":[
    {"type":"MAPSET","field":"/httpResponse;2;0/responseCode;1;0","overwrite":"true","d_enc":"XMLValues","mapseti18n":"true",
     "data":"<Values version=\"2.0\"><value name=\"xml\">200</value></Values>"},
    {"type":"MAPSET","field":"/httpResponse;2;0/reasonPhrase;1;0","overwrite":"true","d_enc":"XMLValues","mapseti18n":"true",
     "data":"<Values version=\"2.0\"><value name=\"xml\">OK</value></Values>"},
    {"type":"MAPSET","field":"/httpResponse;2;0/contentType;1;0","overwrite":"true","d_enc":"XMLValues","mapseti18n":"true",
     "data":"<Values version=\"2.0\"><value name=\"xml\">application/json</value></Values>"}
  ]}
]}
```

## Example 9: JMS send pattern (postOrder)

Convert document to XML, send to JMS queue, return HTTP 202.

```json
{"flow":{"type":"ROOT","version":"3.2","cleanup":"true","nodes":[
  {"type":"INVOKE","service":"pub.xml:documentToXMLString","validate-in":"$none","validate-out":"$none","nodes":[
    {"type":"MAP","mode":"INPUT","nodes":[
      {"type":"MAPCOPY","from":"/request;4;0;mypackage.docTypes:OrderRequest","to":"/document;2;0"}
    ]}
  ]},
  {"type":"INVOKE","service":"pub.jms:send","validate-in":"$none","validate-out":"$none","nodes":[
    {"type":"MAP","mode":"INPUT","nodes":[
      {"type":"MAPSET","field":"/connectionAliasName;1;0","overwrite":"true","d_enc":"XMLValues","mapseti18n":"true",
       "data":"<Values version=\"2.0\"><value name=\"xml\">DEFAULT_IS_JMS_CONNECTION</value></Values>"},
      {"type":"MAPSET","field":"/destinationName;1;0","overwrite":"true","d_enc":"XMLValues","mapseti18n":"true",
       "data":"<Values version=\"2.0\"><value name=\"xml\">/orders/posts</value></Values>"},
      {"type":"MAPSET","field":"/destinationType;1;0","overwrite":"true","d_enc":"XMLValues","mapseti18n":"true",
       "data":"<Values version=\"2.0\"><value name=\"xml\">QUEUE</value></Values>"},
      {"type":"MAPCOPY","from":"/xmldata;1;0","to":"/JMSMessage;2;0/body;2;0/string;1;0"}
    ]}
  ]}
]}}
```

## Example 10: REST connector with BRANCH on HTTP status code

Call external REST API, branch on response code, map success/error responses.

```json
{"type":"SEQUENCE","exit-on":"FAILURE","nodes":[
  {"type":"INVOKE","service":"wm.server.openapi:invoke","validate-in":"$none","validate-out":"$none","nodes":[
    {"type":"MAP","mode":"INPUT","nodes":[
      {"type":"MAPSET","field":"/path;1;0","overwrite":"true","d_enc":"XMLValues","mapseti18n":"true",
       "data":"<Values version=\"2.0\"><value name=\"xml\">/customers</value></Values>"},
      {"type":"MAPSET","field":"/httpMethod;1;0","overwrite":"true","d_enc":"XMLValues","mapseti18n":"true",
       "data":"<Values version=\"2.0\"><value name=\"xml\">POST</value></Values>"},
      {"type":"MAPSET","field":"/radNamespace;1;0","overwrite":"false","d_enc":"XMLValues","mapseti18n":"true",
       "data":"<Values version=\"2.0\"><value name=\"xml\">mypackage.client:apiDescriptor</value></Values>"}
    ]}
  ]},
  {"type":"BRANCH","switch":"","evaluate-labels":"true","nodes":[
    {"type":"SEQUENCE","label":"code = 201","exit-on":"FAILURE","nodes":[
      {"type":"MAP","mode":"STANDALONE","nodes":[
        {"type":"MAPCOPY","from":"/response;3;0","to":"/201;2;0"}
      ]}
    ]},
    {"type":"SEQUENCE","label":"code = 400","exit-on":"FAILURE","nodes":[
      {"type":"MAP","mode":"STANDALONE","nodes":[
        {"type":"MAPCOPY","from":"/response;3;0","to":"/400;2;0"}
      ]}
    ]},
    {"type":"SEQUENCE","label":"$default","exit-on":"FAILURE","nodes":[
      {"type":"MAP","mode":"STANDALONE","nodes":[
        {"type":"MAPCOPY","from":"/response;3;0","to":"/error;2;0"}
      ]}
    ]}
  ]}
]}
```
**Key patterns from real projects (demoOrderManagement, obsCustomerManagement):**
- JDBC results: `/selectOutput;2;0/results;2;0/column;1;0` -> nested typed doc
- JMS body: `/JMSMessage;4;0;pub.jms:JMSMessage/body;2;0/data;2;0`
- HTTP response: `/httpResponse;2;0/responseCode;1;0` etc via MAPSET
- REST connector: `wm.server.openapi:invoke` with path/method/radNamespace + BRANCH on status code
- RecordRef copy: `/source;4;0;pkg:DocType` TO `/target;4;0;pkg:DocType` preserves type

## Example 11: TRY/CATCH with error handling (production API pattern)

REST API service with TRY/CATCH, BRANCH on status, EXIT on failure, error logging in CATCH.
Based on obsCustomerManagement:getCustomers pattern.

```json
{
  "node_nsName": "mypackage.services:getResource",
  "node_pkg": "MyPackage",
  "node_type": "service",
  "svc_type": "flow",
  "svc_subtype": "default",
  "svc_sigtype": "java 3.5",
  "stateless": "yes",
  "pipeline_option": 1,
  "svc_sig": {
    "sig_in": {
      "node_type": "record", "field_type": "record", "field_dim": "0", "nillable": "true",
      "javaclass": "com.wm.util.Values",
      "rec_fields": [
        {"node_type": "field", "field_name": "resourceId", "field_type": "string", "field_dim": "0", "nillable": "true"}
      ]
    },
    "sig_out": {
      "node_type": "record", "field_type": "record", "field_dim": "0", "nillable": "true",
      "javaclass": "com.wm.util.Values",
      "rec_fields": [
        {"node_type": "field", "field_name": "result", "field_type": "string", "field_dim": "0", "nillable": "true"}
      ]
    }
  },
  "flow": {
    "type": "ROOT", "version": "3.0", "cleanup": "true",
    "nodes": [
      {
        "type": "SEQUENCE", "exit-on": "FAILURE", "form": "TRY",
        "nodes": [
          {
            "type": "SEQUENCE", "exit-on": "FAILURE", "comment": "validate input",
            "nodes": [
              {"type": "BRANCH", "switch": "/resourceId", "nodes": [
                {"type": "SEQUENCE", "label": "$null", "exit-on": "FAILURE", "nodes": [
                  {"type": "MAP", "mode": "STANDALONE", "nodes": [
                    {"type": "MAPSET", "field": "/error/status;1;0", "overwrite": "true",
                     "d_enc": "XMLValues", "mapseti18n": "true",
                     "data": "<Values version=\"2.0\"><value name=\"xml\">400</value></Values>"},
                    {"type": "MAPSET", "field": "/error/message;1;0", "overwrite": "true",
                     "d_enc": "XMLValues", "mapseti18n": "true",
                     "data": "<Values version=\"2.0\"><value name=\"xml\">resourceId is required</value></Values>"}
                  ]},
                  {"type": "EXIT", "from": "$parent", "signal": "FAILURE", "failure-message": "resourceId is required"}
                ]},
                {"type": "SEQUENCE", "label": "$default", "exit-on": "FAILURE", "nodes": []}
              ]}
            ]
          },
          {
            "type": "SEQUENCE", "exit-on": "FAILURE", "comment": "call backend service",
            "nodes": [
              {"type": "INVOKE", "service": "mypackage.impl:fetchResource", "validate-in": "$none", "validate-out": "$none"},
              {"type": "MAP", "mode": "STANDALONE", "nodes": [
                {"type": "MAPCOPY", "from": "/fetchOutput/data;1;0", "to": "/result;1;0"}
              ]}
            ]
          },
          {
            "type": "SEQUENCE", "exit-on": "FAILURE", "comment": "set success response",
            "nodes": [
              {"type": "INVOKE", "service": "pub.flow:setHTTPResponse", "validate-in": "$none", "validate-out": "$none",
               "nodes": [
                 {"type": "MAP", "mode": "INPUT", "nodes": [
                   {"type": "MAPSET", "field": "/responseCode;1;0", "overwrite": "true",
                    "d_enc": "XMLValues", "mapseti18n": "true",
                    "data": "<Values version=\"2.0\"><value name=\"xml\">200</value></Values>"}
                 ]}
               ]
              }
            ]
          }
        ]
      },
      {
        "type": "SEQUENCE", "exit-on": "FAILURE", "form": "CATCH",
        "nodes": [
          {"type": "BRANCH", "switch": "/error/status", "nodes": [
            {"type": "SEQUENCE", "label": "$null", "exit-on": "FAILURE", "comment": "unhandled error",
             "nodes": [
              {"type": "MAP", "mode": "STANDALONE", "nodes": [
                {"type": "MAPSET", "field": "/error/status;1;0", "overwrite": "true",
                 "d_enc": "XMLValues", "mapseti18n": "true",
                 "data": "<Values version=\"2.0\"><value name=\"xml\">500</value></Values>"},
                {"type": "MAPSET", "field": "/error/message;1;0", "overwrite": "true",
                 "d_enc": "XMLValues", "mapseti18n": "true",
                 "data": "<Values version=\"2.0\"><value name=\"xml\">Internal error</value></Values>"}
              ]},
              {"type": "INVOKE", "service": "pub.flow:getLastFailureCaught", "validate-in": "$none", "validate-out": "$none"},
              {"type": "INVOKE", "service": "pub.flow:debugLog", "validate-in": "$none", "validate-out": "$none",
               "nodes": [
                 {"type": "MAP", "mode": "INPUT", "nodes": [
                   {"type": "MAPCOPY", "from": "/failureMessage;1;0", "to": "/message;1;0"},
                   {"type": "MAPSET", "field": "/function;1;0", "overwrite": "true",
                    "d_enc": "XMLValues", "mapseti18n": "true",
                    "data": "<Values version=\"2.0\"><value name=\"xml\">mypackage.services:getResource</value></Values>"},
                   {"type": "MAPSET", "field": "/level;1;0", "overwrite": "true",
                    "d_enc": "XMLValues", "mapseti18n": "true",
                    "data": "<Values version=\"2.0\"><value name=\"xml\">Error</value></Values>"}
                 ]}
               ]
              }
            ]}
          ]},
          {"type": "INVOKE", "service": "pub.flow:setHTTPResponse", "validate-in": "$none", "validate-out": "$none",
           "nodes": [
             {"type": "MAP", "mode": "INPUT", "nodes": [
               {"type": "MAPCOPY", "from": "/error/status;1;0", "to": "/responseCode;1;0"},
               {"type": "MAPCOPY", "from": "/error/message;1;0", "to": "/reasonPhrase;1;0"}
             ]}
           ]
          }
        ]
      }
    ]
  }
}
```

**TRY/CATCH key points (from obsCustomerManagement, srvCustomerManagement):**
- `form: "TRY"` and `form: "CATCH"` are SIBLINGS, not nested
- Both always have `exit-on: "FAILURE"`
- In TRY: set `/error/status` and `/error/message` BEFORE the EXIT step so CATCH knows the error type
- EXIT with `from: "$parent"` and `signal: "FAILURE"` triggers the CATCH
- In CATCH: check `/error/status` -- if `$null`, it's an unhandled error (call `pub.flow:getLastFailureCaught`)
- `pub.flow:getLastFailureCaught` returns: `failureMessage`, `failureName`, `failure` (exception object)
- `pub.flow:debugLog` inputs: `message`, `function` (service name), `level` (Error/Warn/Info/Debug)
- For transactions: start before TRY, `pub.art.transaction:rollbackTransaction` in CATCH, commit at end of TRY

## Example 12: TRY/CATCH with transaction and compensation (createCustomer pattern)

Flow structure for transactional service with external API calls and DB operations:

```
FLOW root:
  SEQUENCE (init): startTransaction
  SEQUENCE form="TRY":
    SEQUENCE: call external API -> BRANCH on status -> EXIT on failure
    SEQUENCE: JDBC insert (within transaction)
    SEQUENCE: call second API -> BRANCH on status -> EXIT on failure
    SEQUENCE: commitTransaction + set 201 response
  SEQUENCE form="CATCH":
    BRANCH on /customerId:
      $null: skip rollback (insert never happened)
      $default: rollbackTransaction
    BRANCH on /Organization/id:
      $null: skip (API create never happened)
      $default: call deleteOrganization (compensating action)
    BRANCH on /error/status:
      $null: set 500, getLastFailureCaught, debugLog
    setHTTPResponse from /error/status
    BRANCH on /error/status:
      500: throwExceptionForRetry (for trigger retry)
```

**Key insight:** Check what was already created/modified before deciding what to roll back. Use pipeline variables set during the TRY block as flags.

## Example 13: MAPINVOKE (inline service call in MAP)

Generate a UUID and timestamp inside a MAP step:

```json
{"type": "MAP", "mode": "STANDALONE", "nodes": [
  {"type": "MAPINVOKE", "service": "pub.utils:generateUUID",
   "validate-in": "$none", "validate-out": "$none", "invoke-order": "0",
   "nodes": [
     {"type": "MAP", "mode": "INVOKEINPUT", "nodes": []},
     {"type": "MAP", "mode": "INVOKEOUTPUT", "nodes": [
       {"type": "MAPCOPY", "from": "/UUID;1;0", "to": "/correlationId;1;0"}
     ]}
   ]
  },
  {"type": "MAPINVOKE", "service": "pub.date:getCurrentDateString",
   "validate-in": "$none", "validate-out": "$none", "invoke-order": "1",
   "nodes": [
     {"type": "MAP", "mode": "INVOKEINPUT", "nodes": [
       {"type": "MAPSET", "field": "/pattern;1;0", "overwrite": "true",
        "d_enc": "XMLValues", "mapseti18n": "true",
        "data": "<Values version=\"2.0\"><value name=\"xml\">yyyy-MM-dd'T'HH:mm:ss.SSSZ</value></Values>"}
     ]},
     {"type": "MAP", "mode": "INVOKEOUTPUT", "nodes": [
       {"type": "MAPCOPY", "from": "/value;1;0", "to": "/timestamp;1;0"}
     ]}
   ]
  }
]}
```

**MAPINVOKE rules:**
- Inside MAP mode=STANDALONE, INVOKEINPUT, or INVOKEOUTPUT
- Child MAPs use mode `INVOKEINPUT` and `INVOKEOUTPUT` (not INPUT/OUTPUT)
- `invoke-order` controls execution order when multiple MAPINVOKEs exist
- Common uses: `pub.utils:generateUUID`, `pub.date:getCurrentDate`, `pub.date:getCurrentDateString`, `pub.string:concat`, `pub.list:appendToDocumentList`

## Example 14: MAPSET with global/pipeline variable substitution

```json
{"type": "MAP", "mode": "STANDALONE", "nodes": [
  {"type": "MAPSET", "field": "/apiUrl;1;0", "overwrite": "true",
   "variables": "true", "globalvariables": "true",
   "d_enc": "XMLValues", "mapseti18n": "true",
   "data": "<Values version=\"2.0\"><value name=\"xml\">%API_BASE_URL%/customers/%customerId%</value></Values>"},
  {"type": "MAPSET", "field": "/password;1;0", "overwrite": "true",
   "variables": "false", "globalvariables": "true",
   "d_enc": "XMLValues", "mapseti18n": "true",
   "data": "<Values version=\"2.0\"><value name=\"xml\">%SERVICE_PASSWORD%</value></Values>"}
]}
```

- `variables: "true"` -> `%customerId%` is replaced with pipeline variable value
- `globalvariables: "true"` -> `%API_BASE_URL%` and `%SERVICE_PASSWORD%` are replaced with IS global variable values
- Common pattern: global vars for server URLs and passwords, pipeline vars for dynamic values

## Example 15: pub.flow:clearPipeline (preserve specific variables)

Clean pipeline keeping only specified variables:
```json
{"type": "INVOKE", "service": "pub.flow:clearPipeline", "validate-in": "$none", "validate-out": "$none",
 "nodes": [
   {"type": "MAP", "mode": "INPUT", "nodes": [
     {"type": "MAPSET", "field": "/preserve;1;1", "overwrite": "true",
      "d_enc": "XMLValues", "mapseti18n": "true",
      "data": "<Values version=\"2.0\"><array name=\"xml\" type=\"value\" depth=\"1\"><value>responseCode</value><value>responseBody</value><value>error</value></array></Values>"}
   ]}
 ]
}
```

- `preserve` is a String array (field type `1;1`) listing variable names to keep
- Everything else is removed from the pipeline

## Example 16: REPEAT step (putNode type RETRY -- retry on failure with backoff)

Retry a service call up to 3 times with 5-second intervals. The step Designer
calls REPEAT is typed **`RETRY`** in putNode JSON; `"type": "REPEAT"` is unknown
to the flow compiler and silently discards the step and its children:

```json
{
  "type": "RETRY", "count": "3", "backoff": "5", "repeat-on": "FAILURE",
  "nodes": [
    {"type": "INVOKE", "service": "mypackage.services:callExternalAPI", "validate-in": "$none", "validate-out": "$none"},
    {"type": "BRANCH", "switch": "/responseCode", "nodes": [
      {"type": "SEQUENCE", "label": "200", "exit-on": "FAILURE", "nodes": []},
      {"type": "SEQUENCE", "label": "$default", "exit-on": "FAILURE", "nodes": [
        {"type": "EXIT", "from": "$parent", "signal": "FAILURE", "failure-message": "API call failed with code %responseCode%"}
      ]}
    ]}
  ]
}
```

**RETRY (REPEAT) rules:**
- `repeat-on: "FAILURE"` = retry when a child fails (retry pattern for transient errors)
- `repeat-on: "SUCCESS"` = repeat while children succeed (polling pattern, batch loop, JMS receive loop)
- `count`: max re-executions. `-1` = unlimited. Supports `%variable%` substitution.
- `backoff`: seconds between iterations (Designer "Repeat interval"). `repeat-interval` / `back-off` are ignored.
- EXIT inside RETRY with `from: "$loop"` breaks out of the loop; `$retries` is the iteration counter.
- Verified on IS 12.1: `{"type":"RETRY","count":"3","backoff":"1","repeat-on":"SUCCESS"}` around a
  MAP + `BRANCH evaluate-labels` + `EXIT $loop` writes `<RETRY COUNT="3" BACK-OFF="1" LOOP-ON="SUCCESS">`
  and leaves the loop on the first iteration when the expression matches.

## Example 17: LOOP with conditional limit (process at most N items)

Process items from an array but stop after a configurable limit:

```json
{
  "type": "LOOP", "in-array": "/files", "out-array": "/results",
  "nodes": [
    {"type": "BRANCH", "evaluate-labels": "true", "nodes": [
      {"type": "EXIT", "label": "%processedCount% >= %limit%", "from": "$loop", "signal": "SUCCESS"}
    ]},
    {"type": "SEQUENCE", "exit-on": "FAILURE", "nodes": [
      {"type": "INVOKE", "service": "mypackage.services:processFile", "validate-in": "$none", "validate-out": "$none"},
      {"type": "MAP", "mode": "STANDALONE", "nodes": [
        {"type": "MAPINVOKE", "service": "pub.math:addInts", "validate-in": "$none", "validate-out": "$none", "invoke-order": "0",
         "nodes": [
           {"type": "MAP", "mode": "INVOKEINPUT", "nodes": [
             {"type": "MAPCOPY", "from": "/processedCount;1;0", "to": "/num1;1;0"},
             {"type": "MAPSET", "field": "/num2;1;0", "overwrite": "true",
              "d_enc": "XMLValues", "mapseti18n": "true",
              "data": "<Values version=\"2.0\"><value name=\"xml\">1</value></Values>"}
           ]},
           {"type": "MAP", "mode": "INVOKEOUTPUT", "nodes": [
             {"type": "MAPCOPY", "from": "/value;1;0", "to": "/processedCount;1;0"}
           ]}
         ]
        }
      ]}
    ]}
  ]
}
```

**Pattern:** BRANCH with `evaluate-labels: "true"` at the top of the LOOP body acts as a guard. The EXIT label is evaluated as an expression — when it matches, the loop breaks.

## Example 18: BRANCH with expression labels (multi-condition routing)

Route processing based on complex conditions:

```json
{
  "type": "BRANCH", "evaluate-labels": "true",
  "nodes": [
    {"type": "SEQUENCE", "label": "name != null &amp;&amp; status != null", "exit-on": "FAILURE",
     "comment": "both name and status provided",
     "nodes": [{"type": "INVOKE", "service": "mypackage.services:searchByNameAndStatus"}]},
    {"type": "SEQUENCE", "label": "name != null", "exit-on": "FAILURE",
     "comment": "only name provided",
     "nodes": [{"type": "INVOKE", "service": "mypackage.services:searchByName"}]},
    {"type": "SEQUENCE", "label": "status != null", "exit-on": "FAILURE",
     "comment": "only status provided",
     "nodes": [{"type": "INVOKE", "service": "mypackage.services:searchByStatus"}]},
    {"type": "SEQUENCE", "label": "$default", "exit-on": "FAILURE",
     "nodes": [{"type": "INVOKE", "service": "mypackage.services:searchAll"}]}
  ]
}
```

**Expression syntax:** `= value`, `!= null`, `== $null`, `&amp;&amp;` (AND), `||` (OR), regex `/^pattern/`.
First matching expression wins. `$default` is the fallback.

## Example 19: SOAP client call (WSD connector pattern)

Structure generated by IS when consuming a WSDL:

```json
{
  "type": "SEQUENCE", "exit-on": "FAILURE",
  "nodes": [
    {"type": "MAP", "mode": "STANDALONE", "comment": "set operation",
     "nodes": [
       {"type": "MAPSET", "field": "/wsdOperationName;1;0", "overwrite": "true",
        "d_enc": "XMLValues", "mapseti18n": "true",
        "data": "<Values version=\"2.0\"><value name=\"xml\">getCustomer</value></Values>"}
     ]},
    {"type": "MAP", "mode": "STANDALONE", "comment": "map request to SOAP body",
     "nodes": [
       {"type": "MAPCOPY", "from": "/request;4;0;mypackage.docTypes:getCustomerInput", "to": "/request;2;0/getCustomer;2;0"},
       {"type": "MAPSET", "field": "/soapProtocol;1;0", "overwrite": "true",
        "d_enc": "XMLValues", "mapseti18n": "true",
        "data": "<Values version=\"2.0\"><value name=\"xml\">SOAP 1.1 Protocol</value></Values>"}
     ]},
    {"type": "INVOKE", "service": "pub.client:soapClient", "validate-in": "$none", "validate-out": "$none",
     "nodes": [
       {"type": "MAP", "mode": "INPUT", "nodes": [
         {"type": "MAPSET", "field": "/method;2;0/localName;1;0", "overwrite": "true",
          "d_enc": "XMLValues", "mapseti18n": "true",
          "data": "<Values version=\"2.0\"><value name=\"xml\">getCustomer</value></Values>"}
       ]}
     ]},
    {"type": "BRANCH", "switch": "/soapStatus", "comment": "check SOAP success/fault",
     "nodes": [
       {"type": "SEQUENCE", "label": "0", "exit-on": "FAILURE", "comment": "success",
        "nodes": [
          {"type": "MAP", "mode": "STANDALONE", "nodes": [
            {"type": "MAPCOPY", "from": "/response;2;0/getCustomerResponse;2;0", "to": "/result;4;0;mypackage.docTypes:getCustomerOutput"},
            {"type": "MAPDELETE", "field": "/response;2;0"},
            {"type": "MAPDELETE", "field": "/soapStatus;1;0"}
          ]}
        ]},
       {"type": "SEQUENCE", "label": "$default", "exit-on": "FAILURE", "comment": "SOAP fault",
        "nodes": [
          {"type": "MAP", "mode": "STANDALONE", "nodes": [
            {"type": "MAPCOPY", "from": "/response;2;0/fault;2;0", "to": "/fault;2;0"},
            {"type": "MAPDELETE", "field": "/response;2;0"}
          ]}
        ]}
     ]}
  ]
}
```

**SOAP client pattern (from Tundra, IBM WxMCPServer):**
- `pub.client:soapClient` is the IS built-in SOAP invoker
- `soapStatus=0` means success, anything else is a SOAP fault
- Request: map typed doc -> `/request;2;0/operationName;2;0`
- Response: extract from `/response;2;0/operationNameResponse;2;0`
- Fault: extract from `/response;2;0/fault;2;0`
"#;

const ADAPTER_CONNECTION_REF: &str = r#"# Adapter Connection Configuration Reference

How to create an adapter connection node with `adapter_connection_create`.
Everything below was verified against a live JDBC Adapter 10.3 / IS 12.

## An adapter connection is NOT a JDBC pool

| | `jdbc_pool_*` | `adapter_connection_*` |
|---|---|---|
| What | IS-internal connection pool | JCA connection node owned by an adapter |
| Used by | the server itself (ISCoreAudit, TN, xref...) | adapter services (Select/Insert/Update/Delete), notifications |
| Configured with | a JDBC **URL** + **Driver** class (`com.wm.dd.jdbc.*`) | discrete properties + **DataSource** class (`com.wm.dd.jdbcx.*`) |
| Lives in | `config/jdbc/pool/<name>.xml` | a package namespace, as a node |

`adapter_service_create` can only be built on an adapter connection. A JDBC pool
will never work there, and the JDBC-pool vocabulary (url, uid, pwd, drivers,
mincon/maxcon) is rejected by `connection_settings`.

## The 5 rules

1. **`adapter_type` is the adapter type, not the package.** Take it verbatim from
   `adapter_type_list` -> `adapterName`. JDBC is **`JDBCAdapter`**.
   `WmJDBCAdapter` is the *package* and fails with
   `HTTP 500: [ART.114.232] Adapter Runtime (Metadata): Unable to get the adapter type "WmJDBCAdapter".`
2. **`connection_settings` keys are the `systemName` values returned by
   `adapter_connection_metadata`.** Call it first; never invent key names.
   Properties whose metadata carries a `resourceDomain` only accept the listed values.
3. **`connection_alias` carries no package prefix.** It is the namespace path of
   the node, so it starts at the package's lowercase root folder:
   `petstoreapi.connections:petstore`. Writing `PetstoreAPI.connections:petstore`
   does NOT error -- unlike `put_node`, `createConnectionNode` creates missing
   folders, so you silently get a folder literally named `PetstoreAPI` and the
   alias you must use everywhere else changes. The package goes in
   `package_name`, and there only.
4. **Creation never validates the settings.** `createConnectionNode` accepts
   nonsense (even `{}`) and returns HTTP 200. The configuration is only checked
   when the connection is enabled -- so "created" proves nothing.
5. **The node is created DISABLED.** Call `adapter_connection_enable`, then
   `adapter_connection_state` and check `connectionState: enabled` and
   `hasError: false`.

## Workflow

```
adapter_type_list                      -> exact adapter type name
adapter_connection_metadata(type, factory) -> the systemName of every setting
package_create / folder_create         -> optional, the alias folders are auto-created
adapter_connection_create(...)         -> node exists, disabled, unvalidated
adapter_connection_enable(alias)       -> THIS is what validates the settings
adapter_connection_state(alias)        -> connectionState=enabled, hasError=false
adapter_resource_domain_lookup(...)    -> proves the DB is really reachable
```

## JDBC connection settings

`connection_factory_type` = `com.wm.adapter.wmjdbc.connection.JDBCConnectionFactory`

| systemName | Required | Notes |
|---|---|---|
| `datasourceClass` | **yes** | The only setting enforced at enable time. A DataSource class, see table below |
| `serverName` | in practice | DB host |
| `portNumber` | in practice | as a string: `"5432"` |
| `databaseName` | in practice | DB / catalog name |
| `user` | in practice | DB user |
| `password` | in practice | plaintext here; IS stores it in the outbound password store |
| `transactionType` | no | `LOCAL_TRANSACTION` \| `XA_TRANSACTION` \| `NO_TRANSACTION` |
| `driverType` | no | `Default` |
| `networkProtocol` | no | usually empty |
| `otherProperties` | no | `property1=value1;property2=value2` |

Pool sizing is NOT part of `connection_settings`: use the `pool_min` / `pool_max`
parameters of the tool (they map to connectionManagerSettings).

### DataSource classes bundled with webMethods (DataDirect)

| Database | `datasourceClass` |
|---|---|
| PostgreSQL | `com.wm.dd.jdbcx.postgresql.PostgreSQLDataSource` |
| SQL Server | `com.wm.dd.jdbcx.sqlserver.SQLServerDataSource` |
| Oracle | `com.wm.dd.jdbcx.oracle.OracleDataSource` |
| DB2 | `com.wm.dd.jdbcx.db2.DB2DataSource` |
| MySQL | `com.wm.dd.jdbcx.mysql.MySQLDataSource` |
| Informix | `com.wm.dd.jdbcx.informix.InformixDataSource` |
| Sybase | `com.wm.dd.jdbcx.sybase.SybaseDataSource` |

Watch the package name: `com.wm.dd.jdbcx.*` -- with an x -- are the DataSource classes
the adapter needs; `com.wm.dd.jdbc.*` (what `jdbc_driver_list` returns) are Driver
classes and belong to `jdbc_pool_add`. Passing a Driver class here fails at enable time.
These DataDirect classes implement `DataSource` + `ConnectionPoolDataSource`, so
they cover `NO_TRANSACTION` and `LOCAL_TRANSACTION`; `XA_TRANSACTION` requires a
DataSource class that implements `javax.sql.XADataSource`.

## Verified example -- PostgreSQL

```json
{
  "connection_alias": "petstoreapi.connections:petstore",
  "package_name": "PetstoreAPI",
  "adapter_type": "JDBCAdapter",
  "connection_factory_type": "com.wm.adapter.wmjdbc.connection.JDBCConnectionFactory",
  "connection_settings": "{\"transactionType\":\"LOCAL_TRANSACTION\",\"driverType\":\"Default\",\"datasourceClass\":\"com.wm.dd.jdbcx.postgresql.PostgreSQLDataSource\",\"serverName\":\"localhost\",\"portNumber\":\"5432\",\"databaseName\":\"petstore\",\"user\":\"postgres\",\"password\":\"secret\",\"networkProtocol\":\"\",\"otherProperties\":\"\"}",
  "pool_min": 1,
  "pool_max": 10
}
```

Then `adapter_connection_enable("petstoreapi.connections:petstore")`, and confirm with
`adapter_resource_domain_lookup(connection_alias="petstoreapi.connections:petstore",
service_template="com.wm.adapter.wmjdbc.services.Select",
resource_domain_name="catalogNames")` -- if it returns the catalog list, the
connection really talks to the database.

SQL Server differs only in `datasourceClass` +
`portNumber: "1433"`; Oracle uses `portNumber: "1521"` and often carries the
service name in `otherProperties` (e.g. `ServiceName=ORCLPDB1`).

## Error to cause

| Message | Cause | Fix |
|---|---|---|
| `[ART.114.232] Unable to get the adapter type "X"` | `adapter_type` is a package name or a typo | use `adapter_type_list` -> `adapterName` (`JDBCAdapter`) |
| `[ADA.1.200] The JDBC DataSource class "" cannot be located` | `datasourceClass` missing, or the whole settings object used the wrong key names | re-read `adapter_connection_metadata`, use the `systemName` keys |
| `[ADA.1.200] ... class "com.wm.dd.jdbc.X" cannot be located` | a Driver class was passed instead of a DataSource class | use the `com.wm.dd.jdbcx.*` class |
| `[ART.118.5042] Unable to enable connection resource` | credentials / host / port / database wrong, or DB unreachable | test the same coordinates with `jdbc_pool_test`, check the DB is up |
| create returns 200 but the alias is absent from `adapter_connection_list` | the alias got a package prefix, so the node lives elsewhere | recreate with `folder:name` and delete the wrong node |
| `Invalid JSON: expected value at line 1 column 1` | `connection_settings` was not a JSON **string** | pass a serialized JSON object, not `key=value` text |

## Next step

Once `connectionState` is `enabled`, build adapter services on top of it:
see `wm://docs/adapter-service-reference`.
"#;

const ADAPTER_SERVICE_REF: &str = r#"# Adapter Service Configuration Reference

## Creating JDBC Adapter Services

### Workflow
1. Use `adapter_resource_domain_lookup` to browse database objects:
   - `catalogNames` -> pick catalog
   - `schemaNames` (values: [catalog]) -> pick schema
   - `tableNames` (values: [catalog, schema]) -> pick table
   - `columnInfo` (values: [catalog, schema, table]) -> get columns
2. CustomSQL statement -> `jdbc_custom_sql_create`; batch load of a table ->
   `jdbc_batch_insert_create`. Both build the template properties, create the
   node and verify it exists.
3. Other templates (Select, Insert, Update, Delete, StoredProcedure): build
   `adapter_service_settings` from the column metadata as shown below and
   call `adapter_service_create` (which also verifies the node exists).
4. `service_invoke` with a test input; the output record is `<svc>Output`.

### Select Service Settings
```json
{
  "tables.tableIndexes": ["T1"],
  "tables.catalogName": ["<catalog>"],
  "tables.schemaName": ["dbo"],
  "tables.tableName": ["orders"],
  "tables.tableType": ["TABLE"],
  "tables.realSchemaName": ["dbo"],
  "tables.columnInfo": ["id\nint NOT NULL\n4\n1\n...."],
  "select.expression": ["T1.id", "T1.customer_name"],
  "select.refColumn": ["T1.id", "T1.customer_name"],
  "select.columnType": ["int NOT NULL", "nvarchar NULL"],
  "select.JDBCType": ["INTEGER", "NVARCHAR"],
  "select.outputFieldType": ["java.lang.String", "java.lang.String"],
  "select.resultFieldType": ["java.lang.String", "java.lang.String"],
  "select.outputField": ["id", "customer_name"],
  "select.resultField": ["id", "customer_name"],
  "select.realOutputField": ["id", "customer_name"]
}
```

### Insert Service Settings
```json
{
  "tables.tableIndexes": ["T1"],
  "tables.catalogName": ["<catalog>"],
  "tables.schemaName": ["dbo"],
  "tables.tableName": ["orders"],
  "tables.tableType": ["TABLE"],
  "tables.realSchemaName": ["dbo"],
  "update.column": ["customer_name", "product"],
  "update.columnType": ["nvarchar(100) NULL", "nvarchar(100) NULL"],
  "update.JDBCType": ["NVARCHAR", "NVARCHAR"],
  "update.inputField": ["customer_name", "product"],
  "update.inputFieldType": ["java.lang.String", "java.lang.String"]
}
```
Note: Exclude identity/auto-increment columns from Insert update.* arrays.

### Select: properties that are easy to miss (verified on IS 12.1, JDBC Adapter 10.3, PostgreSQL)
- `select.sortOrder` is REQUIRED: one entry per `select.expression`, `""` for
  no ORDER BY (`"Ascend"` / `"Descend"` otherwise). A Select created without it
  fails on EVERY invocation with `[ART.114.505] ... Cannot load from object
  array because "this.sortOrder" is null` -- the node looks fine in Designer
  and in adapter_service_get, only the runtime breaks.
- int / boolean properties (`select.maxRow`, `select.queryTimeOut`,
  `select.autoDelete`) travel as JSON STRINGS (`"0"`, `"-1"`, `"false"`): a
  JSON number becomes a java.lang.Long and the metadata layer rejects it with
  `[ART.114.238] Value for parameter select.maxRow does not match data type int`.
- `tables.columnInfo` and `tables.realSchemaName` only feed Designer's
  resource-domain lookups; the runtime does not need them.
- Output shape: `<out>/results` is a RECORD LIST (`results;2;1`) even for a
  single row. A flow must read `results[0]` --
  `/selectPetOutput;2;0/results[0];2;1/name;1;0` -- and test the empty case
  (`pub.list:sizeOfList` on `results`, BRANCH on `size` = `0`). Reading
  `results;2;0` as a single record silently yields nulls.

### Select with a WHERE clause (input parameters)
`WHERE T1.id = ?` bound to an input field `id` of the `<in>` record
(`selectPetInput/id`), verified end to end:
```json
{
  "...": "the select.* / tables.* settings above, plus",
  "where.andOr": [""],
  "where.leftParen": [""],
  "where.leftExpr": ["T1.id"],
  "where.operator": ["="],
  "where.rightExpr": ["?"],
  "where.rightParen": [""],
  "where.hiddenJDBCType": ["BIGINT"],
  "where.hiddenInputType": ["java.lang.String"],
  "where.hiddenInputField": ["id"],
  "where.parameter": ["T1.id"],
  "where.inputFieldName": ["T1.id"],
  "where.hiddenInputFieldName": ["id"],
  "where.JDBCType": ["BIGINT"],
  "where.inputType": ["java.lang.String"],
  "where.inputField": ["id"]
}
```
- One row per condition across `where.andOr` / `leftParen` / `leftExpr` /
  `operator` / `rightExpr` / `rightParen`; they are concatenated in that order
  to build the SQL text (empty strings are skipped). First `andOr` is `""`,
  the next rows use `AND`, `OR`, `AND NOT`, `OR NOT`. Operators: `=`, `<>`,
  `<`, `>`, `<=`, `>=`, `LIKE`, `IS`, `IS NOT`. `rightExpr` is `?` for a bound
  parameter, or a literal SQL expression.
- Each `?` consumes, in order, one entry of `where.inputField` (name of the
  field in the input record), `where.inputType` (Java type) and
  `where.JDBCType`; that triple is what binds the prepared statement and
  what generates the input signature. The `hidden*`, `parameter` and
  `inputFieldName` arrays are Designer bookkeeping: keep them consistent
  (`parameter` and `inputFieldName` = the column expression,
  `hiddenInputFieldName` / `hiddenInputField` = the column name).
- Valid values come from adapter_resource_domain_lookup on the Select
  template: `andOr`, `operator`, `leftParen`, `rightParen`,
  `defaultExpression` (`?`), `sortModes`, `columnInfo(catalog, schema, table)`.

### Silent refusals: what `adapter_service_create` now checks for you
`wm.art.dev.service:createAdapterServiceNode` answers HTTP 200 even when the
Adapter Runtime refuses the node; the refusal is only in server.log:
`[ART.117.4030] Unable to create adapter service X. [ART.114.72] Unable to set
JavaBean properties. [ART.114.542] could not set property "realInputFields"
... argument type mismatch`. The tool verifies the node exists afterwards and
returns those log lines as an error when it does not. Rules:
- **Never send an empty JSON array** (`"inputField": []` for a parameterless
  statement): it arrives as `Object[]`, the setter wants `String[]`. The
  tool strips empty arrays and reports them in `omitted_empty_arrays`.
- int / boolean properties are JSON STRINGS (`"0"`, `"-1"`, `"false"`).
- Every `*.` array of one group must have the same length.

### CustomSQL (`com.wm.adapter.wmjdbc.services.CustomSQL`) -- use `jdbc_custom_sql_create`
`jdbc_custom_sql_create(service_name, package_name, connection_alias, sql,
inputs?, outputs?, result_row_field?)` builds everything below, creates the
node and verifies it. Verified settings (IS 12.1, JDBC Adapter 10.3,
PostgreSQL), two bind parameters, two result columns:
```json
{"sql":"SELECT o.order_line_id, o.order_id FROM staging.orders o WHERE o.order_line_id > ? AND o.order_line_id <= ?",
 "sqlFieldType":"java.lang.String",
 "colInfo":"0;from_id;BIGINT;IN;\n1;to_id;BIGINT;IN;\n0;order_line_id;BIGINT;OUT;\n1;order_id;VARCHAR;OUT;\n",
 "inputColIndexes":["0","1"], "inputExpression":["from_id","to_id"], "inputJDBCType":["BIGINT","BIGINT"],
 "inputFieldType":["java.lang.String","java.lang.String"], "inputField":["from_id","to_id"], "realInputFields":["from_id","to_id"],
 "outputColIndexes":["0","1"], "outputExpression":["order_line_id","order_id"], "outputJDBCType":["BIGINT","VARCHAR"],
 "outputFieldType":["java.lang.String","java.lang.String"], "outputField":["order_line_id","order_id"],
 "resultField":["results[].order_line_id","results[].order_id"], "resultFieldType":["java.lang.String[]","java.lang.String[]"],
 "realOutputField":["results[].order_line_id","results[].order_id"],
 "maxRow":"0", "queryTimeOut":"-1", "resultRowField":"", "resultRowFieldType":"", "designTimeLocale":"en",
 "userid":"overrideCredentials.$dbUser","useridType":"java.lang.String","inputUseridSign":"overrideCredentials.$dbUser",
 "password":"overrideCredentials.$dbPassword","passwordType":"java.lang.String","inputPasswordSign":"overrideCredentials.$dbPassword"}
```
- `colInfo` format: one line per column, `index;name;JDBCTYPE;IN|OUT;`, IN and
  OUT indexed separately from 0. The `customSQLcolInfo` lookup (`values:
  [sql]`) returns it for simple statements and **`-1`** as soon as the SQL
  has a join with aliases, a subquery, a function call, `||`, a
  schema-qualified name the parser dislikes (`public.tag`) or a reserved
  word used as an identifier -- the adapter's parser (fdb-sql-parser) gives
  up; write the column list yourself (or pass `outputs` to
  `jdbc_custom_sql_create`; an INSERT/UPDATE/DELETE with typed inputs is
  accepted without it). The runtime does not need the parser: any vendor
  SQL executes.
- `java.lang.String` works for every type in both directions: DATE,
  TIMESTAMP (`yyyy-MM-dd HH:mm:ss.SSS`), NUMERIC, BOOLEAN (`true`/`false`).
- `resultRowField` names an extra output field holding the affected-row count
  (INSERT/UPDATE/DELETE); `resultRowFieldType` is then `java.lang.String`.
- Output shape: `<svc>Output/results[]` -- a document list, empty when no
  row. Input shape: `<svc>Input/<inputField>`.

### BatchInsert (`com.wm.adapter.wmjdbc.services.BatchInsert`) -- use `jdbc_batch_insert_create`
`jdbc_batch_insert_create(service_name, package_name, connection_alias,
catalog?, schema, table, exclude_columns?)` reads the columns with the
`columnInfo` lookup and builds these settings:
```json
{"tables.tableIndexes":["T1"],"tables.catalogName":["winfarm"],"tables.schemaName":["dwh"],"tables.tableName":["dim_customer"],
 "tables.tableType":["TABLE"],"tables.columnInfo":["<columnInfo string from the lookup>"],"tables.realSchemaName":["dwh"],
 "update.column":["customer_code","customer_name"],"update.columnType":["CHARACTER(10) VARYING NOT NULL","CHARACTER(80) VARYING"],
 "update.JDBCType":["VARCHAR","VARCHAR"],"update.expression":["?","?"],
 "update.inputColumn":["customer_code","customer_name"],"update.inputColumnType":["...","..."],"update.inputJDBCType":["VARCHAR","VARCHAR"],
 "update.inputField":["customer_code","customer_name"],"update.inputFieldType":["java.lang.String","java.lang.String"],
 "update.batchInputField":["inputs[].customer_code","inputs[].customer_name"],"update.batchInputFieldType":["java.lang.String[]","java.lang.String[]"],
 "update.realInputField":["inputs[].customer_code","inputs[].customer_name"],"update.queryTimeOut":"-1",
 "updatecount.fieldName":"updateCount","updatecount.updateCountOutputName":["updateCount[]"],
 "updatecount.updateCountOutputType":["java.lang.String[]"],"updatecount.realOutput":["updateCount[]"],
 "designTimeLocale":"en", "userid":"overrideCredentials.$dbUser", "...": "same credential properties as CustomSQL"}
```
- Signature: `<svc>Input/inputs[]` (document LIST, one document per row,
  String fields named after the columns) -> `<svc>Output/updateCount[]`.
- Generated SQL: `INSERT INTO <catalog>.<schema>.<table>(cols) VALUES (?,...)`
  (three-part name: fine on PostgreSQL when catalog = current database).
  `catalog` may be the adapter's own `<current catalog>` entry (first value
  of the `catalogNames` lookup, Designer's default): the INSERT then runs in
  the connection's database without a catalog qualifier -- verified.
  `jdbc_batch_insert_create` uses it when `catalog` is omitted.
- Exclude serial/identity and defaulted columns (`exclude_columns`).
- The connection must be `LOCAL_TRANSACTION`: on a `NO_TRANSACTION`
  connection even the lookups fail with `[ADA.1.213] Batch services can only
  be configured on transaction type "LOCAL_TRANSACTION"`.
- If `inputs` is missing from the pipeline the service runs `executeUpdate`
  with no parameters and the driver answers `(07009/0) Invalid parameter
  binding(s)` -- the cause is the missing list, not the types.
- Throughput reference: 20 000 rows per call, ~8 700 rows/s end to end with
  `otherProperties: "BatchPerformanceWorkaround=true"` on the DataDirect
  PostgreSQL connection.
- Same `update.*` layout without the `batchInputField*` / `realInputField`
  `inputs[].` prefixes gives a single-row Insert.

### Resource-domain lookups with array-valued dependencies
`adapter_resource_domain_lookup` takes `values` as one entry per dependency.
Some domains depend on an ARRAY (declared `*tables.columnInfo` in the
template): `updateColumnNames`, `updateColumnTypes`, `updateJDBCTypes`. The
ART transports such an array as ONE string, elements escaped (`\` -> `\\`,
newline -> the two characters `\n`) and joined by real newlines
(`com.wm.adk.metadata.AdapterValues`). Passing the raw `columnInfo` string
(which contains newlines) splits it into its lines and fails with
`[ART.114.243] ... Cannot read field "value" because "original" is null`.
Pass a nested JSON array instead and the tool encodes it:
`values: [["<columnInfo string>"]]` -> the three domains come back together
(`updateJDBCTypes` gives the adapter's own JDBC type names per column).

`columnInfo` itself is that encoding applied twice: an array of columns,
each column an array `[name, sqlType, java.sql.Types code, position,
identifierQuote]` -- so column fields are separated by the literal two
characters `\n` and columns by real newlines. Codes: 4 INTEGER, 5 SMALLINT,
-5 BIGINT, 2 NUMERIC, 12 VARCHAR, 91 DATE, 93 TIMESTAMP, 16 BOOLEAN.

### Transactions (verified)
A `LOCAL_TRANSACTION` connection with explicit
`pub.art.transaction:startTransaction` / `commitTransaction` /
`rollbackTransaction` in each sub-flow works (any transaction name; TRUNCATE
inside the transaction is fine). Use a second `NO_TRANSACTION` connection for
a run log so its rows are visible while the load runs. In the CATCH block,
BRANCH on the transaction name (`$null` -> nothing to roll back, `$default`
-> rollback) before re-throwing with `EXIT $flow FAILURE`.

### Updating an existing adapter service
`adapter_service_update` locks the node (`wm.server.ns:lockNode`), calls
`wm.art.dev.service:updateAdapterServiceNode` and unlocks it; without the
lock the IS answers `[ART.117.4050] ... needs to be checked out or lock for
edit`. Send the complete settings (adapter_service_get, edit, send back).
Verify with service_invoke: `{"selectPetInput": {"id": "1"}}` must return
`selectPetOutput.results` (possibly empty), not an ART error.
"#;

const FLOW_STEPS_REF: &str = r#"# webMethods Flow Steps Reference
Source: IBM webMethods Integration Server 11.1.0 Documentation

A flow step is a basic unit of work expressed in the webMethods flow language that
Integration Server interprets and executes at run time.

## Available Flow Step Types

INVOKE, BRANCH, LOOP, MAP, SEQUENCE, REPEAT, EXIT, TRY, CATCH, FINALLY,
IF, ELSEIF, ELSE, SWITCH, CASE, DO, UNTIL, WHILE, BREAK, CONTINUE

---

## INVOKE

The INVOKE step calls any type of service, including other flow services and web
service connectors. You can invoke any service on the local Integration Server for
which the caller has appropriate rights, built-in services, and services on remote
Integration Servers. Flow services can call themselves recursively (ensure proper
termination logic). INVOKE also supports input/output validation against the
service signature.

### INVOKE Properties

| Property | Required | Description |
|----------|----------|-------------|
| Comments | No | Descriptive comment for the step. |
| Label | No | Name of a document (IData object) in the pipeline to restrict this step's scope. Leave blank for full pipeline access. |
| Timeout | No | Maximum seconds for step execution. If elapsed, Integration Server issues a FlowTimeoutException and continues with the next step. Supports pipeline variable substitution: `%variableName%` (must be String type). |
| Service | Yes | Fully qualified name of the service to invoke. |
| Validate input | No | Whether to validate input against the service input signature. True = validate, False = skip. |
| Validate output | No | Whether to validate output against the service output signature. True = validate, False = skip. |

### INVOKE Failure Conditions
- The invoked service fails.
- The specified service does not exist.
- The specified service is disabled.

### Pipeline View for INVOKE
The Pipeline view shows two stages:

**Before execution:** Pipeline In (variables in pipeline before service runs) and
Service In (variables the service requires as input). You can insert pipeline
modifiers here (link, set value, drop, add) to adjust pipeline contents before
the service executes.

**After execution:** Service Out (variables the service produces) and Pipeline Out
(variables available to the next step). You can insert pipeline modifiers here
to adjust results.

---

## BRANCH

The BRANCH step conditionally executes a child step based on pipeline variable
values. Two branching methods are available:

**Switch value branching:** Uses a single String variable to determine which child
step executes. The BRANCH step matches the Switch variable's value against each
child step's Label property and executes the matching child.

**Expression branching:** When Evaluate labels is True, each child step's Label
contains a conditional expression. The BRANCH executes the first child whose
label expression evaluates to True.

IMPORTANT: You cannot branch on a switch value and an expression for the same
BRANCH step.

### Branching on Switch Values
Define the switch variable in the BRANCH step's Switch property. Each child step's
Label specifies which switch value triggers its execution. Special labels:
- Empty string (blank Label): matches empty string values
- `$null`: matches null values
- `$default`: matches any unmatched value (default/fallback case)

### Branching on Expressions
Set Evaluate labels to True. Write expressions in each child step's Label property
that include pipeline variables. At run time, the BRANCH executes the first child
whose expression evaluates to True. Use `$default` label for the fallback case.

### BRANCH Properties

| Property | Required | Description |
|----------|----------|-------------|
| Comments | No | Descriptive comment for the step. |
| Scope | No | Document (IData) name to restrict scope. Leave blank for full pipeline access. |
| Timeout | No | Max seconds for execution. FlowTimeoutException on expiry. Supports `%variable%` substitution. |
| Label | No (Yes if used as BRANCH/EXIT target) | Name for this step instance, or `$null`, `$default`, blank. |
| Switch | Conditional | String field whose value determines which child executes. Do not set if Evaluate labels is True. |
| Evaluate labels | Conditional | True = branch on expressions in child labels. False = branch on Switch value. |

### BRANCH Failure Conditions
- The switch field is not in the pipeline and the BRANCH step does not contain a default child step or a child step to handle null values.
- The matching child step fails.
- The BRANCH step does not complete before the time-out period expires.

---

## LOOP

The LOOP step repeats child steps once for each element in a specified input array.
Useful for batch processing (e.g., processing each line item in a purchase order).

Any flow step can be placed inside a LOOP, including nested LOOP steps. Steps
within a LOOP are organized by indentation to show hierarchy.

### Input Array
Specify a pipeline array variable (String list, String table, document list, or
Object list) as the input. The LOOP iterates once per element.

### Output Collection
Optionally specify an output array name. The server collects output from each
iteration into an array. For example, if the loop produces a String variable
named "InventoryStatus" each iteration, the server transforms it into an array
containing all iterations' results.

Note: An EXIT step configured to exit a LOOP or an iteration affects the output
array contents.

### Pipeline Behavior Inside LOOP
Inside a LOOP body, arrays are reduced by one dimension:
- Input: String list becomes String; String table becomes String list; document list becomes single document
- Output: Similarly reduced. Each iteration produces one element; the server reassembles into an array.

### LOOP Properties

| Property | Required | Description |
|----------|----------|-------------|
| Comments | No | Descriptive comment. |
| Scope | No | Document (IData) to restrict scope. Leave blank for full pipeline. |
| Timeout | No | Max seconds. FlowTimeoutException on expiry. Supports `%variable%`. |
| Label | No (Yes if BRANCH/EXIT target) | Step name, or `$null`, `$default`, blank. |
| Input array | Yes | Pipeline array variable to iterate over (String list, String table, document list, or Object list). |
| Output array | No | Field name for collecting iteration output. Server aggregates into array. |

### LOOP Failure Conditions
- The pipeline does not contain the input array.
- The input field is not an array field.
- A child step of the LOOP fails during any iteration.
- The LOOP does not complete before the time-out period expires.

---

## MAP

The MAP step adjusts pipeline contents at any point in a flow, independent of
INVOKE steps. Capabilities:
- Link (copy) values between pipeline fields
- Remove (drop) fields from the pipeline
- Set constant values for pipeline fields
- Invoke transformers for document-to-document transformations
- Initialize input values at the start of a flow service
- Convert between document formats (XML to ebXML, etc.)

Tip: To initialize variables, insert a MAP step at the beginning of the flow
and use Set Value to assign values in Pipeline Out.

### Pipeline View for MAP
Shows a single stage with three columns:
- **Pipeline In:** All variables in the pipeline at this point in the flow.
- **Transformers:** Services inserted for value transformations.
- **Pipeline Out:** Variables available after the MAP step completes.

When first inserted, Pipeline In and Pipeline Out contain identical variables.
If the MAP is the last step, Pipeline Out also includes variables declared as
flow service output.

### MAP Properties

| Property | Required | Description |
|----------|----------|-------------|
| Comments | No | Descriptive comment. |
| Scope | No | Document (IData) to restrict scope. Leave blank for full pipeline. |
| Timeout | No | Max seconds. FlowTimeoutException on expiry. Supports `%variable%`. |
| Label | No (Yes if BRANCH/EXIT target) | Step name, or `$null`, `$default`, blank. |

### MAP Use Cases
1. Assign initial input values (initialize variables) at the start of a flow.
2. Map a document from one format to another using transformers (e.g., cXML to XML).

---

## SEQUENCE

The SEQUENCE step groups multiple flow steps that execute in order. While flow
services execute steps sequentially by default, explicit SEQUENCE is useful for:
- Grouping steps as a single alternative beneath a BRANCH step
- Specifying exit conditions (exit on first failure, first success, or after all complete)

### SEQUENCE Properties

| Property | Required | Description |
|----------|----------|-------------|
| Comments | No | Descriptive comment. |
| Scope | No | Document (IData) to restrict scope. |
| Timeout | No | Max seconds. FlowTimeoutException on expiry. Supports `%variable%`. |
| Label | No (Yes if BRANCH/EXIT target) | Step name, or `$null`, `$default`, blank. |
| Exit on | Yes | When to exit the SEQUENCE. See below. |

### Exit on Values

| Value | Behavior |
|-------|----------|
| FAILURE | Exit when a child step fails. The SEQUENCE executes children until one fails or all complete. This is the default. |
| SUCCESS | Exit when a child step succeeds or after all fail. Executes children until one succeeds or all fail. |
| DONE | Execute ALL child steps regardless of success/failure. |

IMPORTANT: Successful execution of a MAP step (including transformers) does NOT
cause the SEQUENCE to exit when Exit on = SUCCESS.

If a SEQUENCE contains an EXIT step configured to exit from the SEQUENCE,
the EXIT always causes exit regardless of the Exit on setting.

### SEQUENCE Failure Conditions
- **Exit on FAILURE:** Fails if a child step fails, or timeout expires.
- **Exit on SUCCESS:** Fails if ALL child steps fail, or timeout expires.
- **Exit on DONE:** Fails only if timeout expires.

---

## REPEAT

putNode representation: `"type": "RETRY"` with `count`, `backoff` (repeat
interval, seconds) and `repeat-on` (`SUCCESS` | `FAILURE`) -- see the Flow
Language Reference. "REPEAT" is only the Designer display name.

The REPEAT step executes child steps repeatedly, up to a specified count.
Behavior depends on the repeat condition:
- Repeat on FAILURE: Re-execute when any child step fails (retry pattern)
- Repeat on SUCCESS: Re-execute when all child steps succeed (polling pattern)

A configurable delay (repeat interval) can be set between re-executions.

### REPEAT Properties

| Property | Required | Description |
|----------|----------|-------------|
| Comments | No | Descriptive comment. |
| Scope | No | Document (IData) to restrict scope. |
| Timeout | No | Max seconds. FlowTimeoutException on expiry. Supports `%variable%`. |
| Label | No (Yes if BRANCH/EXIT target) | Step name, or `$null`, `$default`, blank. |
| Count | Yes | Max re-executions. 0 = no re-execution; positive integer = that many retries; -1 = unlimited (repeat as long as condition holds). Supports `%variable%`. |
| Repeat interval | No | Seconds to wait before re-executing. 0 = no delay. Supports `%variable%`. |
| Repeat on | Yes | SUCCESS = re-execute when all children succeed. FAILURE = re-execute when any child fails. |

### REPEAT Failure Conditions
- **Repeat on SUCCESS:** Fails when a child within the REPEAT block fails.
- **Repeat on FAILURE:** Fails when the Count limit is reached before children execute successfully.

When a REPEAT step fails as a child of another step, the failure propagates to the parent.

---

## EXIT

The EXIT step terminates execution at various levels: the entire flow service,
a specific ancestor step, or a single loop iteration. It can optionally throw an
exception when the exit represents a failure.

### EXIT Properties

| Property | Required | Description |
|----------|----------|-------------|
| Comments | No | Descriptive comment. |
| Label | No (Yes if BRANCH target) | Step name, or `$null`, `$default`, blank. |
| Exit from | Yes | Scope of exit (see values below). |
| Signal | Yes | SUCCESS = exit cleanly. FAILURE = exit and throw exception. |
| Failure name | No | Fully qualified Java class name for the exception (e.g., `java.lang.Exception`, `com.wm.app.b2b.server.ServiceException`). Must extend java.lang.Exception. Default: `com.wm.lang.flow.FlowException` for $flow, `com.wm.lang.FlowFailure` for others. Supports `%variable%`. |
| Failure instance | No | Pipeline variable (Object type) containing an existing Exception instance (typically from `pub.flow:getLastFailureCaught`). If both Failure name and Failure instance are set, Failure instance is used. |
| Failure message | No | Exception message text. Supports `%variable%` substitution. |

### Exit from Values

| Value | Exits... |
|-------|----------|
| `$parent` | Parent flow step (default). |
| `$loop` | Nearest ancestor LOOP or REPEAT step. |
| `$flow` | Entire flow service. |
| `$iteration` | Current iteration of nearest ancestor LOOP or REPEAT. |
| *label* | Nearest ancestor step whose Label matches this value. If no match, flow exits with exception. |
| *(blank)* | Same as `$loop`. |

Note: Failure name and Failure instance properties are only used when Signal = FAILURE.

---

## Data Mapping in Flow Services

Systems frequently require data transformations for compatible data exchange.
The webMethods flow language supports three transformation types:

### Name Transformations
Resolve differences in how data is named. Example: copying "telephone" to
"phoneNumber" while preserving value and position.

### Structural Transformations
Resolve differences in data type or structure. Example: moving a telephone
number from a flat String field into a nested Document structure.

### Value Transformations
Resolve differences in how values are expressed. Examples: currency codes,
date formats, measurement unit conversions.

### Implementation Methods
Two approaches:
1. **Variable links** between services (MAPCOPY - copy value from one field to another)
2. **Transformers** (specialized services inserted into MAP steps for value transformation)

### Basic Mapping Operations
1. **Link variables** - Copy values between fields across services or document formats (MAPCOPY)
2. **Assign values** - Hard-code values or set defaults for pipeline variables (MAPSET)
3. **Drop variables** - Remove unneeded pipeline variables (MAPDELETE)
4. **Add variables** - Introduce variables not in original input/output declarations

### What Is a Flow Service?
A flow service is written in the webMethods flow language. It combines multiple
services into a single unified service while controlling data flow between them.
Any service type can be invoked within a flow: other flow services, built-in
Integration Server services, adapter services, web service connectors.
Flow services are stored as XML files on the Integration Server.
IMPORTANT: Create and maintain flow services using Designer. You cannot create
or edit a flow service with a text editor (unless using the putNode API).

### What Is the Pipeline?
The pipeline is the data structure (IData object) that flows through a service,
carrying input variables, intermediate results, and output variables. Each flow
step can read from and write to the pipeline. Pipeline modifiers (link, set,
drop, add) adjust pipeline contents at each step boundary.
"#;

const BUILTIN_SERVICES_REF: &str = r#"# webMethods IS Built-In Services Reference

Compact reference for AI-assisted flow service generation. All services are in the WmPublic package.
All String-type numeric parameters use locale-neutral format (`-####.##`).

---

## pub.string (String Folder)

### pub.string:concat
Concatenates two strings.
- **In:** `inString1` (String, req), `inString2` (String, req)
- **Out:** `value` (String) - inString1 + inString2

### pub.string:indexOf
Returns index of first occurrence of a substring.
- **In:** `inString` (String, req), `subString` (String, req), `fromIndex` (String, opt, default 0)
- **Out:** `value` (String) - index, or -1 if not found

### pub.string:substring
Extracts a substring.
- **In:** `inString` (String, req), `beginIndex` (String, req, inclusive), `endIndex` (String, opt, exclusive; if null extends to end)
- **Out:** `value` (String)

### pub.string:replace
Replaces all occurrences of a substring.
- **In:** `inString` (String, req), `searchString` (String, req), `replaceString` (String, req; null/empty removes matches), `useRegex` (String, opt, default false; when true replaceString can use $1 etc.)
- **Out:** `value` (String)

### pub.string:length
Returns string length.
- **In:** `inString` (String, req), `encoding` (String, opt - IANA charset or "autodetect")
- **Out:** `value` (String) - character count

### pub.string:trim
Trims leading/trailing whitespace.
- **In:** `inString` (String, req)
- **Out:** `value` (String)

### pub.string:toLower
Converts to lowercase.
- **In:** `inString` (String, req)
- **Out:** `value` (String)

### pub.string:toUpper
Converts to uppercase.
- **In:** `inString` (String, req)
- **Out:** `value` (String)

### pub.string:compareStrings
Case-sensitive string equality check.
- **In:** `inString1` (String, opt, can be null), `inString2` (String, opt, can be null)
- **Out:** `isEqual` (String) - "true" or "false" (both null = true)

### pub.string:tokenize
Splits string into list by delimiters.
- **In:** `inString` (String, req), `delim` (String, req; null defaults to whitespace/tab/newline), `useRegex` (Boolean, opt, default false)
- **Out:** `valueList` (String List)

### pub.string:makeString
Joins String List into single string with separator.
- **In:** `elementList` (String List, req), `separator` (String, req)
- **Out:** `value` (String)

### pub.string:padLeft
Pads string on left to specified length.
- **In:** `inString` (String, req), `length` (String, req), `padChar` (String, req)
- **Out:** `value` (String)

### pub.string:padRight
Pads string on right to specified length.
- **In:** `inString` (String, req), `length` (String, req), `padChar` (String, req)
- **Out:** `value` (String)

### pub.string:base64Encode
Converts bytes to Base64 string.
- **In:** `bytes` (byte[], req), `useNewLine` (String, opt, default "true" - inserts linebreaks every 76 chars), `encoding` (String, opt, "ASCII" default or "UTF-8")
- **Out:** `value` (String)

### pub.string:base64Decode
Decodes Base64 string to bytes.
- **In:** `string` (String, req), `encoding` (String, opt, "ASCII" default or "UTF-8")
- **Out:** `value` (byte[])

### pub.string:HTMLEncode
Replaces HTML-sensitive chars with entities.
- **In:** `inString` (String, req)
- **Out:** `value` (String)

### pub.string:HTMLDecode
Replaces HTML entities with native characters.
- **In:** `inString` (String, req)
- **Out:** `value` (String)

### pub.string:URLEncode
URL-encodes a string (application/x-www-form-urlencoded).
- **In:** `inString` (String, req)
- **Out:** `value` (String)

### pub.string:URLDecode
Decodes a URL-encoded string.
- **In:** `inString` (String, req)
- **Out:** `value` (String)

### pub.string:numericFormat
Formats a number into a pattern. Rounding mode: HALF_EVEN.
- **In:** `num` (String, req), `pattern` (String, req - symbols: `0`=digit, `#`=optional digit, `.`=decimal, `,`=grouping, `%`=percent)
- **Out:** `value` (String)

### pub.string:messageFormat
Formats strings into a message pattern.
- **In:** `pattern` (String, req), `args` (String List, req)
- **Out:** `value` (String)

### pub.string:lookupDictionary
Looks up key in a Hashtable.
- **In:** `hashtable` (java.util.Hashtable, req), `key` (String, req, case-sensitive)
- **Out:** `value` (String) - null if key not found

### pub.string:lookupTable
Locates key in a String Table.
- **In:** `table` (String Table, req), `key` (String, req)
- **Out:** `value` (String)

### pub.string:bytesToString
Converts byte array to String.
- **In:** `bytes` (byte[], req), `encoding` (String, opt)
- **Out:** `value` (String)

### pub.string:stringToBytes
Converts String to byte array.
- **In:** `inString` (String, req), `encoding` (String, opt)
- **Out:** `value` (byte[])

### pub.string:isNullEmptyOrWhitespace
Checks if string is null, empty, or only whitespace.
- **In:** `inString` (String)
- **Out:** `isNullEmptyOrWhitespace` (String) - "true"/"false"

### pub.string:isNumber
Checks if string can be converted to float.
- **In:** `inString` (String)
- **Out:** `isNumber` (String) - "true"/"false"

### pub.string:isAlphanumeric
Checks if string contains only A-Z, a-z, 0-9.
- **In:** `inString` (String)
- **Out:** `isAlphanumeric` (String) - "true"/"false"

### pub.string:isDate
Checks if string matches a date format pattern.
- **In:** `inString` (String), `pattern` (String)
- **Out:** `isDate` (String) - "true"/"false"

### pub.string:objectToString
Converts object via Java toString(). CRITICAL: output field is `string`, NOT `value`.
- **In:** `object` (Object, req)
- **Out:** `string` (String) -- WARNING: this is `string`, not `value` like most other pub.string services

### pub.string:substitutePipelineVariables
Replaces pipeline variable references with their values.
- **In:** `inString` (String, req)
- **Out:** `value` (String)

---

## pub.math (Math Folder)

All numeric inputs/outputs are Strings (locale-neutral format `-####.##`) unless noted as java.lang.Number.
Float operations support optional `precision` (String) for decimal places.
Special float outputs: `Infinity`, `-Infinity`, `0.0`, `NaN`.

### Arithmetic - Integers
| Service | In | Out (`value` String) |
|---|---|---|
| `pub.math:addInts` | `num1`, `num2` (String) | num1 + num2 |
| `pub.math:subtractInts` | `num1`, `num2` (String) | num1 - num2 |
| `pub.math:multiplyInts` | `num1`, `num2` (String) | num1 * num2 |
| `pub.math:divideInts` | `num1`, `num2` (String) | num1 / num2 |

### Arithmetic - Floats
| Service | In | Out (`value` String) |
|---|---|---|
| `pub.math:addFloats` | `num1`, `num2`, `precision`(opt) | num1 + num2 |
| `pub.math:subtractFloats` | `num1`, `num2`, `precision`(opt) | num1 - num2 |
| `pub.math:multiplyFloats` | `num1`, `num2`, `precision`(opt) | num1 * num2 |
| `pub.math:divideFloats` | `num1`(dividend), `num2`(divisor), `precision`(opt) | num1 / num2 |

### Arithmetic - Objects (java.lang.Number)
| Service | In | Out (`value` java.lang.Number) |
|---|---|---|
| `pub.math:addObjects` | `num1`, `num2` (Number) | sum |
| `pub.math:subtractObjects` | `num1`, `num2` (Number) | difference |
| `pub.math:multiplyObjects` | `num1`, `num2` (Number) | product |
| `pub.math:divideObjects` | `num1`, `num2` (Number) | quotient |

Binary numeric promotion: Double > Float > Long > Integer.

### List Operations
| Service | In | Out (`value` String) |
|---|---|---|
| `pub.math:addIntList` | `numList` (String List) | sum |
| `pub.math:addFloatList` | `numList` (String List) | sum |
| `pub.math:multiplyIntList` | `numList` (String List) | product |
| `pub.math:multiplyFloatList` | `numList` (String List) | product |

### Other Math Services

**pub.math:absoluteValue** - Returns absolute value.
- **In:** `num` (String, req)
- **Out:** `value` (String)

**pub.math:max** - Returns largest number from list.
- **In:** `numList` (String List, req)
- **Out:** `maxValue` (String)

**pub.math:min** - Returns smallest number from list.
- **In:** `numList` (String List, req)
- **Out:** `minValue` (String)

**pub.math:roundNumber** - Rounds a number.
- **In:** `num` (String, req), `numberOfDigits` (String, req), `roundingMode` (String, opt, default "RoundHalfUp"; values: RoundHalfUp, RoundUp, RoundDown, RoundCeiling, RoundFloor, RoundHalfDown, RoundHalfEven)
- **Out:** `roundedNumber` (String)

**pub.math:randomDouble** - Returns pseudorandom double 0.0-1.0.
- **In:** (none)
- **Out:** `number` (String)

**pub.math:toNumber** - Converts string to numeric data type.
- **In:** `num` (String, req)
- **Out:** `value` (java.lang.Number)

---

## pub.list (List Folder)

### pub.list:appendToDocumentList
Appends documents to a document list. Appends REFERENCES, not copies: MAPDELETE `fromItem` after each call, or the next iteration's writes into the same variable overwrite the entry already in the list (verified IS 12.1).
- **In:** `toList` (Document List, opt - creates new if absent), `fromList` (Document List, opt), `fromItem` (Document, opt; added after fromList items)
- **Out:** `toList` (Document List)

### pub.list:appendToStringList
Appends strings to a string list. Appends references, not copies.
- **In:** `toList` (String List, opt - creates new if absent; null throws NPE), `fromList` (String List, opt), `fromItem` (String, opt; added after fromList items)
- **Out:** `toList` (String List)

### pub.list:sizeOfList
Returns element count of a list.
- **In:** `fromList` (Document List | String List | Object List, opt - default size 0)
- **Out:** `size` (String), `fromList` (original list passthrough)

### pub.list:stringListToDocumentList
Converts String List to Document List.
- **In:** `fromList` (String List, req)
- **Out:** `toList` (Document List)

### pub.list:addItemToVector
Adds item(s) to a java.util.Vector.
- **In:** `vector` (java.util.Vector), `item` (Object), `items` (Object List)
- **Out:** `vector` (java.util.Vector)

### pub.list:vectorToArray
Converts java.util.Vector to an array.
- **In:** `vector` (java.util.Vector)
- **Out:** `array` (Object[])

---

## pub.date (Date Folder)

### Date Pattern Symbols
`yyyy`=year, `MM`=month(01-12), `dd`=day, `HH`=hour(00-23), `hh`=hour(01-12), `mm`=minute, `ss`=second, `SSS`=millisecond, `a`=AM/PM, `z`/`Z`=timezone.
Example: `yyyy-MM-dd HH:mm:ss.SSS`, `yyyyMMdd`, `MM/dd/yyyy`

Invalid dates auto-correct (e.g. Feb 30 -> Mar 2). Two-digit years use 50-year moving window.

### pub.date:getCurrentDate
Returns current date as Date object.
- **In:** (none)
- **Out:** `date` (java.util.Date)

### pub.date:getCurrentDateString
Returns current date as formatted string.
- **In:** `pattern` (String, req), `timezone` (String, opt - e.g. "EST", "America/New_York"), `locale` (String, opt - e.g. "en", "fr")
- **Out:** `value` (String)

### pub.date:formatDate
Formats a Date object as a string.
- **In:** `date` (java.util.Date, opt), `pattern` (String, req), `timezone` (String, opt), `locale` (String, opt)
- **Out:** `value` (String)

### pub.date:dateTimeFormat
Converts date/time string from one format to another.
- **In:** `inString` (String, req), `currentPattern` (String, req), `newPattern` (String, req), `locale` (String, opt), `lenient` (String, opt, default "true")
- **Out:** `value` (String)

### pub.date:dateBuild
Builds date string from components. (Deprecated - use pub.datetime:build)
- **In:** `pattern` (String, req), `year` (String, opt, yyyy/yy), `month` (String, opt, 1-12), `dayofmonth` (String, opt)
- **Out:** `value` (String)

### pub.date:dateTimeBuild
Builds date/time string from components. (Deprecated - use pub.datetime:build)
- **In:** `pattern` (String, req), `year` (String, opt), `month` (String, opt), `dayofmonth` (String, opt), `hour` (String, opt, 0-23), `minute` (String, opt), `second` (String, opt), `timezone` (String, opt), `locale` (String, opt)
- **Out:** `value` (String)

### pub.date:compareDates
Compares two dates.
- **In:** `startDate` (String, req), `endDate` (String, req), `startDatePattern` (String, req), `endDatePattern` (String, req)
- **Out:** `result` (String) - "+1" if startDate after endDate, "0" if equal, "-1" if startDate before endDate

### pub.date:calculateDateDifference
Calculates difference between two dates. Each output is the SAME difference in different units (do NOT add them).
- **In:** `startDate` (String, req), `endDate` (String, req), `startDatePattern` (String, req), `endDatePattern` (String, req)
- **Out:** `dateDifferenceSeconds` (String), `dateDifferenceMinutes` (String), `dateDifferenceHours` (String), `dateDifferenceDays` (String) - all truncated to whole numbers
- The difference is ABSOLUTE (no sign): to know which instant is later, compare the two
  values formatted as `yyyyMMddHHmmss` strings instead (verified IS 12.1).

### pub.date:currentNanoTime
Returns current time in nanoseconds (high-precision timer).
- **In:** (none)
- **Out:** `nanoTime` (String)

### pub.date:elapsedNanoTime
Calculates elapsed nanoseconds since a given time.
- **In:** `startNanoTime` (String, req)
- **Out:** `elapsedNanoTime` (String)

### pub.date:getWorkingDays
Returns working days between two dates.
- **In:** `startDate` (String, req), `endDate` (String, req), `startDatePattern` (String, req), `endDatePattern` (String, req)
- **Out:** `workingDays` (String)

### pub.date:incrementDate
Increments date by time intervals. (Deprecated - use pub.datetime:increment)
- **In:** `startDate` (String), `pattern` (String), `years`/`months`/`days`/`hours`/`minutes`/`seconds` (String, opt)
- **Out:** `value` (String)

---

## pub.flow (Flow Folder)

### pub.flow:debugLog
Writes message to server log.
- **In:** `message` (String, opt), `function` (String, opt - source identifier), `level` (String, opt - Off/Fatal(default)/Error/Warn/Info/Debug/Trace)
- **Out:** (none)
- **Note:** Visibility controlled by logging level for facility "0090 pub Flow services"

### pub.flow:getLastError
Gets info about last trapped exception in a flow. Must be first step in catch block.
- **In:** (none)
- **Out:** `lastError` (Document) - structure per pub.event:exceptionInfo; contains error, errorType, errorDump, errorMessage, localizedError, nestedError, etc.
- **Constraints:** Only callable from flow services. Map lastError to pipeline variable immediately if needed by subsequent steps. Does NOT capture EXIT step failures.

### pub.flow:getLastFailureCaught
Returns failure details from a CATCH block (FORM="CATCH" SEQUENCE). Use INSTEAD of getLastError inside TRY/CATCH.
- **In:** (none)
- **Out:** `failureMessage` (String - the EXIT failure-message or exception message), `failureName` (String - exception class name), `failure` (Object - the Java Exception instance, can be passed to EXIT failure-instance)
- **Constraints:** Only callable from within a FORM="CATCH" SEQUENCE. Returns null values if called outside CATCH.

### pub.flow:clearPipeline
Removes all fields from the pipeline.
- **In:** `preserve` (String List, opt - field names to keep)
- **Out:** (none)

### pub.flow:throwExceptionForRetry
Throws ISRuntimeException to trigger service retry. For transient errors only.
- **In:** `wrappedException` (Object, opt), `message` (String, opt)
- **Out:** (none)
- **Constraints:** Only top-level or trigger services can be retried. Nested services cannot.

### pub.flow:invokeService
Dynamically invokes any public IS service.
- **In:** `ifcname` (String, req - e.g. "pub.math"), `svcname` (String, req - e.g. "addInts"), `pipeline` (Document, opt)
- **Out:** varies by invoked service; when pipeline specified, output appears in that pipeline document
- **Throws:** ServiceException if interface or service not found

### pub.flow:savePipeline
Saves pipeline snapshot to memory.
- **In:** `$name` (String, req)
- **Out:** (none)
- **Note:** Not persisted across server restarts. For debugging.

### pub.flow:restorePipeline
Restores previously saved pipeline.
- **In:** `$name` (String, req), `$merge` (String, opt, default "false"), `$remove` (String, opt, default "false" - remove saved copy after restore)
- **Out:** restored pipeline contents

### pub.flow:savePipelineToFile
Saves pipeline to server file.
- **In:** `fileName` (String, req - relative path)
- **Out:** (none)

### pub.flow:restorePipelineFromFile
Restores pipeline from file.
- **In:** `fileName` (String, req), `merge` (String, opt, default "false")
- **Out:** restored pipeline contents

### pub.flow:getTransportInfo
Gets protocol info for how current service was invoked.
- **In:** (none)
- **Out:** `transport` (Document) - contains `protocol` key (e.g. "http", "email") and protocol-specific sub-document
- **Constraint:** Only works for top-level services

### pub.flow:getSession
Inserts session object into pipeline.
- **In:** (none)
- **Out:** `$session` (Document) - current user session info

### pub.flow:getRetryCount
Gets retry count for current service execution.
- **In:** (none)
- **Out:** `retryCount` (String), `maxRetryCount` (String; -1 = retry-until-success trigger)

### pub.flow:getCallingService
Gets parent service info.
- **In:** (none)
- **Out:** calling service name and package info

### pub.flow:setResponse
(Deprecated - use setResponse2) Returns response to caller.
- **In:** `response` (String, req), `contentType` (String, opt - MIME type), `encoding` (String, opt)
- **Out:** (none)

### pub.flow:setResponse2
Returns response to calling process. Replaces setResponse.
- **In:** `response` (String), `contentType` (String, opt), `encoding` (String, opt)
- **Out:** (none)

### pub.flow:setResponseCode
Sets HTTP response code.
- **In:** `code` (String, req - e.g. "200", "404", "500")
- **Out:** (none)

### pub.flow:setResponseHeader
Sets single HTTP response header.
- **In:** `key` (String, req), `value` (String, req)
- **Out:** (none)

### pub.flow:setResponseHeaders
Sets multiple HTTP response headers.
- **In:** `headers` (Document, req)
- **Out:** (none)

### pub.flow:tracePipeline
Writes pipeline field names and values to server log.
- **In:** (none)
- **Out:** (none) - output goes to server log

### pub.flow:setHTTPResponse
Sets HTTP response code and optional headers/body for REST services. Preferred over setResponseCode for REST APIs.
- **In:** `responseCode` (String, req - e.g. "200", "400", "500"), `reasonPhrase` (String, opt - HTTP reason phrase), `contentType` (String, opt - e.g. "application/json"), `responseBody` (String or InputStream, opt)
- **Out:** (none)
- **Note:** Must be called before the flow returns. Common pattern: call in both TRY success path and CATCH error path.

### pub.flow:iterator
Returns IData arrays in batches.
- **In:** batch size and array input
- **Out:** batched array segments

---

## pub.art.transaction (Adapter Transaction Management)

### pub.art.transaction:startTransaction
Starts a managed adapter transaction.
- **In:** `transactionName` (String, req - unique name, commonly a UUID from pub.utils:generateUUID)
- **Out:** (none)
- **Note:** Start BEFORE a TRY block. All adapter operations (JDBC, SAP, etc.) within the same transaction name are atomic.

### pub.art.transaction:commitTransaction
Commits a managed adapter transaction.
- **In:** `transactionName` (String, req - must match the startTransaction name)
- **Out:** (none)
- **Note:** Call at the end of the TRY block, after all DB/adapter operations succeed.

### pub.art.transaction:rollbackTransaction
Rolls back a managed adapter transaction.
- **In:** `transactionName` (String, req - must match the startTransaction name)
- **Out:** (none)
- **Note:** Call in the CATCH block. Check pipeline variables to confirm the transaction was started before rolling back.

---

## Verified call patterns (IS 12.1)

- `pub.scheduler:addOneTimeTask`: `date` as `yyyy/MM/dd`, `time` as `HH:mm:ss`;
  a time in the past is refused with `[ISS.0085.9114]` -- compute the value
  with `pub.date:incrementDate` (addSeconds=3) first.
- Durations: `pub.date:currentNanoTime` and `elapsedNanoTime` return
  `java.lang.Long` objects -- convert with `pub.string:objectToString` (output
  field `string`) before `pub.math:divideFloats` (precision `0`) /
  `roundNumber`.
- `pub.flow:getLastError` in a CATCH: `lastError` is a
  `pub.event:exceptionInfo` document -- map
  `/lastError;4;0;pub.event:exceptionInfo/error;1;0` to a String, then
  `EXIT from="$flow" signal="FAILURE" failure-message="step : %errorMsg%"`
  (`%var%` substitution works in failure-message).
- `pub.jwt:generateSignedJWT`: `expirationTime` accepts ONLY the format `dd/MM/yyyy HH:mm:ss`
  (anything else fails with `[ISS.0163.9015]`); build it with `pub.date:getCurrentDateString` /
  `incrementDate` using that pattern.
- `pub.document:documentListToDocument`: `name` and `value` are REQUIRED -- they are the names
  of the key and value fields inside each list entry (e.g. `name`="key", `value`="val").
- `pub.security.outboundPasswords:setPassword` takes a `WmSecureString` for `password`
  (build it with `pub.security.util:createSecureString` / `convertSecureString`); a plain
  String is rejected. `getPassword` for an unknown key answers `password = null` with NO
  error -- BRANCH on `$null` before using it.
- Invoking a service from a UI: `POST /invoke/<folder>/<svc>` with
  `Content-Type` and `Accept: application/json` + Basic auth; files under
  the package's `pub/` directory are served with the same authentication.

## pub.json (JSON Processing)

### pub.json:documentToJSON
Converts an IData document to a JSON string.
- **In:** `document` (Document, req), `jsonStream` (opt - if true, returns OutputStream)
- **Out:** `jsonString` (String - the JSON representation)
- **Note:** Commonly used in CATCH blocks to serialize error documents for API responses.

### pub.json:jsonStringToDocument
Parses a JSON string into an IData document.
- **In:** `jsonString` (String, req), `decodeIntegerAsLong` (String, default true), `decodeRealAsDouble` (String, default true)
- **Out:** `document` (Document - the parsed IData)
- JSON numbers arrive as `java.lang.Long` / `Double` OBJECTS, not Strings: convert with
  `pub.string:objectToString` (output `string`) before `pub.math:*` or a String mapping.

---

## pub.client (HTTP/SOAP/FTP Client Services)

### pub.client:soapClient
Invokes a SOAP web service endpoint.
- **In:** `request` (Document - SOAP body), `method` (Document - `localName`, `nsURI`), `soapAction` (String), `address` (String, opt - endpoint URL), `soapProtocol` (String - "SOAP 1.1 Protocol" or "SOAP 1.2 Protocol"), `wsdBinderName` (String, opt)
- **Out:** `response` (Document - SOAP response body), `soapStatus` (String - "0"=success, "1"=fault), `header` (Document - HTTP headers)
- **Note:** `soapStatus=0` is success. Check via BRANCH on `/soapStatus`. Fault details in `/response/fault`.

### pub.client:http
Sends an HTTP request (GET, POST, PUT, DELETE, etc.).
- **In:** `url` (String, req), `method` (String - GET/POST/PUT/DELETE), `data` (Document: `string` | `bytes` | `stream` | `args` | `table`), `headers` (Document), `auth` (Document - `type`, `user`, `pass`), `encodingType` (String), `loadAs` (String: `bytes` | `stream` -- there is NO `string` option), `throwExceptionOnHttp401` (String, default true)
- **Out:** `header` (Document: `lines` (Document of response headers), `status` (String, the HTTP code), `statusMessage`), `body` (Document: `bytes` or `stream`), `encoding`
- Verified IS 12.1: the HTTP code is `header/status`, NOT a root-level `statusCode`; convert
  `body/bytes` with `pub.string:bytesToString`; set `throwExceptionOnHttp401` to `false` to
  handle a 401 yourself (token refresh) instead of catching an exception.
- IS 12.1 intercepts any INCOMING `Authorization: Bearer` header on `/invoke` and answers
  `[ISS.0010.8044] token invalid or expired` before the service runs: a flow service cannot
  host an endpoint protected by its own bearer-token check (use Basic auth, an ACL, or host
  the simulator outside the IS).

### pub.client.ftp:login
Opens FTP connection.
- **In:** `serverhost` (String), `serverport` (String, default "21"), `username` (String), `password` (String), `transfertype` (String - "ascii"/"binary"), `newSession` (String - "true"/"false"), `secure` (String - "true" for FTPS)
- **Out:** (session is stored internally)

### pub.client.ftp:get
Downloads file via FTP.
- **In:** `remoteFile` (String - path on server), `localFile` (String, opt - local path)
- **Out:** `content` (InputStream if no localFile), `status` (String)

### pub.client.ftp:put
Uploads file via FTP.
- **In:** `remoteFile` (String), `content` (InputStream/String/bytes), `mode` (String - "ascii"/"binary")
- **Out:** `status` (String)

### pub.client.ftp:logout
Closes FTP session.
- **In:** (none)
- **Out:** (none)

### pub.client.sftp:login / put / get / logout
Same pattern as FTP but for SFTP connections. Uses `serverAlias` or explicit credentials.

---

## Kafka / Streaming Integration

### Kafka Listener Configuration
webMethods IS can monitor Kafka topics via the Apache Kafka connector.

**Setup:** Events > Listeners > Kafka type
- **Connection:** Select a consumer connection (must be pre-configured)
- **Topic Name(s):** Comma-separated Kafka topic names
- **Poll Interval:** Milliseconds between checks (default: 10000ms)
- **Partition(s):** Comma-separated partition identifiers (optional)
- **Offset(s):** Starting consumption point per partition (optional; ignored without partitions)
- **Retry Limit:** Reconnection attempts on failure (default: 5)
- **Retry Backoff:** Milliseconds between retries (default: 10ms)

**Message Flow:** Listener fetches messages from Kafka topic and passes them to associated listener notifications for processing by flow services.

**Constraints:**
- Offset count cannot exceed partition count
- Multiple topic subscriptions do not support multiple partitions or offsets
- Requires pre-configured Kafka consumer account

### Messaging Model
webMethods supports publish-and-subscribe, request/reply, and publish-and-wait patterns. Messaging providers include:
- **Internal:** Built into IS for flow services and workflows
- **Universal Messaging:** Self-hosted webMethods messaging broker
- **External:** JMS connectors for IBM MQ, Apache Kafka, etc.
"#;

const FSL_LANGUAGE_REF: &str = r#"# FSL (Flow Service Language) Reference

FSL is a text-based DSL for authoring flow services, used with the
`dsl_validate` / `fsl_deploy` / `fsl_extract` tools as an alternative to
building a service field-by-field via `put_node`. `dsl_validate` catches
syntax errors before touching the server, which makes FSL pleasant for
services made of scalar mappings, INVOKEs, BRANCH/IF and TRY/CATCH.

## When NOT to use FSL (IS 12.1 compiler limitations, verified)

The compiler (`wm.server.flowGen`, IBM code on the IS side) accepts and
"deploys" constructs it does not actually emit. `dsl_validate` says SUCCESS,
`fsl_deploy` says SUCCESS, and the flow is missing steps:

- **Document-list copies are dropped** when the list variable is not declared
  in the service `input {}`: `copy selectOutput/results -> reps;` followed by
  `copy reps -> insertInput/inputs;` compiles to an EMPTY input MAP (the
  variable is typed `;2;0`, a single record). It only works when the list is
  part of the service signature.
- **`WHILE (...) { ... }` compiles to nothing** -- no LOOP/RETRY is emitted,
  the body is lost.
- **`date` is a reserved word**: `INVOKE pub.date:getCurrentDateString` is a
  parse error (`mismatched input 'date'`), and the documented backtick escape
  ``pub.`date`:getCurrentDateString`` is compiled with the backticks kept,
  so the service fails at run time with `[ISC.0049.9010] unknown service`.
  The same applies to a field named `date` (`pub.scheduler:addOneTimeTask`).
  `fsl_extract` emits `pub.date:` unescaped, so its output is not
  re-validatable (no round trip). Expect the same class of problem with other
  keywords used as names (`time`, `type`, `value`, `pattern`).

Use `put_node` for anything with document lists, loops over records, RETRY,
or `pub.date` / `pub.scheduler` calls -- its JSON is verbose but every step
lands, the tree is validated before writing, and the stored step counts are
verified afterwards. After any `fsl_deploy`, read the service back with
`node_get` (it returns the real step tree) and check that every `copy`,
LOOP and RETRY you wrote is present before invoking it.

## Workflow

1. `package_create` the target package if it doesn't already exist (or
   confirm with `package_list` / `package_info`) -- `fsl_deploy` does NOT
   create the package for you.
2. Write FSL source text (see structure below).
3. `dsl_validate` it -- returns `status` (SUCCESS/FAILED), `validationErrors`,
   `errorCount`. Iterate until clean; this is a pure syntax check, no server
   mutation.
4. `fsl_deploy` it with `package_name`, `ifc_name` (folder path), and
   `flow_name`. This compiles AND deploys in one atomic call. The compiled
   node returned in the response is informational only; do not repost it via
   `put_node` (its `flow.nodes` is flattened to a string).
5. Verify with `node_get` (count the MAPCOPY / LOOP / RETRY steps in
   `flow.nodes` against your source -- see the limitations above) and a
   `service_invoke` test call. `fsl_extract` can
   decompile an existing service back to FSL (useful to see the canonical
   form of something built via `put_node`, or to diff after a redeploy) --
   the round-tripped FSL is semantically equivalent but not always textually
   identical to the original source.

Deploying a `.flow` via `fsl_deploy` does NOT deploy any IS Unit Test suite
associated with it -- test suite files are a separate, filesystem-level
concern outside what this HTTP-only tool surface can reach.

## File Structure

Every FSL file starts with an interface declaration (folder path ONLY, never
the package name -- the package goes in `fsl_deploy`'s `package_name` param,
not in the source text), followed by the service declaration and body.

```fsl
interface orders.receive

service processOrder (
    input {
        String customerId;
        Double[] priceList;
    }
    output {
        String status;
    }
)
properties {
    comment: "Validates and totals an incoming order.";
    visible: private;
}
{
    MAP {
        mapTarget {
            String status;
        }
        set status = "PENDING";
    }
}
```

- No semicolon after the `interface` line.
- `service` declares the name ONLY -- no package, no folder path.
- The `properties { ... }` block (comment, visible, prefetch, validateInput,
  validateOutput, ...) goes AFTER the signature and BEFORE the body's `{`.
  Every property line inside `properties { }` ends with `;`.
- The body is the last top-level `{ ... }` block, containing the actual steps
  (INVOKE, MAP, BRANCH, LOOP, IF, TRY, EXIT, ...).
- No markdown fences, no conversational text -- FSL is raw plain text.

## copy vs set (do not confuse these)

- **`copy source -> target;`** -- moves a value already in the pipeline.
- **`set target = "literal";`** -- assigns a hardcoded literal.
- Never `<-`. Never `set "value" -> target;`. Never bare `target = "value";`
  without `set`. Never `set` and `copy` the same target in the same block --
  IS runs statements in order, so whichever runs second silently wins and the
  other becomes dead code.
- Record/recordList literals are whole-block JSON, never partial slash paths:
  `set items = [{"sku": "A1", "qty": "1"}];` -- NOT
  `set items/sku = "A1";` for an initial assignment.

## Reserved keywords need backticks

If a pipeline variable or schema field name collides with an FSL keyword, it
must be backtick-escaped wherever it's declared or referenced as an
identifier: `pattern`, `value`, `properties`, `type`, `service`, `interface`,
`input`, `output`, `record`, `recordList`, `document`, `branch`, `sequence`,
`loop`, `map`, `mapSource`, `mapTarget`, `copy`, `set`, `drop`.

This shows up constantly on `pub.math:*`/`pub.string:*` output mappings,
whose return parameter is literally named `value`:

```fsl
INVOKE pub.math:addInts {
    input {
        mapSource {
            String a;
            String b;
        }
        mapTarget {
            String num1;
            String num2;
        }
        copy a -> num1;
        copy b -> num2;
    }
    output {
        mapSource {
            String `value`;
        }
        mapTarget {
            String sum;
        }
        copy `value` -> sum;
    }
}
```

Only the reserved leaf name gets backticks -- non-reserved parent path
segments stay unescaped.

## Semicolon placement (the #1 source of parser errors)

- **Statements get a semicolon:** every `set`/`copy`/`drop`, every field
  declaration (`String x;`), every property line inside a signature field's
  `{ }` block or a `properties { }` block.
- **Step-level properties do NOT get a semicolon:** `comment:`,
  `validateInput:`, `invoke-order:`, `switch:`, `inputArray:` etc. when they
  appear as the first lines inside a flow step block (MAP, INVOKE, BRANCH,
  LOOP, ...). `MAP { comment: "x" }` -- correct. `MAP { comment: "x"; }` --
  parser error.
- **Closing braces never get a semicolon**, for ANY block: MAP, INVOKE,
  SEQUENCE, BRANCH, IF/ELSEIF/ELSE, LOOP, WHILE, REPEAT, DO/UNTIL, TRY/CATCH/
  FINALLY, EXIT, TRANSFORM. `SEQUENCE { ... };` is always wrong.
- `BREAK` and `CONTINUE` are standalone -- no trailing semicolon:
  `IF (%y% >= 15) { BREAK }`, not `{ BREAK; }`.
- No single-line multi-statement blocks. One declaration/statement/brace per
  line, nested structures indented -- this isn't just style, dense one-liners
  are a common source of "extraneous input" parse errors.

## Property ordering inside a step

Inside any step block, properties must appear before nested steps, in this
order: general (`comment`, `scope`, `timeout`, `label`) first, then
block-specific (`exitOn` for SEQUENCE/TRY/CATCH/FINALLY, `switch`/
`evaluateLabels` for BRANCH, `inputArray` for LOOP, `count`/`repeatInterval`/
`repeatOn` for REPEAT), then child steps (MAP, INVOKE, IF, ...). Putting a
child step before a property causes an "extraneous input" error.

## EXIT and TRY/CATCH (failure paths)

`EXIT` immediately halts the flow with a signal. `exitFrom: "$flow"` +
`signal: "FAILURE"` is how a service deliberately fails with a message an
IS Unit Test suite can assert against (see the FSL example test-suite note
below):

```fsl
IF (%amount% < 0) {
    comment: "Business rule: amount must be non-negative"
    EXIT {
        exitFrom: "$flow"
        signal: "FAILURE"
        failureMessage: "Amount must be greater than or equal to zero"
    }
}
```

A test case in an IS Unit Test suite that expects this EXIT to fire must use
`<expected><exception class="com.wm.app.b2b.client.ServiceException"
message="Amount must be greater than or equal to zero"/></expected>` in its
`webMethodsTestCase` -- not a `<file>` IData comparison, which only applies
to normal successful output.

`TRY`/`CATCH` are adjacent sibling blocks (not nested), matching putNode's
model: put the risky steps in `TRY { }`, recovery/rethrow logic in
`CATCH { }` immediately after. To propagate the failure out of the current
scope from inside CATCH, use `EXIT { exitFrom: "$parent" signal: "FAILURE" }`.

```fsl
TRY {
    INVOKE risky:operation {}
}
CATCH {
    EXIT {
        exitFrom: "$parent"
        signal: "FAILURE"
        failureMessage: "risky:operation failed"
    }
}
```

## INVOKE vs MAP+TRANSFORM

- Standalone service call, nothing else happening in that step -> `INVOKE
  service:name { input { ... } output { ... } }` at the top level.
- Service call bundled with other pipeline work (extra `set`/`copy`/`drop` in
  the same logical step) -> wrap it as `MAP { TRANSFORM service:name { ... } }`.
  Inside an active `MAP` block, `INVOKE` is never valid -- only `TRANSFORM` is.
- A `MAP` block is either pure direct assignments (`mapSource`/`mapTarget` +
  `set`/`copy` at the MAP's own root, no nested TRANSFORM) OR a pure
  TRANSFORM wrapper (MAP has no root-level `mapSource`/`mapTarget`, all
  mapping lives inside the child TRANSFORM's `input`/`output`). Never mix
  both patterns in the same MAP.

## System variables

`$retries` (REPEAT), `$iterationCount` (DO/WHILE/LOOP), `$null`, `$flow`,
`$default` are implicitly available in their respective contexts. Never
declare them in `mapSource`/`mapTarget` -- just reference them directly, e.g.
`copy repeatVals[$retries] -> target;`.

## Type preservation with catalog services

When mapping into an `INVOKE`/`TRANSFORM` target, the CALLED SERVICE's
declared parameter type always wins over the pipeline source variable's
type -- both `mapTarget` and the calling service's own signature must match
it end-to-end, or the parameter silently arrives as null and the service
throws "Missing Parameter" at runtime. E.g. `pub.math:addInts` takes String
`num1`/`num2` even though it's adding numbers -- an `Integer` pipeline
variable must still be declared `String` in `mapTarget`. Only fall back to
matching the source variable's own type when the target service imposes no
type constraint.
"#;

const UNIT_TEST_REF: &str = r#"# Unit Test Framework Reference (authoring, running, reading results)

## What it is

The official IS unit-testing stack, formerly *WmTestSuite*, is the **Unit Test
Framework** (IBM still calls the server part "Integration Test Suite"):

| Piece | Role | Reachable from this server? |
|---|---|---|
| Designer plugin `com.sag.gcs.wmtestsuite` | GUI to author suites, "Generate Tests" from a service run, run/debug them | No (Eclipse UI, no API). `test_suite_create` writes the same files. |
| IS system package `WmUnitTestManager` (IS 11.1+) | service mocks (`wm.ps.serviceMock`), runner (`wm.task.executor`), suite discovery (`wm.task.asset`), REST API `/admin/utf/...`, web UI `/WmUnitTestManager/` | Yes: `test_run`, `test_check_status`, `test_report`, `test_text_report`, `test_junit_report`, `test_suite_list`, `test_suite_get`, `mock_*` |
| Test Suite Executor (Ant project generated by Designer) | headless runs in CI, JUnit XML + HTML + coverage | No; `test_run` produces the same JUnit XML server-side |

A suite is an XML file (`<webMethodsTestSuite>`) inside the package under test;
each `<webMethodsTestCase>` invokes one service with an input pipeline file and
checks the output pipeline (file, field assertions, or expected exception),
optionally with mocks. Pipeline files use the IDataXMLCoder format
(`pub.flow:savePipelineToFile`). Any `*.xml` under the package directory that
validates as a suite is discovered; Designer's convention is
`resources/test/setup/<suite>.xml` with data files in `resources/test/data/`.

## Prerequisites (check before anything else)

- `WmUnitTestManager` enabled (`package_info`). On IS 11.1/12.1 it ships with
  the server.
- The Unit Test Framework client JARs in `<IS_HOME>/common/lib/testsuite/`
  (junit, serviceMockClient, serviceInterceptor, wmjxpath, xmlunit, httpunit,
  hamcrest, commons-jxpath, xml-soap-api). They are installed with Designer,
  NOT by the IS installer: on a Linux-only IS `test_run` fails with
  `common/lib/testsuite does not exist`, and field assertions error with
  `NoClassDefFoundError com.wm.ps.jxpath.IDataJXPathContext` until the 9 JARs
  are copied from a Designer installation.
- To write suites, the MCP server must reach the IS packages directory on
  disk, or the IS must allow `pub.file:stringToFile` there (extended setting
  `watt.server.file.canWritePaths`; the error `[ISS.0086.9263]` names it).

## Workflow

1. Build the service (`put_node` / `fsl_deploy`) and check it with
   `service_invoke`.
2. `test_suite_create` -- package, suite_name, `tests` (JSON array, below).
3. `test_suite_list` -- confirm the suite and its cases were discovered.
4. `test_run` with `test_suite_packages` = `["<Package>"]`. The call blocks
   until the run completes (`waitForReport` defaults to true on the IS) and
   returns `executionID`, `status` and the counts (`testCount`,
   `failureCount`, `errorCount`).
5. `test_report` (readable verdict + one table per suite with each case's
   result and failure message; `format: "json"` for the structured summary),
   `test_text_report` (raw text with stack traces) or `test_junit_report`
   (JUnit XML for CI). All need status COMPLETED.

## `tests` JSON specification

```json
[
  {"name": "greetAlice", "description": "full pipeline compare",
   "service": "utfdemo.services:greet",
   "input": {"name": "Alice"},
   "expected": {"greeting": "Hello, Alice"}},

  {"name": "defaultName", "service": "utfdemo.services:greet",
   "expected_fields": [{"path": "/greeting", "operator": "==", "value": "Hello, World"}]},

  {"name": "rejectsNegative", "service": "orders.api:validate",
   "input": {"amount": "-1"},
   "expected_exception": {"message": "Amount must be greater than or equal to zero"}},

  {"name": "snapshot", "service": "utfdemo.services:greet",
   "input": {"name": "Bob"}, "record": true},

  {"name": "isolatedFromHttp", "service": "orders.api:fetch",
   "input": {"id": "42"},
   "expected_fields": [{"path": "/status", "value": "OK"}, {"path": "/order/total", "operator": ">", "value": 0, "logical": "and"}],
   "mocks": [
     {"service": "pub.client:http", "pipeline": {"status": "200", "body": {"string": "{\"total\":10}"}}},
     {"service": "orders.db:insert", "alternate_service": "orders.mocks:insertOk", "parms": {"trace": "true"}},
     {"service": "pub.client:smtp", "exception": {"class": "java.io.IOException", "message": "mail down"}, "scope": "server", "lifetime": "suite"}
   ]}
]
```

Per test case:

- `service` -- `folder.sub:name` of the service under test.
- `input` -- JSON object written as `<test>_input.xml`. Omit for no input.
- `expected` -- JSON object written as `<test>_expected.xml`. **Subset
  match**: every expected variable must exist in the actual output with an
  equal value; extra actual variables are ignored (so listing only the
  declared outputs is enough).
- `expected_fields` -- assertions on the actual output: `path` is a JXPath
  (`/greeting`, `/order/lines[1]/qty`), `operator` one of `==` (default)
  `!=` `>` `>=` `<` `<=`, `value` a string/number/boolean literal (omit it to
  compare with the same path of the `expected` pipeline), `logical` joins with
  the previous field (`and` default, `or`, `and not`, `or not`),
  `start_paren`/`end_paren` for grouping. **When fields are present the
  validator evaluates only the fields**: the `expected` pipeline is then just
  the reference for fields given without `value`, not a second comparison.
  To check several variables at once, put them all in `expected` (subset
  match) or list them all as fields.
- `expected_exception` -- `{class, message}`; the test passes when the
  service throws an exception whose text contains the class name and the
  message (substring or regex). `class` defaults to
  `com.wm.app.b2b.client.ServiceException`, which is what a failing flow
  service raises in the JUnit runner. Cannot be combined with `expected` /
  `expected_fields`.
- `record: true` -- Designer's "Generate Tests": the service is invoked now
  with `input`; its declared outputs (per the service signature) become
  `expected`, or, if it fails, the failure becomes `expected_exception`.
  The result JSON lists what was recorded -- check it. The recording runs
  without mocks, so `record` is refused on a test that declares `mocks`.
- `mocks` -- each replaces `service` for the duration of the test (or the rest
  of the suite with `lifetime: "suite"`) with exactly one of: `pipeline`
  (fixed output), `alternate_service` (invoked with the same pipeline, plus
  `parms` merged in), `exception`. `scope` defaults to `session`, which is the
  right value inside a suite (the runner keeps one session for the whole run).
- `description`, `enabled` (false = skipped), `mocks_enabled` (false =
  ignore this test's mocks), `comparator_service` (a service implementing the
  spec `wm.spec:result_comparator`: inputs `actualData`, `expectedData`,
  outputs `result` boolean + `exceptionMessage`), `request_method`
  (`invoke` default; `post`/`get` test an HTTP endpoint with httpunit, then
  `input` is the request body and `expected` a file compared byte for byte).

Pipelines are plain JSON objects. Strings stay Strings, objects become
documents, arrays of strings/objects become String[] / IData[], numbers and
booleans keep Java types (`java.lang.Long`, `java.lang.Double`,
`java.lang.Boolean`) exactly as `service_invoke` sends them -- so give flow
String fields JSON strings (`"5"`, not `5`): a MAPCOPY into a String field
silently drops a Long, and the test then sees `null`. `test_suite_create`
lists every such value under `warnings`; treat them as mistakes unless the
field really is an Object.

## What gets written (Designer-compatible)

```
<Package>/resources/test/setup/<suite_name>.xml
<Package>/resources/test/data/<suite_name>/<test>_input.xml
<Package>/resources/test/data/<suite_name>/<test>_expected.xml
<Package>/resources/test/data/<suite_name>/<test>_mock<N>_<service>.xml        (mock pipeline)
<Package>/resources/test/data/<suite_name>/<test>_mock<N>_<service>_parms.xml  (alternate-service parms)
```

```xml
<?xml version="1.0" encoding="UTF-8"?>
<webMethodsTestSuite name="GreetSuite" description="">
    <webMethodsTestCase description="" name="greetAlice">
        <mock folder="pub.string" name="concat">
            <pipeline filename="resources/test/data/GreetSuite/greetAlice_mock1_pub.string_concat.xml"/>
        </mock>
        <service folder="utfdemo.services" name="greet">
            <input>
                <file filename="resources/test/data/GreetSuite/greetAlice_input.xml"/>
            </input>
            <expected>
                <file filename="resources/test/data/GreetSuite/greetAlice_expected.xml"/>
                <field path="/greeting" operator="==" value="Hello, Alice"/>
            </expected>
        </service>
    </webMethodsTestCase>
</webMethodsTestSuite>
```

Attributes equal to their defaults (`enabled="true"`, `mocksEnabled="true"`,
`scope="session"`, `lifetime="test"`, `requestMethod="invoke"`) are omitted,
as Designer's serializer does. File names are relative to the package
directory. Designer opens these suites; `mode: "append"` also adds cases to a
suite Designer created (even the empty `<webMethodsTestSuite/>`).

A pipeline file:

```xml
<?xml version="1.0" encoding="UTF-8"?>

<IDataXMLCoder version="1.0">
  <record javaclass="com.wm.data.ISMemDataImpl">
    <value name="name">Alice</value>
    <record name="order" javaclass="com.wm.data.ISMemDataImpl">
      <value name="id">42</value>
    </record>
    <array name="tags" type="value" depth="1">
      <value>a</value>
      <value>b</value>
    </array>
  </record>
</IDataXMLCoder>
```

## Service mocks outside suites (`mock_load`)

`mock_load {service, mock_object (alternate service), scope}` intercepts a
service for the whole IS. Scopes: `server` (default -- the only one that
survives across MCP calls, because every call is a new IS session), `user`,
`session` (useless through MCP). `mock_list` shows active mocks per scope,
`mock_clear` needs the same scope as the load, `mock_clear_all` removes
everything. Clean up: server-scoped mocks affect every caller of the IS.

## Gotchas

- A `test_run` on a package that has **no** suite (or an unknown package
  name) silently runs every suite of the server and reports their counts under
  the requested name -- confirm with `test_suite_list` first.
- `test_run` blocks until the run ends; the MCP HTTP timeout is 30 s by
  default. Keep suites small or raise the instance timeout.
- `test_check_status` values: NEW, INVOKED, INPROGRESS, COMPLETED, ERROR,
  UNAVAILABLE (unknown executionID). Reports exist only after COMPLETED; an
  unknown or running executionID yields `Report ... unavailable, or not
  generated`.
- The JUnit XML from `test_junit_report` has its `<properties>` block (JVM
  properties, ~100 KB) stripped unless `include_properties` is true.
- Field paths are evaluated with JXPath on the actual output pipeline; a
  missing variable makes the comparison fail, it does not raise.
- `record: true` captures whatever the service returns today, including bugs:
  read the recorded values in the response before trusting the test.
"#;
