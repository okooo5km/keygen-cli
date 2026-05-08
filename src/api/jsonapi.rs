use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Generic JSON:API single-resource document.
#[derive(Debug, Serialize, Deserialize)]
pub struct Document<T> {
    pub data: T,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub included: Option<Vec<Resource>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default)]
    pub attributes: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relationships: Option<Value>,
}
