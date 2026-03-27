use serde_json::{Value, json};

impl super::ISClient {
    pub async fn flatfile_schema_save(
        &self,
        xml_content: &str,
        package_name: &str,
        schema_name: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "pub.flatFile.generate:saveXMLAsFFSchema",
            &json!({
                "xmlData": xml_content,
                "packageName": package_name,
                "schemaName": schema_name,
            }),
        )
        .await
    }

    pub async fn flatfile_dictionary_create(
        &self,
        package_name: &str,
        dictionary_name: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "pub.flatFile.generate:createFFDictionary",
            &json!({
                "packageName": package_name,
                "dictionaryName": dictionary_name,
            }),
        )
        .await
    }

    pub async fn flatfile_schema_get(
        &self,
        package_name: &str,
        schema_name: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "pub.flatFile.generate:getFFSchemaAsXML",
            &json!({
                "packageName": package_name,
                "schemaName": schema_name,
            }),
        )
        .await
    }

    pub async fn flatfile_schema_delete(
        &self,
        package_name: &str,
        schema_name: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "pub.flatFile.generate:deleteFFSchema",
            &json!({
                "packageName": package_name,
                "schemaName": schema_name,
            }),
        )
        .await
    }
}
