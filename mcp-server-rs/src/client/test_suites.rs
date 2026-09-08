//! Unit Test Framework suite authoring.
//!
//! Generates what Designer's plugin (`com.sag.gcs.wmtestsuite`) writes when a
//! developer creates a test suite or uses "Generate Tests" on a service run:
//! a `webMethodsTestSuite` XML file plus IDataXMLCoder pipeline files inside
//! the package. The XML element/attribute rules mirror
//! `com.wm.ps.test.model.{TestSuiteData,TestData,ServiceData,MockData}`
//! (`convertToDomElement`), the pipeline format mirrors
//! `com.wm.util.coder.IDataXMLCoder`, and the discovery rule of
//! `WmTestSuiteUtils.searchTestSuitesInProject` (any `*.xml` under the
//! package that validates as a suite) is what `test_run` relies on.

use serde::Deserialize;
use serde_json::{Map, Value, json};

use super::ISClient;

/// Folder (inside the package) where Designer's "New webMethods Test Suite"
/// wizard proposes to put suites; the data folder next to it holds pipelines.
pub const SUITE_DIR: &str = "resources/test/setup";
pub const DATA_DIR: &str = "resources/test/data";

/// Exception class thrown to the JUnit runner (a `Context.invoke` client
/// call) when a flow service fails; used when `expected_exception` gives a
/// message but no class, because the validator dereferences the class name.
pub const DEFAULT_EXPECTED_EXCEPTION_CLASS: &str = "com.wm.app.b2b.client.ServiceException";

// ───────────────────────── XML helpers ─────────────────────────────

fn esc_text(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            _ => o.push(ch),
        }
    }
    o
}

fn esc_attr(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\n' => o.push_str("&#10;"),
            _ => o.push(ch),
        }
    }
    o
}

fn attr(out: &mut String, name: &str, value: &str) {
    out.push(' ');
    out.push_str(name);
    out.push_str("=\"");
    out.push_str(&esc_attr(value));
    out.push('"');
}

/// File-name safe version of a suite / test / service name.
pub fn sanitize(name: &str) -> String {
    let s: String = name
        .trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let s = s.trim_matches('.').to_string();
    if s.is_empty() {
        "unnamed".to_string()
    } else {
        s
    }
}

/// Split `folder.sub:service` into (`folder.sub`, `service`).
pub fn split_service(full: &str) -> Result<(&str, &str), String> {
    match full.trim().split_once(':') {
        Some((f, n)) if !f.is_empty() && !n.is_empty() && !n.contains(':') => Ok((f, n)),
        _ => Err(format!(
            "'{full}' is not a fully qualified service name (expected folder.subfolder:serviceName)"
        )),
    }
}

// ───────────────────────── IDataXMLCoder ───────────────────────────

/// Encode a JSON object as an IDataXMLCoder document, the pipeline file
/// format read by the Unit Test Framework (and by pub.flow:restorePipelineFromFile).
///
/// JSON strings become String values, objects become documents, arrays of
/// strings/objects become String[] / IData[] (2-D when nested), numbers and
/// booleans keep their Java types (`java.lang.Long`, `java.lang.Double`,
/// `java.lang.Boolean`) -- exactly what the IS JSON content handler does for
/// service_invoke. Flow services usually expect String fields: pass `"5"`,
/// not `5`, unless the field really is an Object.
pub fn idata_xml(pipeline: &Value) -> Result<String, String> {
    let obj = pipeline.as_object().ok_or_else(|| {
        "a pipeline must be a JSON object (its keys are the pipeline variables)".to_string()
    })?;
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\n<IDataXMLCoder version=\"1.0\">\n  <record javaclass=\"com.wm.data.ISMemDataImpl\">\n",
    );
    encode_fields(obj, 2, &mut out);
    out.push_str("  </record>\n</IDataXMLCoder>\n");
    Ok(out)
}

fn encode_fields(obj: &Map<String, Value>, level: usize, out: &mut String) {
    for (k, v) in obj {
        encode_value(Some(k), v, level, out);
    }
}

fn name_attr(name: Option<&str>) -> String {
    match name {
        Some(n) => format!(" name=\"{}\"", esc_attr(n)),
        None => String::new(),
    }
}

fn encode_value(name: Option<&str>, v: &Value, level: usize, out: &mut String) {
    let pad = "  ".repeat(level);
    let n = name_attr(name);
    match v {
        Value::Null => out.push_str(&format!("{pad}<null{n}/>\n")),
        Value::Bool(b) => out.push_str(&format!("{pad}<jboolean{n}>{b}</jboolean>\n")),
        Value::Number(num) => {
            if num.is_i64() || num.is_u64() {
                out.push_str(&format!(
                    "{pad}<number{n} type=\"java.lang.Long\">{num}</number>\n"
                ));
            } else {
                out.push_str(&format!(
                    "{pad}<double{n} type=\"java.lang.Double\">{num}</double>\n"
                ));
            }
        }
        Value::String(s) => out.push_str(&format!("{pad}<value{n}>{}</value>\n", esc_text(s))),
        Value::Object(o) => {
            out.push_str(&format!(
                "{pad}<record{n} javaclass=\"com.wm.data.ISMemDataImpl\">\n"
            ));
            encode_fields(o, level + 1, out);
            out.push_str(&format!("{pad}</record>\n"));
        }
        Value::Array(items) => {
            let (ty, depth, javaclass) = array_shape(items);
            let jc = javaclass
                .map(|c| format!(" javaclass=\"{c}\""))
                .unwrap_or_default();
            out.push_str(&format!(
                "{pad}<array{n} type=\"{ty}\" depth=\"{depth}\"{jc}>\n"
            ));
            for item in items {
                encode_value(None, item, level + 1, out);
            }
            out.push_str(&format!("{pad}</array>\n"));
        }
    }
}

