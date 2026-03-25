use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StreamingConnectionCreateParam {
    #[schemars(description = "Connection alias base name (letters, digits, underscores)")]
    pub base_name: String,
    #[schemars(description = "Full connection alias name (usually package_baseName)")]
    pub name: String,
    #[schemars(description = "Description of this connection")]
    pub description: String,
    #[schemars(description = "Provider type (e.g., \"Kafka\")")]
    pub provider_type: String,
    #[schemars(description = "Package name")]
    pub package: String,
    #[schemars(description = "Provider URI / bootstrap servers (e.g., \"localhost:9092\")")]
    pub host: String,
    #[schemars(description = "Client prefix for Kafka client ID")]
    pub client_id: String,
    #[schemars(description = "Security protocol: none, SSL, SASL_SSL, SASL_PLAINTEXT")]
    pub security_protocol: Option<String>,
    #[schemars(
        description = "Extra configuration parameters as newline-separated name=value pairs"
    )]
    pub other_properties: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StreamingConnectionNameParam {
    #[schemars(description = "Streaming connection alias name")]
    pub name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StreamingEventSourceCreateParam {
    #[schemars(description = "Connection alias name this event spec belongs to")]
    pub create_alias_name: String,
    #[schemars(description = "Event specification name (unique within the connection alias)")]
    pub create_reference_id: String,
    #[schemars(description = "Kafka topic name")]
    pub topic_name: String,
    #[schemars(
        description = "Key type: none, RAW, STRING, JSON, XML, DOUBLE, FLOAT, INTEGER, LONG"
    )]
    pub key_type: Option<String>,
    #[schemars(
        description = "Value type: none, RAW, STRING, JSON, XML, DOUBLE, FLOAT, INTEGER, LONG"
    )]
    pub value_type: Option<String>,
    #[schemars(description = "Document type name for key (when key type is JSON or XML)")]
    pub key_type_document_type: Option<String>,
    #[schemars(description = "Document type name for value (when value type is JSON or XML)")]
    pub value_type_document_type: Option<String>,
    #[schemars(description = "Charset for key (default: UTF-8)")]
    pub key_type_charset: Option<String>,
    #[schemars(description = "Charset for value (default: UTF-8)")]
    pub value_type_charset: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StreamingEventSourceDeleteParam {
    #[schemars(description = "Connection alias name")]
    pub alias_name: String,
    #[schemars(description = "Event specification reference ID")]
    pub reference_id: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StreamingEventSourceListParam {
    #[schemars(description = "Filter by connection alias name (optional)")]
    pub alias_name: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StreamingTriggerNameParam {
    #[schemars(description = "Trigger name")]
    pub name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
