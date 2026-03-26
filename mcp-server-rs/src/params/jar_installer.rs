use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InstallJarsParam {
    #[schemars(
        description = "JSON array of JAR sources to install. Each entry is an object with:\n- 'url': direct download URL, OR\n- 'maven': Maven coordinates as 'groupId:artifactId:version'\nExample: [{\"maven\":\"com.mysql:mysql-connector-j:9.2.0\"}, {\"url\":\"https://example.com/my.jar\"}]"
    )]
    pub jars: String,
    #[schemars(
        description = "Name for the installer package (e.g., 'WmMySQLDriver'). This package will contain the JARs in code/jars/static/."
    )]
    pub package_name: String,
    #[schemars(description = "Description of the package")]
    pub description: Option<String>,
    #[schemars(
        description = "Whether to bounce (restart) IS after installing (default: true). Required for JARs to be on classpath."
    )]
    pub bounce: Option<bool>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