/// (`type` attribute, `depth` attribute, optional `javaclass`) of an array,
/// following what the IS itself emits: String[] -> value/1, IData[] ->
/// record/1, String[][] -> value/2, Long[] -> object/1 + javaclass, mixed ->
/// object/1.
fn array_shape(items: &[Value]) -> (&'static str, usize, Option<&'static str>) {
    if items.is_empty() {
        return ("value", 1, None);
    }
    if items.iter().all(Value::is_string) {
        return ("value", 1, None);
    }
    if items.iter().all(Value::is_object) {
        return ("record", 1, None);
    }
    if items.iter().all(Value::is_array) {
        let shapes: Vec<_> = items
            .iter()
            .map(|i| array_shape(i.as_array().expect("checked")))
            .collect();
        let (ty, depth, _) = shapes[0];
        let uniform = shapes.iter().all(|s| s.0 == ty && s.1 == depth);
        if uniform && (ty == "value" || ty == "record") {
            return (ty, depth + 1, None);
        }
        return ("object", 1, None);
    }
    if items
        .iter()
        .all(|i| i.as_i64().is_some() || i.as_u64().is_some())
    {
        return ("object", 1, Some("java.lang.Long"));
    }
    if items.iter().all(Value::is_number) {
        return ("object", 1, Some("java.lang.Double"));
    }
    if items.iter().all(Value::is_boolean) {
        return ("object", 1, Some("java.lang.Boolean"));
    }
    ("object", 1, None)
}

// ───────────────────────── suite specification ─────────────────────

/// One `webMethodsTestCase`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestCaseSpec {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Service under test, `folder.sub:name`.
    pub service: String,
    /// Input pipeline (JSON object) -> `<test>_input.xml`.
    #[serde(default)]
    pub input: Option<Value>,
    /// Expected output pipeline (JSON object) -> `<test>_expected.xml`.
    /// Compared as a subset: every expected variable must exist in the actual
    /// output with an equal value; extra actual variables are ignored.
    /// When `expected_fields` is given too, the validator evaluates only the
    /// fields and this pipeline just provides the reference values for
    /// fields declared without `value`.
    #[serde(default)]
    pub expected: Option<Value>,
    /// Assertions on the actual output (`/path` JXPath, operator, value).
    /// When present they replace the whole-pipeline comparison.
    #[serde(default)]
    pub expected_fields: Vec<ExpectedFieldSpec>,
    /// The service is expected to throw.
    #[serde(default)]
    pub expected_exception: Option<ExceptionSpec>,
    /// Invoke the service now with `input` and snapshot its output (or its
    /// failure) as the expectation -- Designer's "Generate Tests".
    #[serde(default)]
    pub record: bool,
    #[serde(default)]
    pub mocks: Vec<MockSpec>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub mocks_enabled: Option<bool>,
    /// A service implementing `wm.spec:result_comparator`.
    #[serde(default)]
    pub comparator_service: Option<String>,
    /// `invoke` (default, Java client), `post` or `get` (HTTP through httpunit).
    #[serde(default)]
    pub request_method: Option<String>,
    #[serde(default)]
    pub request_content_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedFieldSpec {
    /// JXPath into the output pipeline, e.g. `/greeting` or `/order/lines[1]/qty`.
    pub path: String,
    /// `==` (default), `!=`, `>`, `>=`, `<`, `<=`.
    #[serde(default)]
    pub operator: Option<String>,
    /// Literal to compare with (string, number or boolean). Omit it to
    /// compare against the same path of the `expected` pipeline.
    #[serde(default)]
    pub value: Option<Value>,
    /// `and` (default), `or`, `and not`, `or not` -- joins with the previous field.
    #[serde(default)]
    pub logical: Option<String>,
    #[serde(default)]
    pub start_paren: Option<String>,
    #[serde(default)]
    pub end_paren: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExceptionSpec {
    #[serde(default)]
    pub class: Option<String>,
    /// Substring or regex that must appear in the exception text.
    #[serde(default)]
    pub message: Option<String>,
}

/// One `<mock>`: exactly one of `pipeline`, `alternate_service`, `exception`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MockSpec {
    /// Service to intercept, `folder.sub:name`.
    pub service: String,
    /// Fixed output pipeline returned instead of invoking the service.
    #[serde(default)]
    pub pipeline: Option<Value>,
    /// Service invoked instead (same pipeline), `folder.sub:name`.
    #[serde(default)]
    pub alternate_service: Option<String>,
    /// Extra inputs merged into the pipeline before the alternate service runs.
    #[serde(default)]
    pub parms: Option<Value>,
    /// Throw instead of invoking.
    #[serde(default)]
    pub exception: Option<ExceptionSpec>,
    /// `session` (default), `user`, `server`.
    #[serde(default)]
    pub scope: Option<String>,
    /// `test` (default) or `suite` (stays active for the following tests).
    #[serde(default)]
    pub lifetime: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Debug)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug)]
pub struct GeneratedSuite {
    /// Complete suite document.
    pub xml: String,
    /// Only the `<webMethodsTestCase>` elements (for append mode).
    pub cases_xml: String,
    pub files: Vec<GeneratedFile>,
    pub summary: Vec<Value>,
    /// Non-blocking findings (e.g. JSON numbers in pipelines meant for String fields).
    pub warnings: Vec<String>,
}

/// JSON numbers / booleans inside a pipeline. They are written as
/// `java.lang.Long` / `Double` / `Boolean`, which a String-typed flow field
/// (the common case) silently drops -- the caller almost always wants `"2"`.
fn non_string_scalars(v: &Value, path: &str, out: &mut Vec<String>) {
    match v {
        Value::Number(_) | Value::Bool(_) => out.push(format!("{path} = {v}")),
        Value::Object(o) => {
            for (k, c) in o {
                non_string_scalars(c, &format!("{path}/{k}"), out);
            }
        }
        Value::Array(a) => {
            for (i, c) in a.iter().enumerate() {
                non_string_scalars(c, &format!("{path}[{i}]"), out);
            }
        }
        _ => {}
    }
}

fn typed_value_warning(test: &str, what: &str, v: &Value) -> Option<String> {
    let mut found = Vec::new();
    non_string_scalars(v, "", &mut found);
    if found.is_empty() {
        return None;
    }
    Some(format!(
        "test '{test}': {what} contains non-string values ({}); they are written as java.lang.Long / Double / Boolean, which a flow String field silently ignores -- quote them (\"2\") unless the field really is an Object",
        found.join(", ")
    ))
}

fn json_scalar_to_string(v: &Value, what: &str) -> Result<String, String> {
    match v {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        other => Err(format!(
            "{what} must be a string, number or boolean, got {other}"
        )),
    }
}

