use crate::kube;
use crate::ui;
use anyhow::Ok;
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

impl Config {
    pub async fn select_context(mut self, kube_context: &Option<String>) -> Result<Self> {
        if let Some(ctx) = kube_context {
            self.current_context = ctx.clone();
            return Ok(self);
        }

        let current = &self.current_context;
        let input = self
            .contexts
            .iter()
            .map(|c| {
                if &c.name == current {
                    format!("{} *", c.name)
                } else {
                    c.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        match ui::handle_skim(input)? {
            Some(selection) => {
                let new_context = selection.trim_end_matches(" *").to_string();
                self.current_context = new_context;
                Ok(self)
            }
            None => Err(anyhow::anyhow!("Selection cancelled")),
        }
    }

    pub async fn select_namespace(mut self, kube_namespace: &Option<String>) -> Result<Self> {
        if let Some(ns) = kube_namespace {
            self.set_current_namespace(ns.clone());
            return Ok(self);
        }

        let current_context = &self.current_context;
        let current_namespace = self
            .contexts
            .iter()
            .find(|ctx| &ctx.name == current_context)
            .and_then(|ctx| ctx.context.namespace.as_ref());
        let input = kube::get_namespaces(current_namespace).await?;

        match ui::handle_skim(input)? {
            Some(new_namespace) => {
                let new_namespace = new_namespace.trim_end_matches(" *").to_string();
                if new_namespace.is_empty() {
                    println!("No namespace selected");
                    return Ok(self);
                }

                self.set_current_namespace(new_namespace);
                Ok(self)
            }
            None => Err(anyhow::anyhow!("Selection cancelled")),
        }
    }

    fn set_current_namespace(&mut self, namespace: String) {
        if let Some(ctx) = self
            .contexts
            .iter_mut()
            .find(|c| c.name == self.current_context)
        {
            ctx.context.namespace = Some(namespace);
        }
    }
}
