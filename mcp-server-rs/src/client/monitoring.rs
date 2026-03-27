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

    pub async fn server_log(&self, num_lines: Option<&str>) -> Result<Value, String> {
        match num_lines {
            Some(n) => {
                self.invoke_post("wm.server.query:getPartialLog", &json!({"numLines": n}))
                    .await
            }
            None => self.invoke_get("wm.server.query:getLog").await,
        }
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
