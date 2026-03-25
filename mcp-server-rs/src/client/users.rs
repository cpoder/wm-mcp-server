use serde_json::{Value, json};

impl super::ISClient {
    pub async fn user_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.access:userList").await
    }

    pub async fn user_add(&self, username: &str, password: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.access:userAdd",
            &json!({"username": username, "password": password}),
        )
        .await
    }

    pub async fn user_delete(&self, username: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.access:userDelete",
            &json!({"username": username}),
        )
        .await
    }

    pub async fn user_set_disabled(&self, username: &str, disabled: bool) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.access:setUserDisabled",
            &json!({"userName": username, "disabled": disabled.to_string()}),
        )
        .await
    }

    pub async fn group_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.access:groupList").await
    }

    pub async fn group_add(&self, groupname: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.access:groupAdd",
            &json!({"groupname": groupname}),
        )
        .await
    }

    pub async fn group_delete(&self, groupname: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.access:groupDelete",
            &json!({"groupname": groupname}),
        )
        .await
    }

    pub async fn group_change(&self, groupname: &str, membership: &Value) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.access:groupChange",
            &json!({"groupname": groupname, "membership": membership}),
        )
        .await
    }

    pub async fn acl_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.access:aclList").await
    }

    pub async fn acl_add(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.access:aclAdd", settings).await
    }

    pub async fn acl_delete(&self, acl_name: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.access:aclDelete", &json!({"aclName": acl_name}))
            .await
    }

    pub async fn account_locking_get(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.access:getAccountLockingSettings")
            .await
    }

    pub async fn disabled_user_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.access:getDisabledUserList")
            .await
    }
}
