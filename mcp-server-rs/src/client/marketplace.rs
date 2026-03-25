use serde_json::Value;

const REGISTRY_BASE: &str = "https://packages.webmethods.io/rad/wx.packages:manager";

impl super::ISClient {
    // ── Marketplace (packages.webmethods.io) ───────────────────

    pub async fn marketplace_search(
        &self,
        filter: Option<&str>,
        category: Option<&str>,
        registry: Option<&str>,
    ) -> Result<Value, String> {
        let reg = registry.unwrap_or("public");
        let mut url = match filter {
            Some(f) if !f.is_empty() => {
                format!(
                    "{REGISTRY_BASE}/packages/%25{}%25?registry={reg}",
                    urlencoding::encode(f)
                )
            }
            _ => format!("{REGISTRY_BASE}/packages?registry={reg}"),
        };
        if let Some(cat) = category {
            if url.contains('?') {
                url.push_str(&format!("&category={}", urlencoding::encode(cat)));
            } else {
                url.push_str(&format!("?category={}", urlencoding::encode(cat)));
            }
        }
        let r = reqwest::Client::new()
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn marketplace_package_info(
        &self,
        package_name: &str,
        registry: Option<&str>,
    ) -> Result<Value, String> {
        let reg = registry.unwrap_or("public");
        let url = format!(
            "{REGISTRY_BASE}/package/{}/info?registry={reg}",
            urlencoding::encode(package_name)
        );
        let r = reqwest::Client::new()
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn marketplace_package_tags(
        &self,
        package_name: &str,
        registry: Option<&str>,
    ) -> Result<Value, String> {
        let reg = registry.unwrap_or("public");
        let url = format!(
            "{REGISTRY_BASE}/package/{}/tags?registry={reg}",
            urlencoding::encode(package_name)
        );
        let r = reqwest::Client::new()
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn marketplace_categories(&self, registry: Option<&str>) -> Result<Value, String> {
        let reg = registry.unwrap_or("public");
        let url = format!("{REGISTRY_BASE}/package/categories?registry={reg}");
        let r = reqwest::Client::new()
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn marketplace_registries(&self) -> Result<Value, String> {
        let url = format!("{REGISTRY_BASE}/registries");
        let r = reqwest::Client::new()
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn marketplace_package_git(
        &self,
        package_name: &str,
        registry: Option<&str>,
    ) -> Result<Value, String> {
        let reg = registry.unwrap_or("public");
        let url = format!(
            "{REGISTRY_BASE}/package/{}/git?registry={reg}",
            urlencoding::encode(package_name)
        );
        let r = reqwest::Client::new()
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }
}