/// Build the suite XML and the pipeline files for `tests`.
pub fn build_suite(
    suite_name: &str,
    description: Option<&str>,
    mocks_enabled: bool,
    tests: &[TestCaseSpec],
) -> Result<GeneratedSuite, String> {
    if suite_name.trim().is_empty() {
        return Err("suite_name is required".into());
    }
    if tests.is_empty() {
        return Err("tests must contain at least one test case".into());
    }
    let data_dir = format!("{DATA_DIR}/{}", sanitize(suite_name));
    let mut files = Vec::new();
    let mut cases_xml = String::new();
    let mut summary = Vec::new();
    let mut warnings = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for t in tests {
        if t.name.trim().is_empty() {
            return Err("every test case needs a name".into());
        }
        if !seen.insert(t.name.trim().to_string()) {
            return Err(format!("duplicate test case name '{}'", t.name.trim()));
        }
        push_case(
            &mut cases_xml,
            t,
            &data_dir,
            &mut files,
            &mut summary,
            &mut warnings,
        )?;
    }
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<webMethodsTestSuite");
    attr(&mut xml, "name", suite_name.trim());
    attr(&mut xml, "description", description.unwrap_or(""));
    if !mocks_enabled {
        attr(&mut xml, "mocksEnabled", "false");
    }
    xml.push_str(">\n");
    xml.push_str(&cases_xml);
    xml.push_str("</webMethodsTestSuite>\n");
    Ok(GeneratedSuite {
        xml,
        cases_xml,
        files,
        summary,
        warnings,
    })
}

fn push_case(
    out: &mut String,
    t: &TestCaseSpec,
    data_dir: &str,
    files: &mut Vec<GeneratedFile>,
    summary: &mut Vec<Value>,
    warnings: &mut Vec<String>,
) -> Result<(), String> {
    let name = t.name.trim();
    let stem = sanitize(name);
    let (folder, svc) = split_service(&t.service).map_err(|e| format!("test '{name}': {e}"))?;
    let has_pipeline = t.expected.is_some();
    let has_fields = !t.expected_fields.is_empty();
    let has_exc = t.expected_exception.is_some();
    if has_exc && (has_pipeline || has_fields) {
        return Err(format!(
            "test '{name}': expected_exception cannot be combined with expected / expected_fields"
        ));
    }
    if !(has_pipeline || has_fields || has_exc) {
        return Err(format!(
            "test '{name}': give an expected pipeline, expected_fields, an expected_exception, or set record=true"
        ));
    }
    let request_method = t
        .request_method
        .as_deref()
        .map(str::trim)
        .unwrap_or("invoke");
    if !matches!(request_method, "invoke" | "post" | "get") {
        return Err(format!(
            "test '{name}': request_method must be invoke, post or get (got '{request_method}')"
        ));
    }

    out.push_str("    <webMethodsTestCase");
    attr(out, "description", t.description.as_deref().unwrap_or(""));
    attr(out, "name", name);
    if t.enabled == Some(false) {
        attr(out, "enabled", "false");
    }
    if t.mocks_enabled == Some(false) {
        attr(out, "mocksEnabled", "false");
    }
    out.push_str(">\n");

    for (i, m) in t.mocks.iter().enumerate() {
        push_mock(out, m, i, name, &stem, data_dir, files)?;
    }

    out.push_str("        <service");
    attr(out, "folder", folder);
    attr(out, "name", svc);
    if request_method != "invoke" {
        attr(out, "requestMethod", request_method);
    }
    if let Some(ct) = t
        .request_content_type
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty() && *c != "text/xml")
    {
        attr(out, "requestContentType", ct);
    }
    out.push_str(">\n            <input>\n");
    let mut input_file = None;
    match &t.input {
        Some(input) => {
            let path = format!("{data_dir}/{stem}_input.xml");
            files.push(GeneratedFile {
                path: path.clone(),
                content: idata_xml(input).map_err(|e| format!("test '{name}' input: {e}"))?,
            });
            out.push_str("                <file");
            attr(out, "filename", &path);
            out.push_str("/>\n");
            input_file = Some(path);
        }
        None => out.push_str("                <file/>\n"),
    }
    out.push_str("            </input>\n            <expected>\n");
    let mut expected_file = None;
    let expectation;
    if let Some(exc) = &t.expected_exception {
        let class = exc
            .class
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .unwrap_or(DEFAULT_EXPECTED_EXCEPTION_CLASS);
        out.push_str("                <exception");
        attr(out, "class", class);
        if let Some(m) = exc
            .message
            .as_deref()
            .map(str::trim)
            .filter(|m| !m.is_empty())
        {
            attr(out, "message", m);
        }
        out.push_str("/>\n");
        expectation = "exception";
    } else {
        match &t.expected {
            Some(exp) => {
                let path = format!("{data_dir}/{stem}_expected.xml");
                files.push(GeneratedFile {
                    path: path.clone(),
                    content: idata_xml(exp).map_err(|e| format!("test '{name}' expected: {e}"))?,
                });
                out.push_str("                <file");
                attr(out, "filename", &path);
                out.push_str("/>\n");
                expected_file = Some(path);
            }
            None => out.push_str("                <file/>\n"),
        }
        for f in &t.expected_fields {
            if f.path.trim().is_empty() {
                return Err(format!("test '{name}': every expected field needs a path"));
            }
            if let Some(op) = f
                .operator
                .as_deref()
                .map(str::trim)
                .filter(|op| !matches!(*op, "==" | "!=" | ">" | ">=" | "<" | "<="))
            {
                return Err(format!(
                    "test '{name}': field '{}': operator must be one of == != > >= < <= (got '{op}')",
                    f.path
                ));
            }
            if let Some(l) = f
                .logical
                .as_deref()
                .map(str::trim)
                .filter(|l| !matches!(*l, "and" | "or" | "and not" | "or not"))
            {
                return Err(format!(
                    "test '{name}': field '{}': logical must be and, or, 'and not' or 'or not' (got '{l}')",
                    f.path
                ));
            }
            out.push_str("                <field");
            if let Some(l) = f
                .logical
                .as_deref()
                .map(str::trim)
                .filter(|l| !l.is_empty())
            {
                attr(out, "logical", l);
            }
            if let Some(p) = f.start_paren.as_deref().filter(|p| !p.is_empty()) {
                attr(out, "startParen", p);
            }
            attr(out, "path", f.path.trim());
            if let Some(op) = f
                .operator
                .as_deref()
                .map(str::trim)
                .filter(|o| !o.is_empty())
            {
                attr(out, "operator", op);
            }
            if let Some(v) = f.value.as_ref().filter(|v| !v.is_null()) {
                attr(
                    out,
                    "value",
                    &json_scalar_to_string(v, &format!("test '{name}': field '{}' value", f.path))?,
                );
            }
            if let Some(p) = f.end_paren.as_deref().filter(|p| !p.is_empty()) {
                attr(out, "endParen", p);
            }
            out.push_str("/>\n");
        }
        expectation = match (has_pipeline, has_fields) {
            (true, true) => "pipeline+fields",
            (true, false) => "pipeline",
            _ => "fields",
        };
    }
    out.push_str("            </expected>\n");
    if let Some(cmp) = t
        .comparator_service
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty())
    {
        split_service(cmp).map_err(|e| format!("test '{name}' comparator_service: {e}"))?;
        out.push_str("            <comparator");
        attr(out, "service", cmp);
        out.push_str("/>\n");
    }
    out.push_str("        </service>\n    </webMethodsTestCase>\n");

    let mut case_warnings = Vec::new();
    if let Some(v) = &t.input {
        case_warnings.extend(typed_value_warning(name, "input", v));
    }
    if let Some(v) = &t.expected {
        case_warnings.extend(typed_value_warning(name, "expected", v));
    }
    for (i, m) in t.mocks.iter().enumerate() {
        if let Some(v) = &m.pipeline {
            case_warnings.extend(typed_value_warning(
                name,
                &format!("mock #{} pipeline", i + 1),
                v,
            ));
        }
        if let Some(v) = &m.parms {
            case_warnings.extend(typed_value_warning(
                name,
                &format!("mock #{} parms", i + 1),
                v,
            ));
        }
    }
    warnings.extend(case_warnings.iter().cloned());
    summary.push(json!({
        "name": name,
        "service": t.service.trim(),
        "expectation": expectation,
        "warnings": case_warnings,
        "inputFile": input_file,
        "expectedFile": expected_file,
        "mocks": t.mocks.iter().map(|m| m.service.trim()).collect::<Vec<_>>(),
        "enabled": t.enabled.unwrap_or(true),
    }));
    Ok(())
}

