use serde_json::{Value, json};

impl super::ISClient {
    pub async fn scheduler_state(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.schedule:getSchedulerState")
            .await
    }

    pub async fn scheduler_task_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.schedule:getUserTaskList").await
    }

    pub async fn scheduler_task_add(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.schedule:addTask", settings)
            .await
    }

    pub async fn scheduler_task_get(&self, oid: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.schedule:getUserTask", &json!({"oid": oid}))
            .await
    }

    pub async fn scheduler_task_update(
        &self,
        oid: &str,
        settings: &Value,
    ) -> Result<Value, String> {
        let mut payload = json!({"oid": oid});
        if let Some(obj) = settings.as_object() {
            for (k, v) in obj {
                payload
                    .as_object_mut()
                    .unwrap()
                    .insert(k.clone(), v.clone());
            }
        }
        self.invoke_post("wm.server.schedule:updateTask", &payload)
            .await
    }

    pub async fn scheduler_task_cancel(&self, oid: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.schedule:cancelUserTask", &json!({"oid": oid}))
            .await
    }

    pub async fn scheduler_task_suspend(&self, oid: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.schedule:suspendUserTask", &json!({"oid": oid}))
            .await
    }

    pub async fn scheduler_task_resume(&self, oid: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.schedule:wakeupUserTask", &json!({"oid": oid}))
            .await
    }

    pub async fn scheduler_pause(&self) -> Result<Value, String> {
        self.invoke_post("wm.server.schedule:pauseScheduler", &json!({}))
            .await
    }

    pub async fn scheduler_resume(&self) -> Result<Value, String> {
        self.invoke_post("wm.server.schedule:resumeScheduler", &json!({}))
            .await
    }
}
