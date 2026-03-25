use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JmsConnectionCreateParam {
    #[schemars(
        description = "JSON string with JMS connection alias settings (aliasName, description, jndiProviderUrl, connectionFactoryLookupName, user, password, clientID, etc.)"
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JmsConnectionNameParam {
    #[schemars(description = "JMS connection alias name")]
    pub alias_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JmsConnectionUpdateParam {
    #[schemars(description = "JMS connection alias name")]
    pub alias_name: String,
    #[schemars(description = "JSON string with settings to update")]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JmsTriggerCreateParam {
    #[schemars(
        description = "JSON string with trigger settings (triggerName, packageName, connectionAlias, destinationName, destinationType, serviceName, etc.)"
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JmsTriggerNameParam {
    #[schemars(description = "JMS trigger full name (e.g., \"mypkg.triggers:myTrigger\")")]
    pub trigger_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JmsTriggerUpdateParam {
    #[schemars(description = "JMS trigger full name")]
    pub trigger_name: String,
    #[schemars(description = "JSON string with settings to update")]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JmsDestinationListParam {
    #[schemars(description = "JMS connection alias name")]
    pub alias_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