fn push_mock(
    out: &mut String,
    m: &MockSpec,
    index: usize,
    test_name: &str,
    stem: &str,
    data_dir: &str,
    files: &mut Vec<GeneratedFile>,
) -> Result<(), String> {
    let label = format!("test '{test_name}', mock #{}", index + 1);
    let (folder, svc) = split_service(&m.service).map_err(|e| format!("{label}: {e}"))?;
    let kinds = [
        m.pipeline.is_some(),
        m.alternate_service.is_some(),
        m.exception.is_some(),
    ]
    .iter()
    .filter(|b| **b)
    .count();
    if kinds != 1 {
        return Err(format!(
            "{label}: give exactly one of pipeline, alternate_service or exception"
        ));
    }
    if m.parms.is_some() && m.alternate_service.is_none() {
        return Err(format!(
            "{label}: parms only applies together with alternate_service"
        ));
    }
    let scope = m.scope.as_deref().map(str::trim).unwrap_or("session");
    if !matches!(scope, "session" | "user" | "server") {
        return Err(format!(
            "{label}: scope must be session, user or server (got '{scope}')"
        ));
    }
    let lifetime = m.lifetime.as_deref().map(str::trim).unwrap_or("test");
    if !matches!(lifetime, "test" | "suite") {
        return Err(format!(
            "{label}: lifetime must be test or suite (got '{lifetime}')"
        ));
    }
    out.push_str("        <mock");
    attr(out, "folder", folder);
    attr(out, "name", svc);
    if scope != "session" {
        attr(out, "scope", scope);
    }
    if lifetime != "test" {
        attr(out, "lifetime", lifetime);
    }
    if m.enabled == Some(false) {
        attr(out, "enabled", "false");
    }
    out.push_str(">\n");
    let mock_stem = format!("{stem}_mock{}_{}", index + 1, sanitize(&m.service));
    if let Some(p) = &m.pipeline {
        let path = format!("{data_dir}/{mock_stem}.xml");
        files.push(GeneratedFile {
            path: path.clone(),
            content: idata_xml(p).map_err(|e| format!("{label} pipeline: {e}"))?,
        });
        out.push_str("            <pipeline");
        attr(out, "filename", &path);
        out.push_str("/>\n");
    } else if let Some(alt) = &m.alternate_service {
        let (af, an) = split_service(alt).map_err(|e| format!("{label} alternate_service: {e}"))?;
        out.push_str("            <service");
        attr(out, "folder", af);
        attr(out, "name", an);
        if let Some(parms) = &m.parms {
            let path = format!("{data_dir}/{mock_stem}_parms.xml");
            files.push(GeneratedFile {
                path: path.clone(),
                content: idata_xml(parms).map_err(|e| format!("{label} parms: {e}"))?,
            });
            out.push_str(">\n                <pipeline");
            attr(out, "filename", &path);
            out.push_str("/>\n            </service>\n");
        } else {
            out.push_str("/>\n");
        }
    } else if let Some(exc) = &m.exception {
        let class = exc
            .class
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty());
        let message = exc
            .message
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty());
        if class.is_none() && message.is_none() {
            return Err(format!("{label}: exception needs a class and/or a message"));
        }
        out.push_str("            <exception");
        if let Some(c) = class {
            attr(out, "class", c);
        }
        if let Some(msg) = message {
            attr(out, "message", msg);
        }
        out.push_str("/>\n");
    }
    out.push_str("        </mock>\n");
    Ok(())
}

