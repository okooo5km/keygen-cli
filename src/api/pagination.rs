use serde::Deserialize;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub struct Page {
    #[serde(default)]
    pub number: Option<u64>,
    #[serde(default)]
    pub size: Option<u64>,
}
