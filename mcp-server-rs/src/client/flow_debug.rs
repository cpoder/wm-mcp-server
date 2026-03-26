use serde_json::{Value, json};

impl super::ISClient {
    pub async fn flow_debug_start(
        &self,
        service: &str,
        pipeline: Option<&Value>,
        stop_at_start: bool,
    ) -> Result<Value, String> {
        let mut payload = json!({
            "$service": service,
            "$stopAtStart": stop_at_start.to_string(),
        });
        if let Some(pipe) = pipeline {
            payload
                .as_object_mut()
                .unwrap()
                .insert("$pipeline".into(), pipe.clone());
        }
        self.invoke_post("wm.server.flowdebugger:start", &payload)
            .await
    }

    pub async fn flow_debug_execute(
        &self,
        debug_oid: &str,
        command: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.flowdebugger:execute",
            &json!({
                "$debugoid": debug_oid,
                "$debugCommand": command,
            }),
        )
        .await
    }

    pub async fn flow_debug_close(&self, debug_oid: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.flowdebugger:close",
            &json!({"$debugoid": debug_oid}),
        )
        .await
    }

    pub async fn flow_debug_insert_breakpoints(
        &self,
        debug_oid: &str,
        breakpoints: &Value,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.flowdebugger:insertBreakPoints",
            &json!({
                "$debugoid": debug_oid,
                "$breakpoints": breakpoints,
            }),
        )
        .await
    }

    pub async fn flow_debug_remove_all_breakpoints(
        &self,
        debug_oid: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.flowdebugger:removeAllBreakPoints",
            &json!({"$debugoid": debug_oid}),
        )
        .await
    }

    pub async fn flow_debug_set_pipeline(
        &self,
        debug_oid: &str,
        pipeline: &Value,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.flowdebugger:setPipeline",
            &json!({
                "$debugoid": debug_oid,
                "$pipeline": pipeline,
            }),
        )
        .await
    }

    pub async fn flow_debug_stop_service(&self, debug_oid: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.flowdebugger:stopInvokedService",
            &json!({"$debugoid": debug_oid}),
        )
        .await
    }
}
