use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UserAddParam {
    #[schemars(description = "Username (ASCII letters and digits only)")]
    pub username: String,
    #[schemars(description = "Password for the new user")]
    pub password: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UserNameParam {
    #[schemars(description = "Username")]
    pub username: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UserDisableParam {
    #[schemars(description = "Username")]
    pub username: String,
    #[schemars(description = "Set to true to disable, false to enable")]
    pub disabled: bool,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GroupNameParam {
    #[schemars(description = "Group name")]
    pub groupname: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GroupChangeParam {
    #[schemars(description = "Group name")]
    pub groupname: String,
    #[schemars(
        description = "JSON array of usernames to set as group members (replaces current membership)"
    )]
    pub membership: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AclAddParam {
    #[schemars(
        description = "JSON string with ACL settings: aclName, allowList (array of group names), denyList (array of group names)"
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AclNameParam {
    #[schemars(description = "ACL name")]
    pub acl_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