/// Insert `cases_xml` before the closing tag of an existing suite document
/// (also handles the empty `<webMethodsTestSuite .../>` Designer creates).
/// Fails when one of `names` is already a test case of the suite.
pub fn append_cases(existing: &str, cases_xml: &str, names: &[&str]) -> Result<String, String> {
    if !existing.contains("<webMethodsTestSuite") {
        return Err("the existing file is not a webMethodsTestSuite document".into());
    }
    for n in names {
        let needle = format!(" name=\"{}\"", esc_attr(n.trim()));
        let mut rest = existing;
        while let Some(start) = rest.find("<webMethodsTestCase") {
            let tag_end = rest[start..]
                .find('>')
                .map(|e| start + e)
                .unwrap_or(rest.len());
            if rest[start..tag_end].contains(&needle) {
                return Err(format!(
                    "test case '{}' already exists in the suite; pick another name or use mode=overwrite",
                    n.trim()
                ));
            }
            rest = &rest[tag_end..];
        }
    }
    if let Some(pos) = existing.rfind("</webMethodsTestSuite>") {
        let mut out = String::with_capacity(existing.len() + cases_xml.len());
        out.push_str(&existing[..pos]);
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(cases_xml);
        out.push_str(&existing[pos..]);
        return Ok(out);
    }
    // self-closing root element
    let start = existing
        .find("<webMethodsTestSuite")
        .expect("checked above");
    let close = existing[start..]
        .find("/>")
        .map(|c| start + c)
        .ok_or("malformed webMethodsTestSuite element")?;
    let mut out = String::new();
    out.push_str(&existing[..close]);
    out.push_str(">\n");
    out.push_str(cases_xml);
    out.push_str("</webMethodsTestSuite>");
    out.push_str(&existing[close + 2..]);
    Ok(out)
}

// ───────────────────────── package file store ──────────────────────

/// Where suite files live and how they are written: directly on the
/// filesystem when the IS packages directory is reachable from this process,
/// otherwise through pub.file:stringToFile on the IS (which requires the
/// packages directory in watt.server.file.canWritePaths).
pub struct PackageStore {
    pub packages_dir: String,
    pub package: String,
    pub local: bool,
}

impl PackageStore {
    fn abs(&self, rel: &str) -> String {
        let root = self.packages_dir.trim_end_matches(['/', '\\']);
        format!("{root}/{}/{rel}", self.package)
    }

    async fn read(&self, c: &ISClient, rel: &str) -> Result<Option<String>, String> {
        if self.local {
            return match std::fs::read_to_string(self.abs(rel)) {
                Ok(s) => Ok(Some(s)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(format!("cannot read {}: {e}", self.abs(rel))),
            };
        }
        match c
            .invoke_post_text(
                "wm.task.asset:suiteIO",
                &json!({"packageName": self.package, "resourcePath": rel, "resourceType": "text/plain"}),
            )
            .await
        {
            Ok(s) => Ok(Some(s)),
            Err(e) if e.contains("is not available") => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn write(&self, c: &ISClient, rel: &str, content: &str) -> Result<(), String> {
        let abs = self.abs(rel);
        if self.local {
            if let Some(parent) = std::path::Path::new(&abs).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
            }
            return std::fs::write(&abs, content).map_err(|e| format!("cannot write {abs}: {e}"));
        }
        c.invoke_post("pub.file:stringToFile", &json!({"fileName": abs, "data": content}))
            .await
            .map(|_| ())
            .map_err(|e| {
                if e.contains("ISS.0086.9263") {
                    format!(
                        "{e}\n\nThis MCP server has no filesystem access to the IS packages directory ({}) and the IS refuses to write there through pub.file:stringToFile. Either run the MCP server on the IS host, or add that directory to the IS extended setting watt.server.file.canWritePaths (Settings > Extended in the Admin UI, or wm.server.admin:updateExtendedSettings) and retry.",
                        self.packages_dir
                    )
                } else {
                    e
                }
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuiteMode {
    Create,
    Overwrite,
    Append,
}

impl SuiteMode {
    pub fn parse(s: Option<&str>) -> Result<Self, String> {
        match s.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
            None | Some("") | Some("create") => Ok(Self::Create),
            Some("overwrite") => Ok(Self::Overwrite),
            Some("append") => Ok(Self::Append),
            Some(other) => Err(format!(
                "mode must be create, overwrite or append (got '{other}')"
            )),
        }
    }
}

pub struct SuiteCreateOptions {
    pub package: String,
    pub suite_name: String,
    pub description: Option<String>,
    pub mocks_enabled: bool,
    pub mode: SuiteMode,
}

/// Pull the `$error` text out of an IS error response wrapped by `read_checked`
/// (`HTTP 500 ...: {"$error":"...","$errorType":...,"$errorDump":"..."}`).
/// The body snippet is truncated to 2000 chars, so the JSON is often cut in
/// the middle of `$errorDump`: fall back to scanning the `$error` string.
fn is_error_text(e: &str) -> String {
    if let Some(msg) = e
        .find('{')
        .and_then(|start| serde_json::from_str::<Value>(&e[start..]).ok())
        .and_then(|v| v.get("$error").and_then(Value::as_str).map(str::to_string))
    {
        return msg;
    }
    const KEY: &str = "\"$error\":\"";
    let Some(start) = e.find(KEY).map(|i| i + KEY.len()) else {
        return e.to_string();
    };
    let mut out = String::new();
    let mut chars = e[start..].chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return out,
            '\\' => match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('u') => {
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Some(ch) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                        out.push(ch);
                    }
                }
                Some(other) => out.push(other),
                None => break,
            },
            _ => out.push(c),
        }
    }
    if out.is_empty() { e.to_string() } else { out }
}

/// Names of the declared output fields of a service (empty when unknown).
fn output_field_names(node: &Value) -> Vec<String> {
    let mut names = Vec::new();
    let Some(fields) = node
        .pointer("/node/svc_sig/sig_out/rec_fields")
        .or_else(|| node.pointer("/svc_sig/sig_out/rec_fields"))
        .and_then(Value::as_array)
    else {
        return names;
    };
    let mut stack: Vec<&Value> = fields.iter().collect();
    while let Some(f) = stack.pop() {
        match f {
            Value::Array(inner) => stack.extend(inner.iter()),
            Value::Object(o) => {
                if let Some(n) = o.get("field_name").and_then(Value::as_str) {
                    names.push(n.to_string());
                }
            }
            _ => {}
        }
    }
    names
}

impl ISClient {
    pub async fn packages_dir(&self) -> Result<String, String> {
        let paths: Value = self
            .invoke_get("wm.server.query:getServerPaths")
            .await
            .map_err(|e| format!("Failed to get server paths: {e}"))?;
        paths
            .get("packagesDir")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| "Cannot determine the IS packages directory".to_string())
    }

    async fn package_store(&self, package: &str) -> Result<PackageStore, String> {
        let info = self
            .package_info(package)
            .await
            .map_err(|e| format!("package '{package}': {e}"))?;
        if info.get("enabled").is_none() {
            return Err(format!(
                "package '{package}' does not exist on the IS (create it with package_create first)"
            ));
        }
        let packages_dir = self.packages_dir().await?;
        let local = std::path::Path::new(&packages_dir).join(package).is_dir();
        Ok(PackageStore {
            packages_dir,
            package: package.to_string(),
            local,
        })
    }

