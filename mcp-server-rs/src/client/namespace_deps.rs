use serde_json::{Value, json};

impl super::ISClient {
    pub async fn ns_dependency_get_dependents(&self, node_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.ns.dependency:getDependents",
            &json!({"nsName": node_name}),
        )
        .await
    }

    pub async fn ns_dependency_get_references(&self, node_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.ns.dependency:getReferences",
            &json!({"nsName": node_name}),
        )
        .await
    }

    pub async fn ns_dependency_get_unresolved(&self, package_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.ns.dependency:getUnresolved",
            &json!({"package": package_name}),
        )
        .await
    }

    pub async fn ns_dependency_search(
        &self,
        search_string: &str,
        node_type: Option<&str>,
    ) -> Result<Value, String> {
        let mut payload = json!({"searchString": search_string});
        if let Some(nt) = node_type {
            payload["nodeType"] = json!(nt);
        }
        self.invoke_post("wm.server.ns.dependency:search", &payload)
            .await
    }

    pub async fn ns_dependency_refactor_preview(
        &self,
        old_name: &str,
        new_name: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.ns.dependency:refactorPreview",
            &json!({"oldName": old_name, "newName": new_name}),
        )
        .await
    }

    pub async fn ns_dependency_refactor(
        &self,
        old_name: &str,
        new_name: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.ns.dependency:refactor",
            &json!({"oldName": old_name, "newName": new_name}),
        )
        .await
    }
}
