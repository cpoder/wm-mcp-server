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
    DocResource {
        uri: "wm://docs/flow-steps-reference",
        name: "Flow Steps Reference (IBM Docs)",
        description: "Official IBM documentation for all webMethods flow step types: INVOKE, BRANCH, LOOP, MAP, SEQUENCE, REPEAT, EXIT. Properties, behavior rules, failure conditions, and data mapping concepts.",
        content: FLOW_STEPS_REF,
    },
    DocResource {
        uri: "wm://docs/builtin-services",
        name: "Built-In Services Reference",
        description: "Input/output signatures for commonly used IS built-in services: pub.string (concat, replace, substring, etc.), pub.math (addInts, multiplyFloats, etc.), pub.list (appendToDocumentList, etc.), pub.date (formatDate, etc.), pub.flow (debugLog, getLastError, etc.).",
        content: BUILTIN_SERVICES_REF,
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

Exit-on values: `FAILURE` (stop on first failure, default), `SUCCESS` (stop on first success), `DONE` (run all regardless).

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

## Example 4: Nested record mapping (adapter service output extraction)

This pattern: call an adapter service, extract nested fields from the result into a flat output.
Uses type 2 (Record) with nested paths -- NO LOOP needed for single-record access.

```json
{
  "flow": {"type":"ROOT","version":"3.0","cleanup":"true","nodes":[
    {"type":"INVOKE","service":"mypkg.adapters:getAccountDetails","validate-in":"$none","validate-out":"$none",
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
  {"type":"MAPCOPY","from":"/selectOrdersOutput;2;0/results;2;0/order_id;1;0","to":"/orders;4;0;mypkg.docTypes:OrderCanonical/id;1;0"},
  {"type":"MAPCOPY","from":"/selectOrdersOutput;2;0/results;2;0/order_date;1;0","to":"/orders;4;0;mypkg.docTypes:OrderCanonical/date;1;0"},
  {"type":"MAPCOPY","from":"/selectOrdersOutput;2;0/results;2;0/status;1;0","to":"/orders;4;0;mypkg.docTypes:OrderCanonical/status;1;0"},
  {"type":"MAPCOPY","from":"/selectOrdersOutput;2;0/results;2;0/customer_id;1;0","to":"/orders;4;0;mypkg.docTypes:OrderCanonical/customer;2;0/id;1;0"},
  {"type":"MAPCOPY","from":"/selectOrdersOutput;2;0/results;2;0/customer_name;1;0","to":"/orders;4;0;mypkg.docTypes:OrderCanonical/customer;2;0/name;1;0"},
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
    {"type":"MAPCOPY","from":"/JMSMessage;4;0;pub.jms:JMSMessage/body;2;0/data;2;0","to":"/order;4;0;mypkg.docTypes:OrderCanonical"},
    {"type":"MAPDELETE","field":"/JMSMessage;4;0;pub.jms:JMSMessage"}
  ]},
  {"type":"INVOKE","service":"mypkg.jdbc:createOrder","validate-in":"$none","validate-out":"$none","nodes":[
    {"type":"MAP","mode":"INPUT","nodes":[
      {"type":"MAPCOPY","from":"/order;4;0;mypkg.docTypes:OrderCanonical/id;1;0","to":"/createOrderInput;2;0/order_id;1;0"},
      {"type":"MAPCOPY","from":"/order;4;0;mypkg.docTypes:OrderCanonical/status;1;0","to":"/createOrderInput;2;0/status;1;0"}
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
      {"type":"MAPCOPY","from":"/request;4;0;mypkg.docTypes:OrderRequest","to":"/document;2;0"}
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
       "data":"<Values version=\"2.0\"><value name=\"xml\">mypkg.client:apiDescriptor</value></Values>"}
    ]}
  ]},
  {"type":"BRANCH","switch":"","label-expressions":"true","nodes":[
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
  "node_nsName": "mypkg.services:getResource",
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
              {"type": "INVOKE", "service": "mypkg.impl:fetchResource", "validate-in": "$none", "validate-out": "$none"},
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
                    "data": "<Values version=\"2.0\"><value name=\"xml\">mypkg.services:getResource</value></Values>"},
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
Converts object via Java toString().
- **In:** `object` (Object, req)
- **Out:** `value` (String)

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
Appends documents to a document list. Appends references, not copies.
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

## pub.json (JSON Processing)

### pub.json:documentToJSON
Converts an IData document to a JSON string.
- **In:** `document` (Document, req), `jsonStream` (opt - if true, returns OutputStream)
- **Out:** `jsonString` (String - the JSON representation)
- **Note:** Commonly used in CATCH blocks to serialize error documents for API responses.

### pub.json:jsonStringToDocument
Parses a JSON string into an IData document.
- **In:** `jsonString` (String, req)
- **Out:** `document` (Document - the parsed IData)

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
