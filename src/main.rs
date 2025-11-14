use anyhow::Result;
use clap::{Parser, ValueEnum};
use k8s_openapi::api::core::v1::Namespace;
use kube::{Api, Client, api::ListParams};
use serde::{Deserialize, Serialize};
use skim::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
enum Action {
    Context,
    Namespace,
}

#[derive(Debug, Parser)]
#[command()]
struct Args {
    #[arg(value_enum)]
    action: Action,
    item_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    #[serde(rename = "current-context")]
    current_context: String,
    contexts: Vec<ContextEntry>,
    #[serde(flatten)]
    other: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ContextEntry {
    name: String,
    context: Context,
    #[serde(flatten)]
    other: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Context {
    namespace: Option<String>,
    #[serde(flatten)]
    other: HashMap<String, serde_yaml::Value>,
}

fn get_kubeconfig_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME env var is not set"))?;
    Ok(PathBuf::from(home).join(".kube/config"))
}

fn handle_skim(input: String) -> Result<Option<String>> {
    let options = SkimOptionsBuilder::default()
        .height(String::from("50%"))
        .no_multi(true)
        .build()
        .unwrap();

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));

    let output = Skim::run_with(&options, Some(items));

    match output {
        Some(out) if out.is_abort => Ok(None),
        Some(out) => {
            let selection = out
                .selected_items
                .first()
                .map(|item| item.output().to_string());
            Ok(selection)
        }
        None => Ok(None),
    }
}

async fn handle_context(mut config: Config, kube_context: &Option<String>) -> Result<Config> {
    if let Some(ctx) = kube_context {
        config.current_context = ctx.clone();
        return Ok(config);
    }

    let current = &config.current_context;
    let input = config
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

    match handle_skim(input)? {
        Some(selection) => {
            let new_context = selection.trim_end_matches(" *").to_string();
            config.current_context = new_context;
            Ok(config)
        }
        None => Err(anyhow::anyhow!("Selection cancelled")),
    }
}

async fn handle_namespace(mut config: Config, kube_namespace: &Option<String>) -> Result<Config> {
    if let Some(ctx) = kube_namespace {
        let current_context = &config.current_context;
        for context_entry in &mut config.contexts {
            if &context_entry.name == current_context {
                context_entry.context.namespace = Some(ctx.clone());
            }
        }
        return Ok(config);
    }

    let current_context = &config.current_context;
    let current_namespace = config
        .contexts
        .iter()
        .find(|ctx| &ctx.name == current_context)
        .and_then(|ctx| ctx.context.namespace.as_ref());
    let input = get_namespaces(current_namespace).await?;

    match handle_skim(input)? {
        Some(new_namespace) => {
            let new_namespace = new_namespace.trim_end_matches(" *").to_string();
            if new_namespace.is_empty() {
                println!("No namespace selected");
                return Ok(config);
            }

            let current_context = &config.current_context;
            for context_entry in &mut config.contexts {
                if &context_entry.name == current_context {
                    context_entry.context.namespace = Some(new_namespace.clone());
                }
            }

            Ok(config)
        }
        None => Err(anyhow::anyhow!("Selection cancelled")),
    }
}

async fn get_namespaces(current_namespace: Option<&String>) -> Result<String> {
    let client = Client::try_default().await?;
    let namespaces_api: Api<Namespace> = Api::all(client);
    let list_params = ListParams::default();
    let namespaces = namespaces_api.list(&list_params).await?;

    let input = namespaces
        .items
        .iter()
        .filter_map(|ns| ns.metadata.name.as_ref())
        .map(|name| {
            if Some(name) == current_namespace {
                format!("{} *", name)
            } else {
                name.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(input)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let kubeconfig_path = get_kubeconfig_path()?;
    let contents = File::open(&kubeconfig_path)?;
    let config: Config = serde_yaml::from_reader(contents)?;

    let config = match args.action {
        Action::Context => handle_context(config, &args.item_name).await?,
        Action::Namespace => handle_namespace(config, &args.item_name).await?,
    };

    let new_file = File::create(&kubeconfig_path)?;
    serde_yaml::to_writer(new_file, &config)?;

    Ok(())
}