    /// Designer's "Generate Tests": run the service, keep its declared outputs
    /// (or everything when the signature declares none) as the expectation,
    /// or turn a failure into an expected exception.
    async fn record_case(&self, t: &mut TestCaseSpec) -> Result<Value, String> {
        if t.expected.is_some() || !t.expected_fields.is_empty() || t.expected_exception.is_some() {
            return Err(format!(
                "test '{}': record=true replaces the expectation, do not combine it with expected / expected_fields / expected_exception",
                t.name
            ));
        }
        if !t.mocks.is_empty() {
            return Err(format!(
                "test '{}': record=true invokes the service without the test's mocks (they only exist inside a suite run), so the snapshot would not match the mocked run -- give an explicit expected / expected_fields for mocked tests",
                t.name
            ));
        }
        match self
            .service_invoke(t.service.trim(), t.input.as_ref(), None)
            .await
        {
            Ok(output) => {
                let declared = self
                    .node_get(t.service.trim())
                    .await
                    .map(|n| output_field_names(&n))
                    .unwrap_or_default();
                let mut snapshot = Map::new();
                let mut filtered = !declared.is_empty();
                let mut note = None;
                if let Some(obj) = output.as_object() {
                    for (k, v) in obj {
                        if declared.is_empty() || declared.iter().any(|d| d == k) {
                            snapshot.insert(k.clone(), v.clone());
                        }
                    }
                    if snapshot.is_empty() && !obj.is_empty() {
                        // No declared output came back (error branch, undeclared
                        // outputs): keep everything the service left in the
                        // pipeline except what we sent in.
                        let inputs: Vec<&String> = t
                            .input
                            .as_ref()
                            .and_then(Value::as_object)
                            .map(|m| m.keys().collect())
                            .unwrap_or_default();
                        for (k, v) in obj {
                            if !inputs.contains(&k) {
                                snapshot.insert(k.clone(), v.clone());
                            }
                        }
                        filtered = false;
                        note = Some(
                            "none of the declared outputs came back; recorded every non-input variable instead",
                        );
                    }
                }
                let kept: Vec<&String> = snapshot.keys().collect();
                let info = json!({
                    "test": t.name,
                    "recorded": "output",
                    "variables": kept,
                    "filteredBySignature": filtered,
                    "note": note,
                });
                t.expected = Some(Value::Object(snapshot));
                Ok(info)
            }
            Err(e) => {
                let msg = is_error_text(&e);
                t.expected_exception = Some(ExceptionSpec {
                    class: Some(DEFAULT_EXPECTED_EXCEPTION_CLASS.to_string()),
                    message: Some(msg.clone()),
                });
                Ok(json!({"test": t.name, "recorded": "exception", "message": msg}))
            }
        }
    }

    pub async fn test_suite_create(
        &self,
        opts: SuiteCreateOptions,
        mut tests: Vec<TestCaseSpec>,
    ) -> Result<Value, String> {
        if tests.is_empty() {
            return Err("tests must contain at least one test case".into());
        }
        let store = self.package_store(&opts.package).await?;
        let mut recordings = Vec::new();
        for t in tests.iter_mut() {
            if t.record {
                recordings.push(self.record_case(t).await?);
            }
        }
        let generated = build_suite(
            &opts.suite_name,
            opts.description.as_deref(),
            opts.mocks_enabled,
            &tests,
        )?;
        let suite_path = format!("{SUITE_DIR}/{}.xml", sanitize(&opts.suite_name));
        let existing = store.read(self, &suite_path).await?;
        let xml = match (opts.mode, existing) {
            (SuiteMode::Create, Some(_)) => {
                return Err(format!(
                    "{suite_path} already exists in package {}; use mode=append to add test cases or mode=overwrite to replace it",
                    opts.package
                ));
            }
            (SuiteMode::Append, Some(old)) => {
                let names: Vec<&str> = tests.iter().map(|t| t.name.as_str()).collect();
                append_cases(&old, &generated.cases_xml, &names)?
            }
            _ => generated.xml.clone(),
        };
        let mut written = Vec::new();
        for f in &generated.files {
            store.write(self, &f.path, &f.content).await?;
            written.push(f.path.clone());
        }
        store.write(self, &suite_path, &xml).await?;
        written.push(suite_path.clone());
        Ok(json!({
            "package": opts.package,
            "suite": opts.suite_name.trim(),
            "suiteFile": suite_path,
            "mode": match opts.mode { SuiteMode::Create => "create", SuiteMode::Overwrite => "overwrite", SuiteMode::Append => "append" },
            "writtenVia": if store.local { "filesystem" } else { "pub.file:stringToFile" },
            "packageDir": store.abs(""),
            "files": written,
            "tests": generated.summary,
            "warnings": generated.warnings,
            "recordings": recordings,
            "xml": xml,
            "next": format!("test_run with test_suite_packages [\"{}\"], then test_text_report / test_junit_report with the returned executionID", opts.package),
        }))
    }

    /// `wm.task.asset:suites`: the suites and test cases found in packages.
    pub async fn test_suite_list(&self, packages: Option<&Value>) -> Result<Value, String> {
        let mut payload = json!({});
        if let Some(p) = packages {
            payload["testSuitePackages"] = p.clone();
        }
        let v = self.invoke_post("wm.task.asset:suites", &payload).await?;
        Ok(v.get("suites").cloned().unwrap_or(v))
    }

