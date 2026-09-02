use serde_json::{Value, json};

/// Mock scopes understood by `wm.ps.serviceMock:loadMock` / `clearMock`.
/// Any other value is silently downgraded to `session` by the IS.
pub const MOCK_SCOPES: [&str; 3] = ["server", "user", "session"];

/// Normalise the `scope` given to mock_load / mock_clear.
///
/// `None` (or empty) defaults to `server`, `global` is accepted as an alias of
/// `server` (the name this tool documented before 2.10.1), and anything else
/// is rejected here instead of being downgraded to `session` by the IS. A
/// `session` mock is pointless through this server anyway: every MCP call is a
/// separate HTTP request with Basic auth, i.e. a new IS session that ends when
/// the call returns.
pub fn normalize_mock_scope(scope: Option<&str>) -> Result<String, String> {
    let s = scope
        .map(|s| s.trim().to_ascii_lowercase())
        .unwrap_or_default();
    match s.as_str() {
        "" | "global" => Ok("server".to_string()),
        s if MOCK_SCOPES.contains(&s) => Ok(s.to_string()),
        other => Err(format!(
            "Invalid mock scope '{other}': use 'server' (all users and sessions -- the only \
             scope that survives across MCP calls), 'user' (all sessions of the IS user the \
             MCP server authenticates as) or 'session' (current IS session only)"
        )),
    }
}

/// Warning attached to a mock loaded with scope `session`.
pub const SESSION_SCOPE_WARNING: &str = "scope 'session' only covers the IS session of this \
     single HTTP call, which is already closed: the mock will not apply to later calls. Use \
     scope 'server' or 'user'.";

/// Remove every `<properties>...</properties>` block from a JUnit XML report.
///
/// The Ant JUnit formatter dumps all JVM system properties of the runner
/// there (about 100 KB per `<testsuite>`), which is pure noise for a caller
/// that wants test results. `<properties/>` (self-closing) is handled too.
/// The rest of the document is returned verbatim.
pub fn strip_junit_properties(xml: &str) -> String {
    const OPEN: &str = "<properties";
    const CLOSE: &str = "</properties>";
    let mut out = String::with_capacity(xml.len());
    let mut rest = xml;
    while let Some(start) = rest.find(OPEN) {
        let head = &rest[start..];
        let Some(gt) = head.find('>') else { break };
        let end = if head[..gt].ends_with('/') {
            start + gt + 1
        } else {
            match head.find(CLOSE) {
                Some(e) => start + e + CLOSE.len(),
                None => break,
            }
        };
        out.push_str(&rest[..start]);
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

struct Tag<'a> {
    name: &'a str,
    attrs: &'a str,
    closing: bool,
    self_closing: bool,
    end: usize,
}

/// Next element tag at or after `pos`, skipping CDATA, comments, PIs and
/// DOCTYPE. Good enough for the regular XML the Ant JUnit formatter writes.
fn next_tag(s: &str, mut pos: usize) -> Option<(usize, Tag<'_>)> {
    loop {
        let lt = s[pos..].find('<')? + pos;
        let rest = &s[lt..];
        for (open, close) in [("<![CDATA[", "]]>"), ("<!--", "-->"), ("<?", "?>")] {
            if rest.starts_with(open) {
                pos = lt + rest.find(close)? + close.len();
                break;
            }
        }
        if pos > lt {
            continue;
        }
        if rest.starts_with("<!") {
            pos = lt + rest.find('>')? + 1;
            continue;
        }
        let gt = rest.find('>')?;
        let inner = &rest[1..gt];
        let closing = inner.starts_with('/');
        let inner = inner.trim_start_matches('/');
        let self_closing = inner.ends_with('/');
        let inner = inner.trim_end_matches('/').trim();
        let (name, attrs) = match inner.find(char::is_whitespace) {
            Some(i) => (&inner[..i], inner[i..].trim()),
            None => (inner, ""),
        };
        return Some((
            lt,
            Tag {
                name,
                attrs,
                closing,
                self_closing,
                end: lt + gt + 1,
            },
        ));
    }
}

fn xml_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let tail = &rest[amp..];
        let Some(semi) = tail.find(';') else {
            out.push_str(tail);
            return out;
        };
        let entity = &tail[1..semi];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            e if e.starts_with("#x") => u32::from_str_radix(&e[2..], 16)
                .ok()
                .and_then(char::from_u32),
            e if e.starts_with('#') => e[1..].parse::<u32>().ok().and_then(char::from_u32),
            _ => None,
        };
        match decoded {
            Some(c) => out.push(c),
            None => out.push_str(&tail[..=semi]),
        }
        rest = &tail[semi + 1..];
    }
    out.push_str(rest);
    out
}

