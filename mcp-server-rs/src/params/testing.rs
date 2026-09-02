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
pub struct TestReportParam {
    #[schemars(description = "Test execution ID (returned from test_run)")]
    pub execution_id: String,
    #[schemars(
        description = "markdown (default): totals plus one table per suite with each test case, its result, duration and failure message; json: the structured summary (totals, suites[].cases[] with status/message/type/detail)"
    )]
    pub format: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TestJunitReportParam {
    #[schemars(description = "Test execution ID (returned from test_run)")]
    pub execution_id: String,
    #[schemars(
        description = "Keep the <properties> block of each <testsuite> (the JVM system properties of the test runner, ~100 KB of noise). Default false: the block is removed and the rest of the JUnit XML is returned verbatim."
    )]
    pub include_properties: Option<bool>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TestSuiteCreateParam {
    #[schemars(
        description = "Package that owns the service(s) under test; the suite files are written inside it (PascalCase, e.g. \"UtfDemo\")"
    )]
    pub package: String,
    #[schemars(
        description = "Suite name: becomes resources/test/setup/<suite_name>.xml, the data folder resources/test/data/<suite_name>/ and the name attribute of the suite"
    )]
    pub suite_name: String,
    #[schemars(description = "Suite description (documentation only)")]
    pub description: Option<String>,
    #[schemars(
        description = "JSON array of test cases. Each: {name, service (folder.sub:name), input? (JSON pipeline), expected? (JSON pipeline, subset match), expected_fields? ([{path, operator?, value?, logical?}]), expected_exception? ({class?, message?}), record? (true = invoke now and snapshot the outputs or the failure), mocks? ([{service, pipeline | alternate_service [+parms] | exception, scope?, lifetime?}]), description?, enabled?, mocks_enabled?, comparator_service?, request_method?}. Pipelines are JSON objects; String fields take JSON strings."
    )]
    pub tests: String,
    #[schemars(
        description = "create (default: fails if the suite file exists), overwrite (replace it), or append (add the cases to the existing suite; test names must be new)"
    )]
    pub mode: Option<String>,
    #[schemars(description = "Suite-level mocksEnabled flag (default true)")]
    pub mocks_enabled: Option<bool>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TestSuiteListParam {
    #[schemars(
        description = "Optional JSON array of package names to restrict the listing (e.g., [\"UtfDemo\"]). Omit for every enabled custom package."
    )]
    pub packages: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TestSuiteGetParam {
    #[schemars(description = "Package name (PascalCase)")]
    pub package: String,
    #[schemars(
        description = "Path relative to the package directory, e.g. resources/test/setup/GreetSuite.xml or resources/test/data/GreetSuite/greetAlice_input.xml (as listed by test_suite_list)"
    )]
    pub path: String,
    #[schemars(
        description = "true to decode an IDataXMLCoder pipeline file into JSON; false (default) returns the file as text"
    )]
    pub as_pipeline: Option<bool>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MockLoadParam {
    #[schemars(
        description = "Mock scope, default 'server'. 'server' = all users and sessions (the only scope that reliably survives across MCP calls, since every call opens a new IS session); 'user' = all sessions of the IS user the MCP server authenticates as; 'session' = the IS session of this single call, which is closed when it returns (the mock is never seen again). 'global' is accepted as an alias of 'server'; any other value is rejected."
    )]
    pub scope: Option<String>,
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
    #[schemars(
        description = "Scope the mock was loaded with: 'server' (default), 'user' or 'session' ('global' = alias of 'server'). Clearing with a different scope than the one used by mock_load is a silent no-op on the IS."
    )]
    pub scope: Option<String>,
    #[schemars(description = "Full name of the service to unmock")]
    pub service: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