    /// `wm.task.asset:suiteIO`: a suite or pipeline file of a package, raw or
    /// decoded from IDataXMLCoder to JSON.
    pub async fn test_suite_get(
        &self,
        package: &str,
        path: &str,
        as_pipeline: bool,
    ) -> Result<Value, String> {
        if as_pipeline {
            let v = self
                .invoke_post(
                    "wm.task.asset:suiteIO",
                    &json!({"packageName": package, "resourcePath": path, "resourceType": ""}),
                )
                .await?;
            return Ok(v.get("resourceData").cloned().unwrap_or(v));
        }
        let text = self
            .invoke_post_text(
                "wm.task.asset:suiteIO",
                &json!({"packageName": package, "resourcePath": path, "resourceType": "text/plain"}),
            )
            .await?;
        Ok(Value::String(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idata_xml_matches_the_is_encoding() {
        let v = json!({
            "s": "str", "empty": "", "n": 5, "f": 1.5, "b": true, "nul": null,
            "doc": {"a": "1", "inner": {"x": "y"}},
            "list": ["a", "b"], "docs": [{"k": "v"}], "nested": [["1", "2"], ["3"]],
            "nums": [1, 2], "mixed": ["a", {"z": "1"}], "emptyArr": [],
            "xml": "<b>&amp;</b>"
        });
        let x = idata_xml(&v).unwrap();
        assert!(x.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\n<IDataXMLCoder version=\"1.0\">\n  <record javaclass=\"com.wm.data.ISMemDataImpl\">\n"));
        assert!(x.contains("<value name=\"s\">str</value>"));
        assert!(x.contains("<value name=\"empty\"></value>"));
        assert!(x.contains("<number name=\"n\" type=\"java.lang.Long\">5</number>"));
        assert!(x.contains("<double name=\"f\" type=\"java.lang.Double\">1.5</double>"));
        assert!(x.contains("<jboolean name=\"b\">true</jboolean>"));
        assert!(x.contains("<null name=\"nul\"/>"));
        assert!(x.contains("<record name=\"doc\" javaclass=\"com.wm.data.ISMemDataImpl\">\n      <value name=\"a\">1</value>\n      <record name=\"inner\""));
        assert!(
            x.contains("<array name=\"list\" type=\"value\" depth=\"1\">\n      <value>a</value>")
        );
        assert!(x.contains("<array name=\"docs\" type=\"record\" depth=\"1\">\n      <record javaclass=\"com.wm.data.ISMemDataImpl\">"));
        assert!(x.contains("<array name=\"nested\" type=\"value\" depth=\"2\">\n      <array type=\"value\" depth=\"1\">"));
        assert!(x.contains("<array name=\"nums\" type=\"object\" depth=\"1\" javaclass=\"java.lang.Long\">\n      <number type=\"java.lang.Long\">1</number>"));
        assert!(x.contains("<array name=\"mixed\" type=\"object\" depth=\"1\">"));
        assert!(x.contains("<array name=\"emptyArr\" type=\"value\" depth=\"1\">\n    </array>"));
        assert!(x.contains("<value name=\"xml\">&lt;b&gt;&amp;amp;&lt;/b&gt;</value>"));
        assert!(x.ends_with("  </record>\n</IDataXMLCoder>\n"));
        assert!(idata_xml(&json!(["not", "an", "object"])).is_err());
    }

    fn spec(s: &str) -> TestCaseSpec {
        serde_json::from_str(s).unwrap()
    }

    #[test]
    fn suite_xml_follows_designer_rules() {
        let tests = vec![
            spec(
                r#"{"name":"greetAlice","description":"full compare","service":"utfdemo.services:greet","input":{"name":"Alice"},"expected":{"greeting":"Hello, Alice"}}"#,
            ),
            spec(
                r#"{"name":"fields","service":"utfdemo.services:greet","expected_fields":[{"path":"/greeting","operator":"==","value":"Hello, World"},{"path":"/n","value":5,"logical":"or"}],"mocks":[{"service":"pub.string:concat","pipeline":{"value":"MOCKED"},"scope":"server","lifetime":"suite"},{"service":"pub.client:http","alternate_service":"utfdemo.mocks:http","parms":{"x":"1"}},{"service":"pub.file:getFile","exception":{"class":"java.io.IOException","message":"boom"},"enabled":false}]}"#,
            ),
            spec(
                r#"{"name":"fails","service":"utfdemo.services:greet","expected_exception":{"message":"Amount must be"},"enabled":false,"mocks_enabled":false,"comparator_service":"utfdemo.test:compare","request_method":"invoke"}"#,
            ),
        ];
        let g = build_suite("GreetSuite", Some("demo & \"quotes\""), true, &tests).unwrap();
        let x = &g.xml;
        assert!(x.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<webMethodsTestSuite name=\"GreetSuite\" description=\"demo &amp; &quot;quotes&quot;\">\n"));
        assert!(x.ends_with("</webMethodsTestSuite>\n"));
        assert!(
            !x.contains("mocksEnabled=\"true\""),
            "defaults are omitted like Utils.setAttribute does"
        );
        assert!(
            x.contains("<webMethodsTestCase description=\"full compare\" name=\"greetAlice\">")
        );
        assert!(x.contains("<service folder=\"utfdemo.services\" name=\"greet\">"));
        assert!(x.contains("<input>\n                <file filename=\"resources/test/data/GreetSuite/greetAlice_input.xml\"/>"));
        assert!(x.contains("<expected>\n                <file filename=\"resources/test/data/GreetSuite/greetAlice_expected.xml\"/>\n            </expected>"));
        assert!(x.contains("<webMethodsTestCase description=\"\" name=\"fields\">"));
        assert!(x.contains("<mock folder=\"pub.string\" name=\"concat\" scope=\"server\" lifetime=\"suite\">\n            <pipeline filename=\"resources/test/data/GreetSuite/fields_mock1_pub.string_concat.xml\"/>"));
        assert!(x.contains("<mock folder=\"pub.client\" name=\"http\">\n            <service folder=\"utfdemo.mocks\" name=\"http\">\n                <pipeline filename=\"resources/test/data/GreetSuite/fields_mock2_pub.client_http_parms.xml\"/>\n            </service>"));
        assert!(x.contains("<mock folder=\"pub.file\" name=\"getFile\" enabled=\"false\">\n            <exception class=\"java.io.IOException\" message=\"boom\"/>"));
        assert!(x.contains("<expected>\n                <file/>\n                <field path=\"/greeting\" operator=\"==\" value=\"Hello, World\"/>\n                <field logical=\"or\" path=\"/n\" value=\"5\"/>"));
        assert!(x.contains("<webMethodsTestCase description=\"\" name=\"fails\" enabled=\"false\" mocksEnabled=\"false\">"));
        assert!(x.contains("<exception class=\"com.wm.app.b2b.client.ServiceException\" message=\"Amount must be\"/>"));
        assert!(x.contains("<comparator service=\"utfdemo.test:compare\"/>"));
        assert!(
            !x.contains("requestMethod"),
            "invoke is the default and must be omitted"
        );
        let paths: Vec<&str> = g.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(
            paths,
            vec![
                "resources/test/data/GreetSuite/greetAlice_input.xml",
                "resources/test/data/GreetSuite/greetAlice_expected.xml",
                "resources/test/data/GreetSuite/fields_mock1_pub.string_concat.xml",
                "resources/test/data/GreetSuite/fields_mock2_pub.client_http_parms.xml",
            ]
        );
        assert_eq!(g.summary.len(), 3);
        assert_eq!(g.summary[1]["expectation"], "fields");
        assert_eq!(g.summary[2]["expectation"], "exception");
    }

    #[test]
    fn non_string_scalars_are_reported() {
        let g = build_suite("S", None, true, &[spec(r#"{"name":"t","service":"a:b","input":{"n":5,"doc":{"ok":true},"list":["x"]},"expected":{"greeting":"hi"},"mocks":[{"service":"x:y","pipeline":{"total":1.5}}]}"#)]).unwrap();
        assert_eq!(g.warnings.len(), 2, "{:?}", g.warnings);
        assert!(
            g.warnings[0].contains("input")
                && g.warnings[0].contains("/n = 5")
                && g.warnings[0].contains("/doc/ok = true"),
            "{}",
            g.warnings[0]
        );
        assert!(
            g.warnings[1].contains("mock #1 pipeline") && g.warnings[1].contains("/total = 1.5"),
            "{}",
            g.warnings[1]
        );
        assert_eq!(g.summary[0]["warnings"].as_array().unwrap().len(), 2);
        let clean = build_suite(
            "S",
            None,
            true,
            &[spec(r#"{"name":"t","service":"a:b","expected":{"n":"5"}}"#)],
        )
        .unwrap();
        assert!(clean.warnings.is_empty());
    }

    #[test]
    fn suite_validation_errors_are_specific() {
        let err = build_suite(
            "S",
            None,
            true,
            &[spec(r#"{"name":"t","service":"a.b:c"}"#)],
        )
        .unwrap_err();
        assert!(
            err.contains(
                "expected pipeline, expected_fields, an expected_exception, or set record=true"
            ),
            "{err}"
        );
        let err = build_suite(
            "S",
            None,
            true,
            &[spec(r#"{"name":"t","service":"nocolon","expected":{}}"#)],
        )
        .unwrap_err();
        assert!(err.contains("fully qualified service name"), "{err}");
        let err = build_suite(
            "S",
            None,
            true,
            &[spec(
                r#"{"name":"t","service":"a:b","expected":{},"mocks":[{"service":"x:y"}]}"#,
            )],
        )
        .unwrap_err();
        assert!(
            err.contains("exactly one of pipeline, alternate_service or exception"),
            "{err}"
        );
        let err = build_suite(
            "S",
            None,
            true,
            &[spec(
                r#"{"name":"t","service":"a:b","expected_fields":[{"path":"/x","operator":"~"}]}"#,
            )],
        )
        .unwrap_err();
        assert!(err.contains("operator must be one of"), "{err}");
        let err = build_suite(
            "S",
            None,
            true,
            &[
                spec(r#"{"name":"t","service":"a:b","expected":{}}"#),
                spec(r#"{"name":"t","service":"a:b","expected":{}}"#),
            ],
        )
        .unwrap_err();
        assert!(err.contains("duplicate test case name"), "{err}");
        assert!(
            serde_json::from_str::<TestCaseSpec>(
                r#"{"name":"t","service":"a:b","expectedFields":[]}"#
            )
            .unwrap_err()
            .to_string()
            .contains("unknown field")
        );
    }

    #[test]
    fn append_inserts_before_closing_tag_and_rejects_duplicates() {
        let existing = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<webMethodsTestSuite name=\"S\" description=\"\">\n    <webMethodsTestCase description=\"\" name=\"old\">\n    </webMethodsTestCase>\n</webMethodsTestSuite>\n";
        let out = append_cases(
            existing,
            "    <webMethodsTestCase name=\"new\"/>\n",
            &["new"],
        )
        .unwrap();
        assert!(out.ends_with("name=\"old\">\n    </webMethodsTestCase>\n    <webMethodsTestCase name=\"new\"/>\n</webMethodsTestSuite>\n"), "{out}");
        assert!(
            append_cases(existing, "", &["old"])
                .unwrap_err()
                .contains("already exists")
        );
        let empty = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<webMethodsTestSuite/>";
        let out =
            append_cases(empty, "    <webMethodsTestCase name=\"new\"/>\n", &["new"]).unwrap();
        assert_eq!(
            out,
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<webMethodsTestSuite>\n    <webMethodsTestCase name=\"new\"/>\n</webMethodsTestSuite>"
        );
        assert!(append_cases("<foo/>", "", &[]).is_err());
    }

    #[test]
    fn helpers() {
        assert_eq!(sanitize(" my suite:1/2 "), "my_suite_1_2");
        assert_eq!(sanitize("..."), "unnamed");
        assert_eq!(split_service("a.b:c").unwrap(), ("a.b", "c"));
        assert!(split_service("a.b:c:d").is_err());
        assert_eq!(SuiteMode::parse(None).unwrap(), SuiteMode::Create);
        assert_eq!(SuiteMode::parse(Some("Append")).unwrap(), SuiteMode::Append);
        assert!(SuiteMode::parse(Some("replace")).is_err());
        assert_eq!(
            is_error_text("HTTP 500: {\"$error\":\"boom\",\"$errorType\":\"x\"}"),
            "boom"
        );
        assert_eq!(is_error_text("plain"), "plain");
        // truncated body (read_checked keeps 2000 chars): the JSON no longer parses
        let truncated = "HTTP 500 Internal Server Error: {\"$error\":\"java.lang.NumberFormatException: \\\"x\\\" \\u00e9\",\"$errorType\":\"com.wm.app.b2b.server.ServiceException\",\"$errorDump\":\"com.wm.app.b2b.server.ServiceException: java.lang.NumberFormatEx";
        assert_eq!(
            is_error_text(truncated),
            "java.lang.NumberFormatException: \"x\" \u{e9}"
        );
        let node = json!({"node": {"svc_sig": {"sig_out": {"rec_fields": [{"_type_info": "IData[]"}, [{"field_name": "greeting"}, {"field_name": "code"}]]}}}});
        let mut names = output_field_names(&node);
        names.sort();
        assert_eq!(names, ["code", "greeting"]);
    }
}