fn parse_attrs(s: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        let ns = i;
        while i < b.len() && b[i] != b'=' && !b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= b.len() {
            break;
        }
        let name = &s[ns..i];
        while i < b.len() && b[i] != b'=' {
            i += 1;
        }
        i += 1;
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= b.len() || (b[i] != b'"' && b[i] != b'\'') {
            break;
        }
        let q = b[i];
        i += 1;
        let vs = i;
        while i < b.len() && b[i] != q {
            i += 1;
        }
        out.push((name.to_string(), xml_unescape(&s[vs..i.min(b.len())])));
        i += 1;
    }
    out
}

fn attr<'a>(attrs: &'a [(String, String)], name: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v.as_str())
}

fn num(attrs: &[(String, String)], name: &str) -> f64 {
    attr(attrs, name)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0)
}

type Attrs = Vec<(String, String)>;

/// A `<testsuite>` being read: its attributes and the cases seen so far.
struct SuiteAcc {
    attrs: Attrs,
    cases: Vec<Value>,
}

/// A `<testcase>` being read.
struct CaseAcc {
    attrs: Attrs,
    status: &'static str,
    message: Option<String>,
    kind: Option<String>,
    detail: Option<String>,
}

fn finish_case(case: &mut Option<CaseAcc>, suite: &mut Option<SuiteAcc>) {
    let Some(CaseAcc {
        attrs,
        status,
        message,
        kind,
        detail,
    }) = case.take()
    else {
        return;
    };
    let raw = attr(&attrs, "name").unwrap_or("").to_string();
    // the Ant formatter names cases "<suite index>.<case index> <name>"
    let (id, name) = match raw.split_once(' ') {
        Some((prefix, rest))
            if prefix.chars().all(|c| c.is_ascii_digit() || c == '.') && !rest.is_empty() =>
        {
            (Some(prefix.to_string()), rest.to_string())
        }
        _ => (None, raw.clone()),
    };
    let entry = json!({
        "id": id,
        "name": name,
        "status": status,
        "time": num(&attrs, "time"),
        "message": message,
        "type": kind,
        "detail": detail,
    });
    suite
        .get_or_insert_with(|| SuiteAcc {
            attrs: Vec::new(),
            cases: Vec::new(),
        })
        .cases
        .push(entry);
}

/// Structured view of a JUnit XML report (`<testsuites>` / `<testsuite>` /
/// `<testcase>` with `<failure>` / `<error>` / `<skipped>` children):
/// `{"totals": {...}, "suites": [{"name", "package", "tests", "failures",
/// "errors", "skipped", "time", "cases": [{"id", "name", "status", "time",
/// "message", "type", "detail"}]}]}`.
pub fn junit_summary(xml: &str) -> Value {
    let xml = strip_junit_properties(xml);
    let mut suites: Vec<Value> = Vec::new();
    let mut suite: Option<SuiteAcc> = None;
    let mut case: Option<CaseAcc> = None;
    let mut pos = 0;
    while let Some((_, tag)) = next_tag(&xml, pos) {
        pos = tag.end;
        match (tag.name, tag.closing) {
            ("testsuite", false) => {
                suite = Some(SuiteAcc {
                    attrs: parse_attrs(tag.attrs),
                    cases: Vec::new(),
                });
                if tag.self_closing {
                    let s = suite.take().expect("just set");
                    suites.push(suite_json(&s.attrs, s.cases));
                }
            }
            ("testsuite", true) => {
                finish_case(&mut case, &mut suite);
                if let Some(s) = suite.take() {
                    suites.push(suite_json(&s.attrs, s.cases));
                }
            }
            ("testcase", false) => {
                finish_case(&mut case, &mut suite);
                case = Some(CaseAcc {
                    attrs: parse_attrs(tag.attrs),
                    status: "passed",
                    message: None,
                    kind: None,
                    detail: None,
                });
                if tag.self_closing {
                    finish_case(&mut case, &mut suite);
                }
            }
            ("testcase", true) => finish_case(&mut case, &mut suite),
            (kind @ ("failure" | "error" | "skipped"), false) => {
                if let Some(c) = case.as_mut() {
                    let a = parse_attrs(tag.attrs);
                    c.status = match kind {
                        "failure" => "failed",
                        "error" => "error",
                        _ => "skipped",
                    };
                    c.message = attr(&a, "message").map(str::to_string);
                    c.kind = attr(&a, "type").map(str::to_string);
                    if !tag.self_closing {
                        let close = format!("</{kind}>");
                        if let Some(e) = xml[pos..].find(&close) {
                            let text = xml_unescape(xml[pos..pos + e].trim());
                            let first = text.lines().next().unwrap_or("").trim().to_string();
                            if !first.is_empty() {
                                c.detail = Some(first);
                            }
                            pos += e + close.len();
                        }
                    }
                }
            }
            _ => {}
        }
    }
    finish_case(&mut case, &mut suite);
    if let Some(s) = suite.take() {
        suites.push(suite_json(&s.attrs, s.cases));
    }
    let sum = |k: &str| {
        suites
            .iter()
            .map(|s| s[k].as_f64().unwrap_or(0.0))
            .sum::<f64>()
    };
    json!({
        "totals": {
            "suites": suites.len(),
            "tests": sum("tests") as u64,
            "failures": sum("failures") as u64,
            "errors": sum("errors") as u64,
            "skipped": sum("skipped") as u64,
            "time": sum("time"),
        },
        "suites": suites,
    })
}

