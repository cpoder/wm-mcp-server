use serde_json::{Value, json};

impl super::ISClient {
    pub async fn global_var_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.globalvariables:listGlobalVariables")
            .await
    }

    pub async fn global_var_get(&self, key: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.globalvariables:getGlobalVariableValue",
            &json!({"key": key}),
        )
        .await
    }

    pub async fn global_var_add(
        &self,
        key: &str,
        value: &str,
        is_password: bool,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.globalvariables:addGlobalVariable",
            &json!({"key": key, "value": value, "isPassword": is_password.to_string()}),
        )
        .await
    }

    pub async fn global_var_edit(&self, key: &str, value: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.globalvariables:editGlobalVariable",
            &json!({"key": key, "value": value}),
        )
        .await
    }

    pub async fn global_var_remove(&self, key: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.globalvariables:removeGlobalVariable",
            &json!({"key": key}),
        )
        .await
    }
}
