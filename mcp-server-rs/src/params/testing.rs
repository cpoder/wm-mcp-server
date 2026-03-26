use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TestRunParam {
    #[schemars(
        description = "JSON string array of package names containing test suites to run (e.g., [\"MyPackage\"]). Or use [\"*\"] for all packages."
    )]
    pub test_suite_packages: String,
    #[schemars(description = "IS username to run tests as (default: current user)")]
    pub test_user: Option<String>,
    #[schemars(description = "Password for the test user")]
    pub test_user_password: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TestExecutionIdParam {
    #[schemars(description = "Test execution ID (returned from test_run)")]
    pub execution_id: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MockLoadParam {
    #[schemars(
        description = "Mock scope: session (current session only) or global (all sessions)"
    )]
    pub scope: String,
    #[schemars(description = "Full name of the service to mock (e.g., \"pub.math:addInts\")")]
    pub service: String,
    #[schemars(
        description = "Full name of the mock service to use instead (e.g., \"mytest:mockAddInts\")"
    )]
    pub mock_object: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MockClearParam {
    #[schemars(description = "Mock scope: session or global")]
    pub scope: String,
    #[schemars(description = "Full name of the service to unmock")]
    pub service: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
