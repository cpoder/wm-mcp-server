use serde_json::{Value, json};

impl super::ISClient {
    pub async fn server_health(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.query:getServerHealth").await
    }

    pub async fn server_stats(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.query:getStats").await
    }

    pub async fn server_settings(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.query:getSettings").await
    }

    pub async fn server_extended_settings(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.query:getExtendedSettings").await
    }

    pub async fn server_service_stats(&self, service_name: Option<&str>) -> Result<Value, String> {
        match service_name {
            Some(name) => {
                self.invoke_post(
                    "wm.server.query:getServiceStats",
                    &json!({"serviceName": name}),
                )
                .await
            }
            None => self.invoke_get("wm.server.query:getAllServiceStats").await,
        }
    }

    pub async fn server_thread_dump(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.query:getThreadDump").await
    }

    pub async fn server_session_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.query:getSessionList").await
    }

    pub async fn server_license_info(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.query:getLicenseInfo").await
    }

    /// Tail of `server.log`. `wm.server.query:getPartialLog` wants the log
    /// KEY (`log: "server"`, not the file name), a lowercase `numlines`, and
    /// `descendchecked: "true"` to count from the end; `getLog` is not used
    /// because it resolves the file as `server<yyyymmdd>.log` and fails on
    /// instances that do not rotate. Entries come back newest first from
    /// IS and are re-ordered oldest first here.
    pub async fn server_log(&self, num_lines: Option<&str>) -> Result<Value, String> {
        let n = num_lines.filter(|s| !s.trim().is_empty()).unwrap_or("200");
        let lines = self.log_tail(n).await?;
        Ok(json!({"log": "server", "numLines": lines.len(), "lines": lines}))
    }

    /// Last `n` entries of server.log, oldest first.
    pub(crate) async fn log_tail(&self, n: &str) -> Result<Vec<String>, String> {
        let v = self
            .invoke_post(
                "wm.server.query:getPartialLog",
                &json!({"log": "server", "numlines": n, "descendchecked": "true"}),
            )
            .await?;
        let mut lines: Vec<String> = v
            .get("logEntries")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(|s| s.trim_end().to_string())
                    .collect()
            })
            .unwrap_or_default();
        lines.reverse();
        Ok(lines)
    }

    pub async fn server_circuit_breaker_stats(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.query:getCircuitBreakerStats")
            .await
    }

    // ── Server Admin Operations ──────────────────────────────────

    pub async fn server_thread_interrupt(&self, thread_id: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.query:interruptThread",
            &json!({"threadId": thread_id}),
        )
        .await
    }

    pub async fn server_thread_kill(&self, thread_id: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.query:killThread",
            &json!({"threadId": thread_id}),
        )
        .await
    }

    pub async fn server_session_kill(&self, session_id: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.admin:killSession",
            &json!({"sessionID": session_id}),
        )
        .await
    }

    pub async fn server_ssl_cache_clear(&self) -> Result<Value, String> {
        self.invoke_post("wm.server.admin:clearSSLCache", &json!({}))
            .await
    }
}
