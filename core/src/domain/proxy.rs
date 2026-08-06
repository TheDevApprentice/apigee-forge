use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proxy {
    pub name: String,
    #[serde(default)]
    pub revisions: Vec<ProxyRevision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyRevision {
    pub number: u32,
    #[serde(default)]
    pub deployed: bool,
}
