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

    pub async fn marketplace_install(
        &self,
        package_name: &str,
        tag: Option<&str>,
        registry: Option<&str>,
    ) -> Result<Value, String> {
        // Step 1: Get git info
        let reg = registry.unwrap_or("public");
        let git_url = format!(
            "{REGISTRY_BASE}/package/{}/git?registry={reg}",
            urlencoding::encode(package_name)
        );
        let git_info: Value = reqwest::Client::new()
            .get(&git_url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Failed to get git info: {e}"))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse git info: {e}"))?;

        let owner = git_info
            .get("repoOwner")
            .and_then(|v| v.as_str())
            .ok_or("Missing repoOwner in git info")?;
        let repo = git_info
            .get("repoName")
            .and_then(|v| v.as_str())
            .ok_or("Missing repoName in git info")?;

        // Step 2: Determine tag (use provided or get latest from tags)
        let install_tag = if let Some(t) = tag {
            t.to_string()
        } else {
            let tags_url = format!(
                "{REGISTRY_BASE}/package/{}/tags?registry={reg}",
                urlencoding::encode(package_name)
            );
            let tags: Value = reqwest::Client::new()
                .get(&tags_url)
                .header("Accept", "application/json")
                .send()
                .await
                .map_err(|e| format!("Failed to get tags: {e}"))?
                .json()
                .await
                .map_err(|e| format!("Failed to parse tags: {e}"))?;
            tags.get("availableTags")
                .and_then(|v| v.as_array())
                .and_then(|a| a.last())
                .and_then(|v| v.get("tag"))
                .and_then(|v| v.as_str())
                .unwrap_or("main")
                .to_string()
        };

        // Step 3: Get IS packages directory
        let paths: Value = self
            .invoke_get("wm.server.query:getServerPaths")
            .await
            .map_err(|e| format!("Failed to get server paths: {e}"))?;
        let packages_dir = paths
            .get("packagesDir")
            .and_then(|v| v.as_str())
            .ok_or("Cannot determine IS packages directory")?;

        // Step 4: Download the zip from GitHub
        let zip_url = format!(
            "https://github.com/{owner}/{repo}/archive/refs/tags/{}.zip",
            urlencoding::encode(&install_tag)
        );
        let zip_bytes = reqwest::Client::new()
            .get(&zip_url)
            .send()
            .await
            .map_err(|e| format!("Failed to download: {e}"))?
            .bytes()
            .await
            .map_err(|e| format!("Failed to read zip: {e}"))?;

        // Step 5: Extract to temp dir, find the package root, copy to IS
        let temp_dir = std::env::temp_dir().join(format!("wm_pkg_{package_name}"));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp dir: {e}"))?;

        let cursor = std::io::Cursor::new(zip_bytes.as_ref());
        let mut archive =
            zip::ZipArchive::new(cursor).map_err(|e| format!("Failed to open zip: {e}"))?;

        // Extract all files
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read zip entry: {e}"))?;
            let path = file
                .enclosed_name()
                .ok_or("Invalid zip entry name")?
                .to_owned();

            // Remap: remove the "RepoName-tag/" prefix and replace with package_name
            let components: Vec<_> = path.components().collect();
            if components.len() < 2 {
                continue; // skip root dir entry
            }
            let relative: std::path::PathBuf = components[1..].iter().collect();
            let target = std::path::Path::new(packages_dir)
                .join(package_name)
                .join(&relative);

            if file.is_dir() {
                std::fs::create_dir_all(&target)
                    .map_err(|e| format!("Failed to create dir: {e}"))?;
            } else {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent dir: {e}"))?;
                }
                let mut outfile = std::fs::File::create(&target)
                    .map_err(|e| format!("Failed to create file: {e}"))?;
                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| format!("Failed to write file: {e}"))?;
            }
        }

        // Cleanup temp
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Step 6: Activate the package
        let activate_result = self
            .invoke_get(&format!(
                "wm.server.packages/packageActivate?package={package_name}"
            ))
            .await;

        Ok(serde_json::json!({
            "status": "installed",
            "package": package_name,
            "tag": install_tag,
            "source": format!("https://github.com/{owner}/{repo}"),
            "packagesDir": packages_dir,
            "activateResult": activate_result.unwrap_or(serde_json::json!({"note": "activate may need package_reload"})),
        }))
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
