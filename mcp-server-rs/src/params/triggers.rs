use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TriggerNameParam {
    #[schemars(description = "Trigger full name (e.g., \"mypkg.triggers:myTrigger\")")]
    pub trigger_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TriggerCreateParam {
    #[schemars(
        description = "JSON string with trigger settings. Required: triggerName (full ns path), package. Key fields: conditions (array of {conditionName, serviceName, joinType, messageType (array), filter (array)}), joinTimeOut, queueCapacity, maxRetryAttempts, retryInterval, isConcurrent, maxExecutionThreads, dupDetection."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TriggerSetPropertiesParam {
    #[schemars(description = "Trigger full name")]
    pub trigger_name: String,
    #[schemars(
        description = "JSON string with properties to update: executeEnabled, joinTimeOut, queueCapacity, maxRetryAttempts, retryInterval, isConcurrent, maxExecutionThreads, etc."
    )]
    pub properties: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MessagingConnectionCreateParam {
    #[schemars(
        description = "JSON string with connection settings. Required: aliasName, type (UM/LOCAL). For UM: um_rname (nsp://host:port). Optional: description, enabled, clientID, csqSize, csqDrainInOrder."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MessagingConnectionNameParam {
    #[schemars(description = "Messaging connection alias name")]
    pub alias_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
