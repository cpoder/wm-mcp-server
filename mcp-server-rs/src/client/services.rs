use super::flow_check;
use serde_json::{Value, json};
use std::collections::HashMap;

impl super::ISClient {
    // ── Service Management via putNode ─────────────────────────────────

    /// Create or update a namespace node with `wm.server.ns:putNode`.
    ///
    /// Three things happen around the raw call, each the answer to a real
    /// silent failure seen on IS 12.1:
    ///
    /// 1. The flow tree is checked against the constructs the flow compiler
    ///    knows (see [`flow_check`]): an unknown step type or key is dropped
    ///    by IS without any error, so it is refused here before anything is
    ///    written.
    /// 2. The node is created first when it does not exist yet. With
    ///    `watt.server.ns.lockingMode=full` (the default), `putNode` calls
    ///    `lockNode` before writing and a missing node fails with
    ///    `[ISS.0081.9001] Node ... does not exist`; a flow service shell is
    ///    made with `serviceAdd`, any other node type with `makeNode`.
    /// 3. When `verify` is set, the node is read back as XML (the only
    ///    encoding that carries the step tree) and the step counts per type
    ///    are compared with what was sent; a deficit is an error.
    pub async fn put_node(&self, node_data: &Value, verify: bool) -> Result<Value, String> {
        let ns_name = node_data
            .get("node_nsName")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or("node_data.node_nsName is required (\"folder.subfolder:name\")")?;
        let pkg = node_data
            .get("node_pkg")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or("node_data.node_pkg is required (the package that owns the node)")?;
        let node_type = node_data
            .get("node_type")
            .and_then(Value::as_str)
            .unwrap_or("service");

        if let Some(flow) = node_data.get("flow") {
            let problems = flow_check::validate_flow_tree(flow);
            if !problems.is_empty() {
                return Err(format!(
                    "flow tree rejected before writing (IS would drop these silently):\n- {}",
                    problems.join("\n- ")
                ));
            }
        }

        let existing = self.node_get_json(ns_name).await?;
        let created_shell = if existing.get("node").is_none_or(Value::is_null) {
            if node_type == "service" {
                let svc_type = node_data
                    .get("svc_type")
                    .and_then(Value::as_str)
                    .unwrap_or("flow");
                if svc_type != "flow" {
                    return Err(format!(
                        "{ns_name} does not exist and put_node can only create flow services \
                         (svc_type \"{svc_type}\" requested); create the node another way first"
                    ));
                }
                self.service_create(pkg, ns_name)
                    .await
                    .map_err(|e| format!("cannot create the service shell {ns_name}: {e}"))?;
            } else {
                self.make_node(node_type, pkg, ns_name)
                    .await
                    .map_err(|e| format!("cannot create the {node_type} node {ns_name}: {e}"))?;
            }
            true
        } else {
            false
        };

        let r = self
            .client
            .post(self.url("/invoke/wm.server.ns/putNode"))
            .json(node_data)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        super::read_checked(r).await?;

        let mut result = json!({
            "status": if created_shell { "created" } else { "updated" },
            "node": ns_name,
            "package": pkg,
        });
        if verify {
            let report = self.verify_node(node_data, ns_name).await?;
            let ok = report.get("status").and_then(Value::as_str) == Some("ok");
            result["verification"] = report;
            if !ok {
                return Err(format!(
                    "{ns_name} was written but the stored node differs from what was sent \
                     (steps or fields dropped by IS): {}",
                    serde_json::to_string(&result["verification"]).unwrap_or_default()
                ));
            }
        }
        Ok(result)
    }

    /// Read the node back and compare it with the payload that was sent:
    /// flow step counts per type for services, field counts for records.
    async fn verify_node(&self, sent: &Value, ns_name: &str) -> Result<Value, String> {
        let stored = self.node_get_xml(ns_name).await?;
        let node = stored.get("node").cloned().unwrap_or(Value::Null);
        if node.is_null() {
            return Ok(json!({
                "status": "mismatch",
                "detail": "the node does not exist after putNode returned HTTP 200",
            }));
        }
        if let Some(flow) = sent.get("flow") {
            let stored_flow = node.get("flow").cloned().unwrap_or(Value::Null);
            if stored_flow.get("nodes").is_some_and(Value::is_string) {
                return Ok(json!({
                    "status": "unverifiable",
                    "detail": "IS returned the flow steps as a string; node_get cannot see the tree",
                }));
            }
            return Ok(flow_check::compare_flows(flow, &stored_flow));
        }
        if sent.get("rec_fields").is_some() {
            let want = flow_check::count_fields(sent);
            let have = flow_check::count_fields(&node);
            let status = if have >= want { "ok" } else { "mismatch" };
            return Ok(json!({
                "status": status,
                "fields_sent": want,
                "fields_stored": have,
            }));
        }
        Ok(json!({"status": "ok", "detail": "node exists (no flow or record fields to compare)"}))
    }

