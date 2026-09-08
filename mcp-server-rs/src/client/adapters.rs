use serde_json::{Value, json};

impl super::ISClient {
    // ── Adapter Connection Management ──────────────────────────────────

    pub async fn adapter_type_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.art.admin:retrieveAdapterTypesList")
            .await
    }

    pub async fn adapter_connection_metadata(
        &self,
        adapter_type: &str,
        factory_type: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.art.dev.connection:fetchConnectionMetadata",
            &json!({
                "adapterTypeName": adapter_type,
                "connectionFactoryType": factory_type,
            }),
        )
        .await
    }

    pub async fn adapter_connection_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.art.admin.connection:listAllResources")
            .await
    }

    pub async fn adapter_connection_create(
        &self,
        connection_alias: &str,
        package_name: &str,
        adapter_type: &str,
        connection_factory_type: &str,
        connection_settings: &Value,
        connection_manager_settings: &Value,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.art.dev.connection:createConnectionNode",
            &json!({
                "connectionAlias": connection_alias,
                "packageName": package_name,
                "adapterTypeName": adapter_type,
                "connectionFactoryType": connection_factory_type,
                "connectionSettings": connection_settings,
                "connectionManagerSettings": connection_manager_settings,
            }),
        )
        .await?;
        // createConnectionNode just echoes the request pipeline back, so there is
        // nothing worth returning from the body. What the caller actually needs to
        // know is that the node is born DISABLED and is useless until enabled.
        Ok(json!({
            "status": "created",
            "connection": connection_alias,
            "state": "disabled",
            "next_step": "call adapter_connection_enable, then adapter_connection_state to confirm connectionState=enabled",
        }))
    }

    pub async fn adapter_connection_enable(&self, connection_alias: &str) -> Result<Value, String> {
        self.invoke_post(
            "pub.art.connection:enableConnection",
            &json!({"connectionAlias": connection_alias}),
        )
        .await?;
        Ok(json!({"status": "enabled", "connection": connection_alias}))
    }

    pub async fn adapter_connection_disable(
        &self,
        connection_alias: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "pub.art.connection:disableConnection",
            &json!({"connectionAlias": connection_alias}),
        )
        .await?;
        Ok(json!({"status": "disabled", "connection": connection_alias}))
    }

    pub async fn adapter_connection_state(&self, connection_alias: &str) -> Result<Value, String> {
        self.invoke_post(
            "pub.art.connection:queryConnectionState",
            &json!({"connectionAlias": connection_alias}),
        )
        .await
    }

    // ── Adapter Listener Management ────────────────────────────────────

    pub async fn adapter_listener_list(&self, adapter_type: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/pub.art.listener:listAdapterListeners"))
            .json(&json!({"adapterTypeName": adapter_type}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let text = super::read_checked(r).await?;
        serde_json::from_str(&text).map_err(|e| e.to_string())
    }

    pub async fn adapter_listener_create(
        &self,
        listener_alias: &str,
        package_name: &str,
        adapter_type: &str,
        connection_alias: &str,
        listener_settings: Option<&Value>,
    ) -> Result<Value, String> {
        let mut payload = json!({
            "listenerAlias": listener_alias,
            "packageName": package_name,
            "adapterTypeName": adapter_type,
            "connectionAlias": connection_alias,
        });
        if let Some(settings) = listener_settings {
            payload
                .as_object_mut()
                .unwrap()
                .insert("listenerSettings".into(), settings.clone());
        }
        let r = self
            .client
            .post(self.url("/invoke/wm.art.dev.listener:createListenerNode"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let text = super::read_checked(r).await?;
        let truncated: String = text.chars().take(500).collect();
        Ok(json!({"status": "created", "listener": listener_alias, "response": truncated}))
    }

    pub async fn adapter_listener_enable(&self, listener_alias: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/pub.art.listener:enableListener"))
            .json(&json!({"listenerAlias": listener_alias}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        super::read_checked(r).await?;
        Ok(json!({"status": "enabled", "listener": listener_alias}))
    }

    pub async fn adapter_listener_disable(&self, listener_alias: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/pub.art.listener:disableListener"))
            .json(&json!({"listenerAlias": listener_alias}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        super::read_checked(r).await?;
        Ok(json!({"status": "disabled", "listener": listener_alias}))
    }

    // ── Adapter Service Management ─────────────────────────────────────

    /// Create an adapter service and PROVE it exists afterwards.
    ///
    /// `wm.art.dev.service:createAdapterServiceNode` answers HTTP 200 even
    /// when the Adapter Runtime refused the node -- the refusal only goes to
    /// server.log (`[ART.117.4030] Unable to create adapter service ...`).
    /// The classic trigger is an empty JSON array in the settings: `[]`
    /// arrives as `Object[]`, the template setter wants `String[]`, and
    /// `[ART.114.542] could not set property "realInputFields" ... argument
    /// type mismatch` kills the node. Empty arrays are therefore removed
    /// from the settings before the call (reported in `omitted_empty_arrays`),
    /// and a node that is missing after the call is an error carrying the
    /// matching server.log lines.
    pub async fn adapter_service_create(
        &self,
        service_name: &str,
        package_name: &str,
        connection_alias: &str,
        service_template: &str,
        adapter_service_settings: Option<&Value>,
    ) -> Result<Value, String> {
        let mut payload = json!({
            "serviceName": service_name,
            "packageName": package_name,
            "connectionAlias": connection_alias,
            "serviceTemplate": service_template,
        });
        let mut omitted = Vec::new();
        if let Some(settings) = adapter_service_settings {
            let cleaned = strip_empty_arrays(settings, &mut omitted);
            payload
                .as_object_mut()
                .unwrap()
                .insert("adapterServiceSettings".into(), cleaned);
        }
        let r = self
            .client
            .post(self.url("/invoke/wm.art.dev.service:createAdapterServiceNode"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        super::read_checked(r).await?;

        if !self.node_exists(service_name).await? {
            let short = service_name.rsplit(':').next().unwrap_or(service_name);
            let hints: Vec<String> = self
                .log_tail("150")
                .await
                .unwrap_or_default()
                .into_iter()
                .filter(|l| l.contains(service_name) || l.contains(short) || l.contains("[ART.1"))
                .rev()
                .take(6)
                .collect();
            return Err(format!(
                "IS answered 200 but {service_name} does not exist: the Adapter Runtime refused the node                  (see server.log, [ART.117.4030]). Usual causes: a property of the wrong Java type                  (empty JSON array -> Object[], JSON number for an int/boolean property -> send strings),                  an unknown property name, or inconsistent array lengths. server.log tail: {}",
                if hints.is_empty() {
                    "(no matching lines)".to_string()
                } else {
                    hints.join(" | ")
                }
            ));
        }
        let mut out = json!({"status": "created", "service": service_name, "verified": true});
        if !omitted.is_empty() {
            out["omitted_empty_arrays"] = json!(omitted);
        }
        Ok(out)
    }

    // ── Adapter Notification Management ────────────────────────────────

    pub async fn adapter_notification_list(&self, adapter_type: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/pub.art.notification:listAdapterPollingNotifications"))
            .json(&json!({"adapterTypeName": adapter_type}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let text = super::read_checked(r).await?;
        serde_json::from_str(&text).map_err(|e| e.to_string())
    }

    pub async fn adapter_notification_create_polling(
        &self,
        notification_name: &str,
        package_name: &str,
        connection_alias: &str,
        notification_template: &str,
        notification_settings: Option<&Value>,
    ) -> Result<Value, String> {
        let mut payload = json!({
            "notificationName": notification_name,
            "packageName": package_name,
            "connectionAlias": connection_alias,
            "notificationTemplate": notification_template,
        });
        if let Some(settings) = notification_settings {
            payload
                .as_object_mut()
                .unwrap()
                .insert("notificationSettings".into(), settings.clone());
        }
        let r = self
            .client
            .post(self.url("/invoke/wm.art.dev.notification:createPollingNotificationNode"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let text = super::read_checked(r).await?;
        let truncated: String = text.chars().take(500).collect();
        Ok(json!({"status": "created", "notification": notification_name, "response": truncated}))
    }

    pub async fn adapter_notification_create_listener(
        &self,
        notification_name: &str,
        package_name: &str,
        listener_alias: &str,
        notification_template: &str,
        notification_settings: Option<&Value>,
    ) -> Result<Value, String> {
        let mut payload = json!({
            "notificationName": notification_name,
            "packageName": package_name,
            "listenerAlias": listener_alias,
            "notificationTemplate": notification_template,
        });
        if let Some(settings) = notification_settings {
            payload
                .as_object_mut()
                .unwrap()
                .insert("notificationSettings".into(), settings.clone());
        }
        let r = self
            .client
            .post(self.url("/invoke/wm.art.dev.notification:createListenerNotificationNode"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let text = super::read_checked(r).await?;
        let truncated: String = text.chars().take(500).collect();
        Ok(json!({"status": "created", "notification": notification_name, "response": truncated}))
    }

    // ── Adapter Metadata (Designer-like) ─────────────────────────────

    pub async fn adapter_service_template_list(
        &self,
        connection_alias: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.art.ns:getAdapterServiceTemplateList",
            &json!({"connectionAlias": connection_alias}),
        )
        .await
    }

    pub async fn adapter_service_template_metadata(
        &self,
        connection_alias: &str,
        service_template: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.art.dev.service:fetchAdapterServiceTemplateMetadata",
            &json!({
                "connectionAlias": connection_alias,
                "serviceTemplate": service_template,
            }),
        )
        .await
    }

    /// `wm.art.metadata:resourceDomainLookupValues`. `values` is one entry
    /// per dependency of the domain, in order. A dependency that is itself
    /// an array (the `*tables.columnInfo` kind that `updateColumnNames` /
    /// `updateColumnTypes` / `updateJDBCTypes` declare) is passed as a JSON
    /// array inside `values` and encoded the way the ART expects
    /// (see [`adapter_values_from_array`]).
    pub async fn adapter_resource_domain_lookup(
        &self,
        connection_alias: &str,
        service_template: &str,
        resource_domain_name: &str,
        values: Option<&Value>,
    ) -> Result<Value, String> {
        let mut payload = json!({
            "connectionAlias": connection_alias,
            "serviceTemplate": service_template,
            "resourceDomainName": resource_domain_name,
        });
        if let Some(v) = values {
            payload
                .as_object_mut()
                .unwrap()
                .insert("values".into(), normalize_lookup_values(v)?);
        }
        self.invoke_post("wm.art.metadata:resourceDomainLookupValues", &payload)
            .await
    }

    /// Names of the `values` of the first resource domain in a lookup answer.
    pub(crate) async fn lookup_names(
        &self,
        connection_alias: &str,
        service_template: &str,
        resource_domain_name: &str,
        values: Option<&Value>,
    ) -> Result<Vec<String>, String> {
        let v = self
            .adapter_resource_domain_lookup(
                connection_alias,
                service_template,
                resource_domain_name,
                values,
            )
            .await?;
        Ok(v.get("resourceDomainValues")
            .and_then(Value::as_array)
            .and_then(|d| d.first())
            .and_then(|d| d.get("values"))
            .and_then(Value::as_array)
            .map(|vals| {
                vals.iter()
                    .filter_map(|x| x.get("name").and_then(Value::as_str))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default())
    }

    pub async fn adapter_service_get(&self, node_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.art.ns:queryAdapterServiceData",
            &json!({"nodeName": node_name}),
        )
        .await
    }

    /// Lock a namespace node for edit, as Designer does before saving it.
    /// `updateAdapterServiceNode` refuses to run on an unlocked node:
    /// `[ART.117.4050] ... needs to be checked out or lock for edit`.
    pub async fn lock_node(&self, node_ns_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.ns:lockNode",
            &json!({"node_nsName": node_ns_name}),
        )
        .await
    }

    pub async fn unlock_node(&self, node_ns_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.ns:unLockNode",
            &json!({"node_nsName": node_ns_name}),
        )
        .await
    }

    /// Update an adapter service: lock, `updateAdapterServiceNode`, unlock
    /// (the unlock always runs, the update result is reported afterwards).
    pub async fn adapter_service_update(
        &self,
        service_name: &str,
        settings: &Value,
    ) -> Result<Value, String> {
        let mut payload = json!({"serviceName": service_name});
        if let Some(obj) = settings.as_object() {
            for (k, v) in obj {
                payload
                    .as_object_mut()
                    .unwrap()
                    .insert(k.clone(), v.clone());
            }
        }
        self.lock_node(service_name)
            .await
            .map_err(|e| format!("cannot lock {service_name} for edit: {e}"))?;
        let result = self
            .invoke_post("wm.art.dev.service:updateAdapterServiceNode", &payload)
            .await;
        let unlock = self.unlock_node(service_name).await;
        let mut v = result?;
        if let (Err(e), Some(o)) = (unlock, v.as_object_mut()) {
            o.insert(
                "warning".into(),
                json!(format!("updated, but the node could not be unlocked: {e}")),
            );
        }
        Ok(v)
    }
}

/// Remove empty JSON arrays from adapter settings (any depth), recording
/// the dotted names of what was removed. `[]` cannot be typed by the IS JSON
/// decoder, becomes `Object[]`, and makes the ART reject the whole node.
pub(crate) fn strip_empty_arrays(settings: &Value, omitted: &mut Vec<String>) -> Value {
    fn walk(v: &Value, path: &str, omitted: &mut Vec<String>) -> Option<Value> {
        match v {
            Value::Array(a) if a.is_empty() => {
                omitted.push(path.to_string());
                None
            }
            Value::Object(o) => {
                let mut out = serde_json::Map::new();
                for (k, child) in o {
                    let p = if path.is_empty() {
                        k.clone()
                    } else {
                        format!("{path}.{k}")
                    };
                    if let Some(c) = walk(child, &p, omitted) {
                        out.insert(k.clone(), c);
                    }
                }
                Some(Value::Object(out))
            }
            other => Some(other.clone()),
        }
    }
    walk(settings, "", omitted).unwrap_or(Value::Object(Default::default()))
}

/// Encode a String[] the way `com.wm.adk.metadata.AdapterValues.fromArray`
/// does: each element escaped (`\` -> `\\`, newline -> the two characters
/// `\n`), elements joined by real newlines, trailing newline included. This
/// is how the ART transports array-valued metadata (a table's `columnInfo`,
/// the `*field` dependencies of a resource-domain lookup) inside a single
/// String; `getValueSequence` splits any value containing a newline back
/// into an array.
pub fn adapter_values_from_array(items: &[String]) -> String {
    let mut out = String::new();
    for item in items {
        for c in item.chars() {
            match c {
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                other => out.push(other),
            }
        }
        out.push('\n');
    }
    out
}

/// Inverse of [`adapter_values_from_array`] (`AdapterValues.toArray`).
pub fn adapter_values_to_array(text: &str) -> Vec<String> {
    text.split('\n')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut out = String::new();
            let mut chars = s.chars();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    match chars.next() {
                        Some('n') => out.push('\n'),
                        Some(other) => out.push(other),
                        None => out.push('\\'),
                    }
                } else {
                    out.push(c);
                }
            }
            out
        })
        .collect()
}

/// Turn the tool's `values` JSON into what the ART reads: an array of
/// strings, where a nested array becomes one AdapterValues-encoded string.
pub(crate) fn normalize_lookup_values(v: &Value) -> Result<Value, String> {
    let items: Vec<Value> = match v {
        Value::Array(a) => a.clone(),
        Value::String(s) => vec![Value::String(s.clone())],
        Value::Null => vec![],
        other => {
            return Err(format!(
                "values must be a JSON array (strings, or arrays of strings for array-valued dependencies), got {other}"
            ));
        }
    };
    let mut out = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        out.push(match item {
            Value::String(s) => s.clone(),
            Value::Array(inner) => {
                let strs = inner
                    .iter()
                    .map(|x| match x {
                        Value::String(s) => Ok(s.clone()),
                        Value::Number(n) => Ok(n.to_string()),
                        Value::Bool(b) => Ok(b.to_string()),
                        other => Err(format!(
                            "values[{i}] must contain only strings, got {other}"
                        )),
                    })
                    .collect::<Result<Vec<String>, String>>()?;
                adapter_values_from_array(&strs)
            }
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => String::new(),
            other => {
                return Err(format!(
                    "values[{i}] must be a string or an array, got {other}"
                ));
            }
        });
    }
    Ok(Value::Array(out.into_iter().map(Value::String).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_arrays_are_stripped_and_reported() {
        let settings = json!({
            "sql": "SELECT 1",
            "inputField": [],
            "outputField": ["x"],
            "nested": {"a": [], "b": "keep"},
        });
        let mut omitted = Vec::new();
        let cleaned = strip_empty_arrays(&settings, &mut omitted);
        assert_eq!(omitted, vec!["inputField", "nested.a"]);
        assert_eq!(
            cleaned,
            json!({"sql": "SELECT 1", "outputField": ["x"], "nested": {"b": "keep"}})
        );
    }

    #[test]
    fn adapter_values_round_trip_matches_the_art_encoding() {
        let items = vec![
            "plain".to_string(),
            "two\nlines".to_string(),
            "back\\slash".to_string(),
        ];
        let encoded = adapter_values_from_array(&items);
        assert_eq!(encoded, "plain\ntwo\\nlines\nback\\\\slash\n");
        assert_eq!(adapter_values_to_array(&encoded), items);
    }

    #[test]
    fn nested_lookup_values_become_one_encoded_string() {
        let v = json!(["winfarm", ["id\nint4\n4\n1", "name\nvarchar\n12\n2"]]);
        let n = normalize_lookup_values(&v).unwrap();
        assert_eq!(n[0], "winfarm");
        assert_eq!(n[1], "id\\nint4\\n4\\n1\nname\\nvarchar\\n12\\n2\n");
        assert_eq!(
            normalize_lookup_values(&json!("cat")).unwrap(),
            json!(["cat"])
        );
        assert!(normalize_lookup_values(&json!({"a": 1})).is_err());
    }
}
