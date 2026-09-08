//! webMethods Integration Server HTTP Client
//!
//! Pure HTTP client for interacting with the IS REST API.
//! All operations use the IS HTTP API - no disk access required.

mod adapters;
mod alerts;
mod auditing;
mod enterprise_gw;
pub(crate) mod flow_check;
mod flow_debug;
mod flow_gen;
mod global_vars;
mod health;
pub(crate) mod idata_xml;
mod ip_access;
mod jar_installer;
mod jdbc_pools;
mod jdbc_services;
pub use jdbc_services::SqlField;
mod jms;
mod jndi;
mod jwt;
mod marketplace;
mod monitoring;
mod mqtt;
mod namespace;
mod oauth;
mod packages;
mod packages_ext;
mod password_policy;
mod ports;
mod proxy;
mod quiesce;
mod remote_servers;
mod scheduler;
mod security;
mod services;
mod sftp;
mod streaming;
mod test_suites;
mod testing;
pub use test_suites::{SuiteCreateOptions, SuiteMode, TestCaseSpec};
pub use testing::{
    SESSION_SCOPE_WARNING, junit_markdown, junit_summary, normalize_mock_scope,
    strip_junit_properties,
};
mod triggers;
mod users;
mod webservices;
mod websocket;

mod cache;
mod flatfile;
mod ldap;
mod logger;
mod namespace_deps;
mod outbound_passwords;
mod port_access;
mod saml;

use reqwest::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue};
use serde_json::{Value, json};

pub struct ISClient {
    base_url: String,
    client: Client,
}

impl ISClient {
    pub fn new(base_url: &str, username: &str, password: &str, timeout_secs: u64) -> Self {
        use base64::{Engine, prelude::BASE64_STANDARD};
        let credentials = format!("{username}:{password}");
        let encoded = BASE64_STANDARD.encode(credentials.as_bytes());

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {encoded}")).expect("valid header"),
        );

        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    // ── Internal helpers ───────────────────────────────────────────────

    pub(crate) async fn invoke_get(&self, service: &str) -> Result<Value, String> {
        let r = self
            .client
            .get(self.url(&format!("/invoke/{service}")))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let text = read_checked(r).await?;
        if text.trim().is_empty() {
            Ok(json!({"status": "ok"}))
        } else {
            serde_json::from_str(&text).map_err(|e| e.to_string())
        }
    }

    /// GET an IS service and ask for the answer as `IDataXMLCoder` XML
    /// (`Accept: text/xml`) instead of JSON, decoded back to JSON by
    /// [`idata_xml::idata_xml_to_json`]. Needed wherever the pipeline holds
    /// objects the JSON encoder cannot render (flow step trees).
    pub(crate) async fn invoke_get_xml(
        &self,
        service: &str,
        query: &[(&str, &str)],
    ) -> Result<Value, String> {
        let r = self
            .client
            .get(self.url(&format!("/invoke/{service}")))
            .query(query)
            .header(ACCEPT, "text/xml")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let text = read_checked(r).await?;
        idata_xml::idata_xml_to_json(&text)
    }

    pub(crate) async fn invoke_post(
        &self,
        service: &str,
        payload: &Value,
    ) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url(&format!("/invoke/{service}")))
            .json(payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let text = read_checked(r).await?;
        if text.trim().is_empty() {
            Ok(json!({"status": "ok"}))
        } else {
            serde_json::from_str(&text).map_err(|e| e.to_string())
        }
    }

    /// POST to an IS service whose successful response is NOT JSON -- for
    /// example the WmUnitTestManager report services, which write the
    /// report through `responseString` + `HTTPServerUtil.setResponse2` as
    /// `application/xml` / `text/plain`. Errors still arrive as JSON and are
    /// surfaced by `read_checked`; the body is returned verbatim otherwise.
    pub(crate) async fn invoke_post_text(
        &self,
        service: &str,
        payload: &Value,
    ) -> Result<String, String> {
        let r = self
            .client
            .post(self.url(&format!("/invoke/{service}")))
            .json(payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        read_checked(r).await
    }
}

/// Read a response body, converting any non-2xx status into an error that
/// INCLUDES the IS response body. webMethods returns the real cause of a
/// failure (invalid WmPath, missing document type, flow-compiler stack
/// trace, "node already exists", ...) in the body; `error_for_status` drops
/// it, leaving the caller with only a generic "500" it cannot act on.
pub(crate) async fn read_checked(r: reqwest::Response) -> Result<String, String> {
    let status = r.status();
    let body = r.text().await.map_err(|e| e.to_string())?;
    if status.is_success() {
        Ok(body)
    } else {
        Err(describe_http_error(status, &body))
    }
}

/// One-line description of a failed IS call: the IS error message, its
/// message id, the exception class and the innermost IS frame when the body
/// is the usual `{"$error": ..., "$errorDump": ...}` document, the raw
/// (truncated) body otherwise. The stack dump itself is deliberately left
/// out: it is thousands of characters of `com.wm.app.b2b.server.invoke.*`
/// frames that never say more than the first line already does.
pub(crate) fn describe_http_error(status: reqwest::StatusCode, body: &str) -> String {
    if let Some(summary) = summarize_is_error(body) {
        return format!("HTTP {status}: {summary}");
    }
    let snippet: String = body.trim().chars().take(2000).collect();
    if snippet.is_empty() {
        format!("HTTP {status}")
    } else {
        format!("HTTP {status}: {snippet}")
    }
}

