use serde_json::{Value, json};

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

    pub async fn test_text_report(&self, execution_id: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.task.executor:textreport",
            &json!({"executionID": execution_id}),
        )
        .await
    }

    pub async fn test_junit_report(&self, execution_id: &str) -> Result<Value, String> {
        self.invoke_post(
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
