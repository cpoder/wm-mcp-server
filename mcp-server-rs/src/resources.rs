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
        description: "Complete reference for webMethods flow service development via putNode API: step types, WmPath format, mapping rules, LOOP patterns, and working examples.",
        content: FLOW_LANGUAGE_REF,
    },
    DocResource {
        uri: "wm://docs/putnode-examples",
        name: "putNode Working Examples",
        description: "Tested, working putNode JSON examples for common flow service patterns: simple service, INVOKE with mappings, LOOP over records, BRANCH, record array handling.",
        content: PUTNODE_EXAMPLES,
    },
    DocResource {
        uri: "wm://docs/adapter-service-reference",
        name: "Adapter Service Configuration Reference",
        description: "How to create JDBC adapter services with full table/column configuration. Select, Insert, CustomSQL examples with correct adapter_service_settings JSON.",
        content: ADAPTER_SERVICE_REF,
    },
];

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
- `/accounts;4;1;mypkg.doctypes:account` -- typed record array (RecordRef to doc type)
- `/accounts;4;0;mypkg.doctypes:account/customerName;1;0` -- field inside current LOOP iteration element

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
  "field": "/accounts;4;1;mypkg.doctypes:account",
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
    {"type": "SEQUENCE", "label": "$default", "nodes": [/* default steps */]}
  ]
}
```

### SEQUENCE
Group steps with a label and exit condition.
```json
{"type": "SEQUENCE", "label": "myLabel", "exit-on": "FAILURE", "nodes": [/* steps */]}
```

### EXIT
Exit from the current flow, loop, or sequence.
```json
{"type": "EXIT", "from": "$flow", "signal": "FAILURE"}
```
`from` values: `$flow`, `$loop`, `$parent` (exit sequence)

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
  "rec_ref": "mypkg.doctypes:account",
  "rec_closed": "true"
}
```

## Document Type Creation

Before using RecordRef fields, create the document type:
```json
{
  "node_nsName": "mypkg.doctypes:account",
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
"#;

const PUTNODE_EXAMPLES: &str = r#"# putNode Working Examples

All examples below have been tested and verified on IS 11.1.

## Example 1: Simple service with MAPSET default value

```json
{
  "node_nsName": "mypkg.services:greet",
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
  "node_nsName": "mypkg.doctypes:account",
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
  "node_nsName": "mypkg.services:searchAccounts",
  "node_pkg": "MyPackage",
  "node_type": "service",
  "svc_type": "flow", "svc_subtype": "default", "svc_sigtype": "java 3.5",
  "stateless": "yes", "pipeline_option": 1,
  "svc_sig": {
    "sig_in": {"node_type":"record","field_type":"record","field_dim":"0","nillable":"true","javaclass":"com.wm.util.Values","rec_fields":[]},
    "sig_out": {"node_type":"record","field_type":"record","field_dim":"0","nillable":"true","javaclass":"com.wm.util.Values",
      "rec_fields":[
        {"node_type":"record","field_name":"accounts","field_type":"recref","field_dim":"1","nillable":"true","rec_ref":"mypkg.doctypes:account","rec_closed":"true"}
      ]
    }
  },
  "flow": {"type":"ROOT","version":"3.2","cleanup":"true","nodes":[
    {"type":"MAP","mode":"STANDALONE","nodes":[
      {"type":"MAPSET","field":"/accounts;4;1;mypkg.doctypes:account","overwrite":"true","d_enc":"XMLValues","mapseti18n":"true",
       "data":"<Values version=\"2.0\"><array name=\"xml\" type=\"record\" depth=\"1\"><record javaclass=\"com.wm.util.Values\"><value name=\"accountName\">acc1</value><value name=\"customerName\">Alice</value></record><record javaclass=\"com.wm.util.Values\"><value name=\"accountName\">acc2</value><value name=\"customerName\">Bob</value></record></array></Values>"}
    ]}
  ]}
}
```

### Step 3: Main service with LOOP
```json
{
  "node_nsName": "mypkg.services:getCustomers",
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
    {"type":"INVOKE","service":"mypkg.services:searchAccounts","validate-in":"$none","validate-out":"$none"},
    {"type":"LOOP","in-array":"/accounts","out-array":"/customers","nodes":[
      {"type":"MAP","mode":"STANDALONE","nodes":[
        {"type":"MAPCOPY","from":"/accounts;4;0;mypkg.doctypes:account/customerName;1;0","to":"/customers;1;0"}
      ]}
    ]},
    {"type":"MAP","mode":"STANDALONE","nodes":[
      {"type":"MAPDELETE","field":"/accounts;4;1;mypkg.doctypes:account"}
    ]}
  ]}
}
```

**Key points:**
- MAPCOPY from path uses type 4 (RecordRef): `/accounts;4;0;mypkg.doctypes:account/customerName;1;0`
- Dimension is 0 (current iteration element, not the array)
- Document type reference is required after the dimension
- MAPDELETE after LOOP cleans up the temporary array from pipeline output

## Example 3: BRANCH (conditional logic)

```json
{
  "flow": {"type":"ROOT","version":"3.0","cleanup":"true","nodes":[
    {"type":"BRANCH","switch":"/action","nodes":[
      {"type":"SEQUENCE","label":"create","exit-on":"FAILURE","nodes":[
        {"type":"INVOKE","service":"mypkg.services:createRecord","validate-in":"$none","validate-out":"$none"}
      ]},
      {"type":"SEQUENCE","label":"delete","exit-on":"FAILURE","nodes":[
        {"type":"INVOKE","service":"mypkg.services:deleteRecord","validate-in":"$none","validate-out":"$none"}
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
"#;

const ADAPTER_SERVICE_REF: &str = r#"# Adapter Service Configuration Reference

## Creating JDBC Adapter Services

### Workflow
1. Use `adapter_resource_domain_lookup` to browse database objects:
   - `catalogNames` -> pick catalog
   - `schemaNames` (values: [catalog]) -> pick schema
   - `tableNames` (values: [catalog, schema]) -> pick table
   - `columnInfo` (values: [catalog, schema, table]) -> get columns
2. Build `adapter_service_settings` JSON from the column metadata
3. Call `adapter_service_create` with the settings

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
"#;