    pub async fn service_create(&self, package: &str, service_path: &str) -> Result<Value, String> {
        let (interface_part, service_name) = if let Some(pos) = service_path.rfind(':') {
            (&service_path[..pos], &service_path[pos + 1..])
        } else {
            ("", service_path)
        };

        let mut payload: HashMap<&str, &str> = HashMap::new();
        payload.insert("service", service_name);
        payload.insert("package", package);
        payload.insert("serviceType", "flow");
        if !interface_part.is_empty() {
            payload.insert("interface", interface_part);
        }

        let r = self
            .client
            .post(self.url("/invoke/wm.server.services/serviceAdd"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        super::read_checked(r).await?;
        Ok(json!({"status": "created", "service": service_path, "package": package}))
    }

    /// `wm.server.ns:makeNode` for folders (`interface`), document types
    /// (`record`) and any other node type that can be created empty.
    pub(crate) async fn make_node(
        &self,
        node_type: &str,
        package: &str,
        ns_name: &str,
    ) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/wm.server.ns/makeNode"))
            .json(&json!({
                "node_type": node_type,
                "node_nsName": ns_name,
                "node_pkg": package,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        super::read_checked(r).await?;
        Ok(json!({"status": "created", "node": ns_name, "package": package}))
    }

    /// Invoke a service. `timeout_secs` overrides the client-wide timeout for
    /// this one call (long-running pipelines). A failed invocation returns
    /// the IS error as structured JSON text (`error`, `errorType`, `at`,
    /// `cause`, ...) rather than the raw stack dump.
    pub async fn service_invoke(
        &self,
        service_path: &str,
        inputs: Option<&Value>,
        timeout_secs: Option<u64>,
    ) -> Result<Value, String> {
        let url = self.url(&format!("/invoke/{}", service_path));
        let mut req = if let Some(body) = inputs {
            self.client.post(&url).json(body)
        } else {
            self.client.get(&url)
        };
        if let Some(t) = timeout_secs {
            req = req.timeout(std::time::Duration::from_secs(t));
        }
        let r = req.send().await.map_err(|e| e.to_string())?;
        let status = r.status();
        let text = r.text().await.map_err(|e| e.to_string())?;
        if !status.is_success() {
            return Err(match super::is_error_details(&text) {
                Some(mut d) => {
                    d["httpStatus"] = json!(status.as_u16());
                    d["service"] = json!(service_path);
                    serde_json::to_string_pretty(&d).unwrap_or_default()
                }
                None => super::describe_http_error(status, &text),
            });
        }
        if text.trim().is_empty() {
            Ok(json!({"status": "invoked"}))
        } else {
            serde_json::from_str(&text).map_err(|e| e.to_string())
        }
    }

    // ── Document Type Management ───────────────────────────────────────

    /// Create an empty document type. Idempotent: an existing node of that
    /// name answers `status: "exists"` instead of `[ISS.0085.9080] node name
    /// ... already in use`.
    pub async fn document_type_create(
        &self,
        package: &str,
        doc_path: &str,
    ) -> Result<Value, String> {
        match self.make_node("record", package, doc_path).await {
            Ok(_) => Ok(json!({"status": "created", "document": doc_path, "package": package})),
            Err(e) if is_already_exists(&e) => {
                Ok(json!({"status": "exists", "document": doc_path, "package": package}))
            }
            Err(e) => Err(e),
        }
    }
}

/// IS phrases "already there" differently per node kind:
/// `[ISS.0085.9080] node name "X" already in use` (services, records),
/// `[ISS.0085.9082] folder node "X" exists in package` (folders).
pub(crate) fn is_already_exists(err: &str) -> bool {
    err.contains("ISS.0085.9080")
        || err.contains("ISS.0085.9082")
        || err.contains("already in use")
        || err.contains("exists in package")
        || err.contains("already exists")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn already_exists_detection_covers_folder_and_node_phrasings() {
        assert!(is_already_exists(
            "HTTP 500: [ISS.0085.9082] folder node \"dup\" exists in package (ServiceException)"
        ));
        assert!(is_already_exists(
            "HTTP 500: [ISS.0085.9080] node name \"doc\" already in use (ServiceException)"
        ));
        assert!(!is_already_exists(
            "HTTP 500: [ISS.0085.9078] node belongs to invalid package"
        ));
    }
}
