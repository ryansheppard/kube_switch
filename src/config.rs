use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(rename = "current-context")]
    pub current_context: String,
    pub contexts: Vec<ContextEntry>,
    #[serde(flatten)]
    other: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ContextEntry {
    pub name: String,
    pub context: Context,
    #[serde(flatten)]
    other: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Context {
    pub namespace: Option<String>,
    #[serde(flatten)]
    other: HashMap<String, serde_yaml::Value>,
}

pub fn get_kubeconfig_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME env var is not set"))?;
    Ok(PathBuf::from(home).join(".kube/config"))
}
