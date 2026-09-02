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
        if let Some(settings) = adapter_service_settings {
            payload
                .as_object_mut()
                .unwrap()
                .insert("adapterServiceSettings".into(), settings.clone());
        }
        let r = self
            .client
            .post(self.url("/invoke/wm.art.dev.service:createAdapterServiceNode"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let text = super::read_checked(r).await?;
        let truncated: String = text.chars().take(500).collect();
        Ok(json!({"status": "created", "service": service_name, "response": truncated}))
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
                .insert("values".into(), v.clone());
        }
        self.invoke_post("wm.art.metadata:resourceDomainLookupValues", &payload)
            .await
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
