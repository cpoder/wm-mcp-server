use serde_json::{Value, json};

impl super::ISClient {
    pub async fn jdbc_pool_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.jdbcpool:getPoolDefinitions")
            .await
    }

    pub async fn jdbc_pool_add(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.jdbcpool:addPoolAlias", settings)
            .await
    }

    pub async fn jdbc_pool_update(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.jdbcpool:updatePoolAlias", settings)
            .await
    }

    pub async fn jdbc_pool_delete(&self, pool: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.jdbcpool:deletePoolAlias", &json!({"pool": pool}))
            .await
    }

    pub async fn jdbc_pool_test(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.jdbcpool:testPoolAlias", settings)
            .await
    }

    pub async fn jdbc_pool_restart(&self, pool: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.jdbcpool:restart", &json!({"pool": pool}))
            .await
    }

    pub async fn jdbc_driver_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.jdbcpool:getDriverDefinitions")
            .await
    }

    pub async fn jdbc_function_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.jdbcpool:getFunctionalDefinitions")
            .await
    }
}