fn suite_json(attrs: &[(String, String)], cases: Vec<Value>) -> Value {
    let count = |status: &str| cases.iter().filter(|c| c["status"] == status).count();
    json!({
        "name": attr(attrs, "name").unwrap_or(""),
        "package": attr(attrs, "package"),
        "timestamp": attr(attrs, "timestamp"),
        "tests": if attr(attrs, "tests").is_some() { num(attrs, "tests") as u64 } else { cases.len() as u64 },
        "failures": if attr(attrs, "failures").is_some() { num(attrs, "failures") as u64 } else { count("failed") as u64 },
        "errors": if attr(attrs, "errors").is_some() { num(attrs, "errors") as u64 } else { count("error") as u64 },
        "skipped": if attr(attrs, "skipped").is_some() { num(attrs, "skipped") as u64 } else { count("skipped") as u64 },
        "passed": count("passed"),
        "time": num(attrs, "time"),
        "cases": cases,
    })
}

fn md_cell(s: &str) -> String {
    s.replace('|', "\\|")
        .replace(['\n', '\r'], " ")
        .trim()
        .to_string()
}

/// Markdown rendering of `junit_summary`: totals, then a table per suite.
pub fn junit_markdown(summary: &Value, execution_id: &str) -> String {
    let t = &summary["totals"];
    let tests = t["tests"].as_u64().unwrap_or(0);
    let failures = t["failures"].as_u64().unwrap_or(0);
    let errors = t["errors"].as_u64().unwrap_or(0);
    let skipped = t["skipped"].as_u64().unwrap_or(0);
    let verdict = if failures + errors == 0 {
        "SUCCESS"
    } else {
        "FAILURE"
    };
    let mut out = format!(
        "# Test report {verdict} (execution {execution_id})\n\n**{tests} tests, {failures} failures, {errors} errors, {skipped} skipped** in {:.2} s across {} suite(s)\n",
        t["time"].as_f64().unwrap_or(0.0),
        t["suites"].as_u64().unwrap_or(0)
    );
    for s in summary["suites"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
    {
        let cases = s["cases"].as_array().map(Vec::as_slice).unwrap_or(&[]);
        let pkg = s["package"]
            .as_str()
            .map(|p| format!(" (package {p})"))
            .unwrap_or_default();
        out.push_str(&format!(
            "\n## {}{pkg}: {}/{} passed\n\n| # | Test | Result | Time | Details |\n|---|---|---|---|---|\n",
            s["name"].as_str().unwrap_or("?"),
            s["passed"].as_u64().unwrap_or(0),
            s["tests"].as_u64().unwrap_or(cases.len() as u64)
        ));
        for c in cases {
            let status = match c["status"].as_str().unwrap_or("") {
                "passed" => "PASS",
                "failed" => "FAIL",
                "error" => "ERROR",
                "skipped" => "SKIP",
                other => other,
            };
            let details = [c["message"].as_str(), c["detail"].as_str()]
                .into_iter()
                .flatten()
                .map(md_cell)
                .filter(|d| !d.is_empty())
                .collect::<Vec<_>>();
            let mut details = details.join(" -- ");
            if details.chars().count() > 300 {
                details = details.chars().take(300).collect::<String>() + "...";
            }
            out.push_str(&format!(
                "| {} | {} | {status} | {:.3} s | {details} |\n",
                c["id"].as_str().unwrap_or(""),
                md_cell(c["name"].as_str().unwrap_or("")),
                c["time"].as_f64().unwrap_or(0.0)
            ));
        }
    }
    out
}

impl super::ISClient {
    pub async fn test_run(
        &self,
        packages: &Value,
        test_user: Option<&str>,
        test_user_password: Option<&str>,
    ) -> Result<Value, String> {
        let mut payload = json!({"testSuitePackages": packages});
        if let Some(u) = test_user {
            payload
                .as_object_mut()
                .unwrap()
                .insert("testuser".into(), json!(u));
        }
        if let Some(p) = test_user_password {
            payload
                .as_object_mut()
                .unwrap()
                .insert("testuserpassword".into(), json!(p));
        }
        self.invoke_post("wm.task.executor:run", &payload).await
    }

    pub async fn test_check_status(&self, execution_id: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.task.executor:checkstatus",
            &json!({"executionID": execution_id}),
        )
        .await
    }

    /// Plain-text JUnit report (`text/plain` body, not JSON).
    pub async fn test_text_report(&self, execution_id: &str) -> Result<String, String> {
        self.invoke_post_text(
            "wm.task.executor:textreport",
            &json!({"executionID": execution_id}),
        )
        .await
    }

    /// JUnit XML report (`application/xml` body, not JSON), returned verbatim.
    pub async fn test_junit_report(&self, execution_id: &str) -> Result<String, String> {
        self.invoke_post_text(
            "wm.task.executor:junitxmlreport",
            &json!({"executionID": execution_id}),
        )
        .await
    }

    pub async fn mock_load(
        &self,
        scope: &str,
        service: &str,
        mock_object: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.ps.serviceMock:loadMock",
            &json!({"scope": scope, "service": service, "mockObject": mock_object}),
        )
        .await
    }

    pub async fn mock_clear(&self, scope: &str, service: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.ps.serviceMock:clearMock",
            &json!({"scope": scope, "service": service}),
        )
        .await
    }

    pub async fn mock_clear_all(&self) -> Result<Value, String> {
        self.invoke_post("wm.ps.serviceMock:clearAllMocks", &json!({}))
            .await
    }

    pub async fn mock_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.ps.serviceMock:getMockedServices").await
    }

    pub async fn mock_suspend(&self) -> Result<Value, String> {
        self.invoke_post("wm.ps.serviceMock:suspendMocks", &json!({}))
            .await
    }

    pub async fn mock_resume(&self) -> Result<Value, String> {
        self.invoke_post("wm.ps.serviceMock:resumeMocks", &json!({}))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_defaults_to_server_and_accepts_global_alias() {
        assert_eq!(normalize_mock_scope(None).unwrap(), "server");
        assert_eq!(normalize_mock_scope(Some("")).unwrap(), "server");
        assert_eq!(normalize_mock_scope(Some("global")).unwrap(), "server");
        assert_eq!(normalize_mock_scope(Some(" Server ")).unwrap(), "server");
        assert_eq!(normalize_mock_scope(Some("USER")).unwrap(), "user");
        assert_eq!(normalize_mock_scope(Some("session")).unwrap(), "session");
    }

    #[test]
    fn scope_rejects_unknown_values() {
        let err = normalize_mock_scope(Some("everywhere")).unwrap_err();
        assert!(err.contains("Invalid mock scope 'everywhere'"), "{err}");
        assert!(err.contains("'server'"), "{err}");
    }

    #[test]
    fn strips_properties_blocks_and_keeps_results() {
        let xml = "<testsuites>\n<testsuite name=\"A\" tests=\"1\">\n<properties>\n\
                   <property name=\"java.version\" value=\"21\" />\n</properties>\n\
                   <testcase name=\"t1\"><failure message=\"boom\"/></testcase>\n\
                   </testsuite>\n<testsuite name=\"B\"><properties/><testcase name=\"t2\"/>\
                   </testsuite>\n</testsuites>";
        let out = strip_junit_properties(xml);
        assert!(!out.contains("<propert"), "{out}");
        assert!(out.contains("<testcase name=\"t1\"><failure message=\"boom\"/></testcase>"));
        assert!(out.contains("<testsuite name=\"B\"><testcase name=\"t2\"/></testsuite>"));
        assert!(out.starts_with("<testsuites>") && out.ends_with("</testsuites>"));
    }

    const SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8" ?>
<testsuites>
  <testsuite errors="1" failures="1" hostname="h" id="0" name="PetstoreMocks" package="PetstoreAPI" skipped="1" tests="4" time="0.175" timestamp="2026-09-03T00:10:00">
    <properties><property name="java.version" value="21" /></properties>
    <testcase classname="com.wm.ps.test.WmTestCase" name="0.0 getPetMockedRecord" time="0.072" />
    <testcase classname="com.wm.ps.test.WmTestCase" name="0.1 getPetMockedArray" time="0.039">
      <failure message="/responseCode{200} == 200 and&#xa;/name{null} == &apos;Rex&apos;" type="junit.framework.AssertionFailedError">junit.framework.AssertionFailedError: /responseCode{200} == 200 and
	at com.wm.ps.test.DefaultResultsValidator.assertValid(DefaultResultsValidator.java:164)</failure>
    </testcase>
    <testcase classname="c" name="0.2 boom" time="0.01"><error message="x &lt; y" type="java.lang.RuntimeException">java.lang.RuntimeException: x &lt; y</error></testcase>
    <testcase classname="c" name="0.3 later" time="0"><skipped/></testcase>
    <system-out><![CDATA[Suite Started : PetstoreMocks <not a tag> ]]></system-out>
    <system-err/>
  </testsuite>
</testsuites>"#;

    #[test]
    fn junit_summary_reads_cases_and_outcomes() {
        let s = junit_summary(SAMPLE);
        assert_eq!(s["totals"]["tests"], 4);
        assert_eq!(s["totals"]["failures"], 1);
        assert_eq!(s["totals"]["errors"], 1);
        assert_eq!(s["totals"]["skipped"], 1);
        let suite = &s["suites"][0];
        assert_eq!(suite["name"], "PetstoreMocks");
        assert_eq!(suite["package"], "PetstoreAPI");
        assert_eq!(suite["passed"], 1);
        let cases = suite["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 4);
        assert_eq!(cases[0]["id"], "0.0");
        assert_eq!(cases[0]["name"], "getPetMockedRecord");
        assert_eq!(cases[0]["status"], "passed");
        assert_eq!(cases[1]["status"], "failed");
        assert_eq!(
            cases[1]["message"],
            "/responseCode{200} == 200 and\n/name{null} == 'Rex'"
        );
        assert_eq!(cases[1]["type"], "junit.framework.AssertionFailedError");
        assert_eq!(
            cases[1]["detail"],
            "junit.framework.AssertionFailedError: /responseCode{200} == 200 and"
        );
        assert_eq!(cases[2]["status"], "error");
        assert_eq!(cases[2]["message"], "x < y");
        assert_eq!(cases[3]["status"], "skipped");
        let md = junit_markdown(&s, "abc");
        assert!(
            md.starts_with("# Test report FAILURE (execution abc)"),
            "{md}"
        );
        assert!(md.contains("**4 tests, 1 failures, 1 errors, 1 skipped**"));
        assert!(md.contains("## PetstoreMocks (package PetstoreAPI): 1/4 passed"));
        assert!(md.contains("| 0.0 | getPetMockedRecord | PASS | 0.072 s |  |"));
        assert!(md.contains("| 0.1 | getPetMockedArray | FAIL | 0.039 s | /responseCode{200} == 200 and /name{null} == 'Rex' -- junit.framework.AssertionFailedError"));
        assert!(md.contains("| 0.3 | later | SKIP |"));
        assert!(junit_markdown(&junit_summary("<testsuites/>"), "x").contains("SUCCESS"));
    }

    #[test]
    fn strip_is_identity_without_properties() {
        let xml =
            "<testsuites><testsuite name=\"A\"><testcase name=\"t\"/></testsuite></testsuites>";
        assert_eq!(strip_junit_properties(xml), xml);
    }
}
