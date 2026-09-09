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

    /// `wm.server.access:aclAssign(target, acl?, browseaclgroup?, readaclgroup?,
    /// writeaclgroup?)` -- `acl` is the EXECUTE ACL. The service answers
    /// HTTP 200 whatever happens and only its `message` tells the truth
    /// (`Changed permissions for <node>` on success; with the wrong parameter
    /// names it used to say `Cannot invoke NSName.getFullName() because
    /// "nsName" is null` and change nothing). The node is read back through
    /// `getNodeNameListForAcl` to prove the execute ACL took.
    pub async fn acl_assign(
        &self,
        node_name: &str,
        acl_name: &str,
        browse_acl: Option<&str>,
        read_acl: Option<&str>,
        write_acl: Option<&str>,
    ) -> Result<Value, String> {
        // IS stores ANY name as the ACL, existing or not, and the node then
        // answers 403 to everyone: check the names against aclList first.
        let known: Vec<String> = self
            .acl_list()
            .await?
            .get("aclgroups")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|g| g.get("name").and_then(Value::as_str))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        for name in [Some(acl_name), browse_acl, read_acl, write_acl]
            .into_iter()
            .flatten()
        {
            if !name.is_empty() && !known.iter().any(|k| k == name) {
                return Err(format!(
                    "ACL \"{name}\" does not exist (IS would store it anyway and lock the node out); existing ACLs: {}",
                    known.join(", ")
                ));
            }
        }
        let mut payload = json!({"target": node_name, "acl": acl_name});
        for (key, value) in [
            ("browseaclgroup", browse_acl),
            ("readaclgroup", read_acl),
            ("writeaclgroup", write_acl),
        ] {
            if let Some(v) = value.filter(|v| !v.is_empty()) {
                payload[key] = json!(v);
            }
        }
        let v = self
            .invoke_post("wm.server.access:aclAssign", &payload)
            .await?;
        let message = v
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if !message.starts_with("Changed permissions") {
            return Err(format!(
                "aclAssign did not change {node_name}: IS answered \"{message}\" (does the node exist? is \"{acl_name}\" an existing ACL? see acl_list)"
            ));
        }
        let listed = self
            .acl_get_nodes_for_acl(acl_name)
            .await
            .ok()
            .and_then(|l| {
                l.get("nameList")
                    .and_then(Value::as_array)
                    .map(|a| a.iter().any(|n| n.as_str() == Some(node_name)))
            });
        let mut out = json!({
            "status": "assigned",
            "node": node_name,
            "execute_acl": acl_name,
            "message": message,
        });
        if let Some(b) = browse_acl {
            out["browse_acl"] = json!(b);
        }
        if let Some(r) = read_acl {
            out["read_acl"] = json!(r);
        }
        if let Some(w) = write_acl {
            out["write_acl"] = json!(w);
        }
        if let Some(seen) = listed {
            out["verified"] = json!(seen);
        }
        Ok(out)
    }

    /// `wm.server.access:getNodeNameListForAcl(acl)` -> `nameList`.
    pub async fn acl_get_nodes_for_acl(&self, acl_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.access:getNodeNameListForAcl",
            &json!({"acl": acl_name}),
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
