use anyhow::Result;
use clap::Parser;
use std::fs::File;

mod cli;
mod config;
mod handlers;
mod kube;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    let kubeconfig_path = config::get_kubeconfig_path()?;
    let contents = File::open(&kubeconfig_path)?;
    let config: config::Config = serde_yaml::from_reader(contents)?;

    let config = match args.action {
        cli::Action::Context => handlers::handle_context(config, &args.item_name).await?,
        cli::Action::Namespace => handlers::handle_namespace(config, &args.item_name).await?,
    };

    let new_file = File::create(&kubeconfig_path)?;
    serde_yaml::to_writer(new_file, &config)?;

    Ok(())
}
