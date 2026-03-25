use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MqttConnectionCreateParam {
    #[schemars(
        description = "JSON string with MQTT connection settings: aliasName, description, brokerURL (tcp://host:port), clientID. Optional: cleanSession, keepAliveInterval, connectionTimeout, user, password, sslSettings."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MqttConnectionNameParam {
    #[schemars(description = "MQTT connection alias name")]
    pub alias_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MqttConnectionUpdateParam {
    #[schemars(description = "MQTT connection alias name")]
    pub alias_name: String,
    #[schemars(description = "JSON string with settings to update")]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MqttTriggerCreateParam {
    #[schemars(
        description = "JSON string with trigger settings: triggerName (full ns path), packageName, connectionAlias, topicName, qos (0/1/2), serviceName."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MqttTriggerNameParam {
    #[schemars(description = "MQTT trigger full name (e.g., \"mypkg.triggers:myMqttTrigger\")")]
    pub trigger_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
