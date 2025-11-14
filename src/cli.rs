use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum Action {
    Context,
    Namespace,
}

#[derive(Debug, Parser)]
#[command()]
pub struct Args {
    #[arg(value_enum)]
    pub action: Action,
    pub item_name: Option<String>,
}
