use serde_json::{Value, json};

impl super::ISClient {
    pub async fn install_jars(
        &self,
        jars: &Value,
        package_name: &str,
        description: &str,
        bounce: bool,
    ) -> Result<Value, String> {
        // Step 1: Get IS packages directory
        let paths: Value = self
            .invoke_get("wm.server.query:getServerPaths")
            .await
            .map_err(|e| format!("Failed to get server paths: {e}"))?;
        let packages_dir = paths
            .get("packagesDir")
            .and_then(|v| v.as_str())
            .ok_or("Cannot determine IS packages directory")?;

        let pkg_path = std::path::Path::new(packages_dir).join(package_name);
        let jars_dir = pkg_path.join("code").join("jars").join("static");

        // Step 2: Create package directory structure
        std::fs::create_dir_all(&jars_dir)
            .map_err(|e| format!("Failed to create package dir: {e}"))?;

        // Step 3: Create manifest.v3
        let manifest = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>

<Values version="2.0">
  <value name="enabled">yes</value>
  <value name="system_package">no</value>
  <value name="version">1.0</value>
  <value name="name">{package_name}</value>
  <null name="startup_services"/>
  <null name="shutdown_services"/>
  <null name="replication_services"/>
  <null name="requires"/>
  <null name="listACL"/>
  <value name="webappLoad">yes</value>
  <value name="reloadWithDependentPackage">yes</value>
  <null name="build"/>
  <value name="description">{description}</value>
</Values>"#
        );
        std::fs::write(pkg_path.join("manifest.v3"), manifest)
            .map_err(|e| format!("Failed to write manifest: {e}"))?;

        // Step 4: Download each JAR
        let jar_list = jars.as_array().ok_or("jars must be a JSON array")?;

        let mut installed_jars = Vec::new();
        let client = reqwest::Client::new();

        for jar_spec in jar_list {
            let (url, filename) = if let Some(maven) =
                jar_spec.get("maven").and_then(|v| v.as_str())
            {
                // Parse maven coordinates: groupId:artifactId:version
                let parts: Vec<&str> = maven.split(':').collect();
                if parts.len() != 3 {
                    return Err(format!(
                        "Invalid maven coordinates '{maven}': expected groupId:artifactId:version"
                    ));
                }
                let (group, artifact, version) = (parts[0], parts[1], parts[2]);
                let group_path = group.replace('.', "/");
                let url = format!(
                    "https://repo1.maven.org/maven2/{group_path}/{artifact}/{version}/{artifact}-{version}.jar"
                );
                let filename = format!("{artifact}-{version}.jar");
                (url, filename)
            } else if let Some(url) = jar_spec.get("url").and_then(|v| v.as_str()) {
                let filename = url
                    .rsplit('/')
                    .next()
                    .unwrap_or("downloaded.jar")
                    .to_string();
                (url.to_string(), filename)
            } else {
                return Err("Each jar entry must have 'maven' or 'url' field".into());
            };

            let response = client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("Failed to download {url}: {e}"))?;

            if !response.status().is_success() {
                return Err(format!(
                    "Failed to download {url}: HTTP {}",
                    response.status()
                ));
            }

            let bytes = response
                .bytes()
                .await
                .map_err(|e| format!("Failed to read {url}: {e}"))?;

            let jar_path = jars_dir.join(&filename);
            std::fs::write(&jar_path, &bytes)
                .map_err(|e| format!("Failed to write {filename}: {e}"))?;

            installed_jars.push(json!({
                "filename": filename,
                "size": bytes.len(),
                "source": url,
            }));
        }

        // Step 5: Activate the package
        let activate_result = self
            .invoke_get(&format!(
                "wm.server.packages/packageActivate?package={package_name}"
            ))
            .await;

        // Step 6: Bounce IS if requested
        let bounce_result = if bounce {
            match self.shutdown(true).await {
                Ok(v) => Some(v),
                Err(e) => Some(json!({"error": e})),
            }
        } else {
            None
        };

        Ok(json!({
            "status": "installed",
            "package": package_name,
            "packagesDir": packages_dir,
            "jars": installed_jars,
            "activateResult": activate_result.unwrap_or(json!({"note": "activation attempted"})),
            "bounce": bounce,
            "bounceResult": bounce_result,
            "note": if bounce { "IS is restarting. Wait ~30s before reconnecting." } else { "JARs installed but IS NOT restarted. JARs won't be on classpath until IS bounces." },
        }))
    }
}
