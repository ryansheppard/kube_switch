use anyhow::Result;
use clap::Parser;
use std::fs::File;

mod cli;
mod config;
mod kube;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    let kubeconfig_path = config::get_kubeconfig_path()?;
    let contents = File::open(&kubeconfig_path)?;
    let config: config::Config = serde_yaml::from_reader(contents)?;

    let config = match args.action {
        cli::Action::Context => config.select_context(&args.item_name).await?,
        cli::Action::Namespace => config.select_namespace(&args.item_name).await?,
    };

    let new_file = File::create(&kubeconfig_path)?;
    serde_yaml::to_writer(new_file, &config)?;

    Ok(())
}
