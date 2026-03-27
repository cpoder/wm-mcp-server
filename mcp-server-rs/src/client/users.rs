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

    // ── ACL Extended ─────────────────────────────────────────────

    pub async fn acl_assign(&self, node_name: &str, acl_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.access:aclAssign",
            &json!({"nodeName": node_name, "aclName": acl_name}),
        )
        .await
    }

    pub async fn acl_get_nodes_for_acl(&self, acl_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.access:getNodeNameListForAcl",
            &json!({"aclName": acl_name}),
        )
        .await
    }

    pub async fn acl_get_default_access(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.access:getDefaultAccess").await
    }

    pub async fn acl_set_default_access(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.access:setDefaultAccess", settings)
            .await
    }

    // ── Account Locking Extended ─────────────────────────────────

    pub async fn account_locking_update(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.access:updateAccountLockingSettings", settings)
            .await
    }

    pub async fn account_locking_reset(&self) -> Result<Value, String> {
        self.invoke_post("wm.server.access:resetAccountLockingSettings", &json!({}))
            .await
    }

    pub async fn account_locked_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.access:listLockedAccounts").await
    }

    pub async fn account_unlock(&self, username: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.access:unlockAccount",
            &json!({"username": username}),
        )
        .await
    }
}