/// Parse an IS error document. Returns `{error, errorType, errorMsgId?,
/// service?, at?}` -- `at` is the first stack frame outside the generic
/// invoke machinery, i.e. the method that actually threw.
pub(crate) fn is_error_details(body: &str) -> Option<Value> {
    let v: Value = serde_json::from_str(body.trim()).ok()?;
    let error = v.get("$error")?.as_str()?.to_string();
    let mut details = serde_json::Map::new();
    details.insert("error".into(), Value::String(error));
    for (src, dst) in [
        ("$errorType", "errorType"),
        ("$errorMsgId", "errorMsgId"),
        ("$service", "service"),
    ] {
        if let Some(s) = v
            .get(src)
            .or_else(|| v.get("$errorInfo").and_then(|i| i.get(src)))
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        {
            details.insert(dst.into(), Value::String(s.to_string()));
        }
    }
    if let Some(dump) = v.get("$errorDump").and_then(Value::as_str) {
        let frame = dump
            .lines()
            .map(str::trim)
            .filter_map(|l| l.strip_prefix("at "))
            .find(|f| {
                !f.starts_with("com.wm.app.b2b.server.")
                    && !f.starts_with("java.")
                    && !f.starts_with("jdk.")
                    && !f.starts_with("wm.bci.")
                    && !f.starts_with("com.wm.ps.")
                    && !f.starts_with("com.wm.util.pool.")
            })
            .map(|f| f.split('(').next().unwrap_or(f).trim().to_string());
        if let Some(f) = frame {
            details.insert("at".into(), Value::String(f));
        }
        // The cause chain often carries the real reason (JDBC SQLState,
        // "argument type mismatch", ...); keep its first line.
        if let Some(cause) = dump
            .lines()
            .map(str::trim)
            .find_map(|l| l.strip_prefix("Caused by: "))
        {
            details.insert("cause".into(), Value::String(cause.to_string()));
        }
    }
    Some(Value::Object(details))
}

pub(crate) fn summarize_is_error(body: &str) -> Option<String> {
    let d = is_error_details(body)?;
    let mut s = d["error"].as_str().unwrap_or_default().to_string();
    let mut extras = Vec::new();
    if let Some(t) = d.get("errorType").and_then(Value::as_str) {
        extras.push(t.rsplit('.').next().unwrap_or(t).to_string());
    }
    if let Some(at) = d.get("at").and_then(Value::as_str) {
        extras.push(format!("at {at}"));
    }
    if !extras.is_empty() {
        s.push_str(&format!(" ({})", extras.join(" ")));
    }
    if let Some(c) = d.get("cause").and_then(Value::as_str) {
        s.push_str(&format!("; caused by: {c}"));
    }
    Some(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    const IS_ERROR: &str = r#"{"$error":"[ISS.0081.9001] Node a:b does not exist","$errorType":"com.wm.app.b2b.server.ServiceException","$errorDump":"com.wm.app.b2b.server.ServiceException: [ISS.0081.9001] Node a:b does not exist\n\tat wm.server.nsimpl.lockNode(nsimpl.java:418)\n\tat wm.server.nsimpl.putNode(nsimpl.java:5768)\n\tat java.base/jdk.internal.reflect.DirectMethodHandleAccessor.invoke(DirectMethodHandleAccessor.java:103)\n\tat com.wm.app.b2b.server.JavaService.baseInvoke(JavaService.java:417)\n","$errorInfo":{"$errorMsgId":"ISS.0081.9001","$service":"wm.server.ns:putNode"}}"#;

    #[test]
    fn is_error_is_summarized_to_one_line_with_the_throwing_frame() {
        let s = summarize_is_error(IS_ERROR).unwrap();
        assert_eq!(
            s,
            "[ISS.0081.9001] Node a:b does not exist (ServiceException at wm.server.nsimpl.lockNode)"
        );
        let d = is_error_details(IS_ERROR).unwrap();
        assert_eq!(d["errorMsgId"], "ISS.0081.9001");
        assert_eq!(d["service"], "wm.server.ns:putNode");
        assert!(d.get("cause").is_none());
    }

    #[test]
    fn cause_chain_is_kept() {
        let body = r#"{"$error":"[ART.117.4002] boom","$errorType":"com.wm.app.b2b.server.ServiceException","$errorDump":"x\n\tat com.wm.app.b2b.server.invoke.InvokeManager.process(InvokeManager.java:1)\nCaused by: java.sql.SQLException: (07009/0) Invalid parameter binding(s)\n\tat foo.Bar.baz(Bar.java:1)\n"}"#;
        let s = summarize_is_error(body).unwrap();
        assert!(s.starts_with("[ART.117.4002] boom (ServiceException at foo.Bar.baz)"));
        assert!(s.ends_with(
            "; caused by: java.sql.SQLException: (07009/0) Invalid parameter binding(s)"
        ));
    }

    #[test]
    fn non_is_bodies_fall_back_to_a_snippet() {
        assert!(summarize_is_error("<html>401</html>").is_none());
        assert!(summarize_is_error("").is_none());
        let msg = describe_http_error(reqwest::StatusCode::UNAUTHORIZED, "<html>nope</html>");
        assert_eq!(msg, "HTTP 401 Unauthorized: <html>nope</html>");
        let msg = describe_http_error(reqwest::StatusCode::NOT_FOUND, "   ");
        assert_eq!(msg, "HTTP 404 Not Found");
    }
}
