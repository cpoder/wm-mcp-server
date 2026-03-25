use serde_json::{Value, json};

impl super::ISClient {
    // ── Package Management Extended ────────────────────────────

    pub async fn package_delete(&self, package_name: &str) -> Result<Value, String> {
        self.invoke_get(&format!(
            "wm.server.packages/packageDelete?package={package_name}"
        ))
        .await
    }

    pub async fn package_info(&self, package_name: &str) -> Result<Value, String> {
        self.invoke_get(&format!(
            "wm.server.packages/packageInfo?package={package_name}"
        ))
        .await
    }

    pub async fn package_dependencies(&self, package_name: &str) -> Result<Value, String> {
        self.invoke_get(&format!(
            "wm.server.packages/getDependenciesList?package={package_name}"
        ))
        .await
    }

    pub async fn package_jar_list(&self, package_name: &str) -> Result<Value, String> {
        self.invoke_get(&format!(
            "wm.server.packages/jarList?package={package_name}"
        ))
        .await
    }

    // ── Document Type Generation ───────────────────────────────

    pub async fn doctype_gen_from_json(
        &self,
        json_string: &str,
        package_name: &str,
        ifc_name: &str,
        record_name: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.record:generateFromJSONString",
            &json!({
                "jsonString": json_string,
                "packageName": package_name,
                "ifcName": ifc_name,
                "recordName": record_name,
            }),
        )
        .await
    }

    pub async fn doctype_gen_from_json_schema(
        &self,
        json_schema: &str,
        package_name: &str,
        ifc_name: &str,
        record_name: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.record:generateFromJSONSchema",
            &json!({
                "jsonSchema": json_schema,
                "packageName": package_name,
                "ifcName": ifc_name,
                "recordName": record_name,
            }),
        )
        .await
    }

    pub async fn doctype_gen_from_xsd(
        &self,
        xsd_source: &str,
        package_name: &str,
        ifc_name: &str,
        record_name: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.record:generateFromXSDSource",
            &json!({
                "xsdSource": xsd_source,
                "packageName": package_name,
                "ifcName": ifc_name,
                "recordName": record_name,
            }),
        )
        .await
    }

    pub async fn doctype_gen_from_xml(
        &self,
        xml_string: &str,
        package_name: &str,
        ifc_name: &str,
        record_name: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.record:generateFromXMLString",
            &json!({
                "xmlString": xml_string,
                "packageName": package_name,
                "ifcName": ifc_name,
                "recordName": record_name,
            }),
        )
        .await
    }

    // ── URL Aliases ────────────────────────────────────────────

    pub async fn url_alias_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.httpUrlAlias:listAlias").await
    }

    pub async fn url_alias_add(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.httpUrlAlias:addAlias", settings)
            .await
    }

    pub async fn url_alias_get(&self, alias: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.httpUrlAlias:getAlias", &json!({"alias": alias}))
            .await
    }

    pub async fn url_alias_delete(&self, alias: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.httpUrlAlias:deleteAlias",
            &json!({"alias": alias}),
        )
        .await
    }

    // ── SAP Document Type Generation ───────────────────────────

    pub async fn sap_idoc_doctype_create(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.sap.Dev:createIDocDocumentType", settings)
            .await
    }

    pub async fn sap_rfc_doctype_create(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.sap.Dev:createRFCDocumentType", settings)
            .await
    }
}
