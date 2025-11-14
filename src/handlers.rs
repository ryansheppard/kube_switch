use crate::config::Config;
use crate::kube;
use crate::ui;
use anyhow::Result;

pub async fn handle_context(mut config: Config, kube_context: &Option<String>) -> Result<Config> {
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

    match ui::handle_skim(input)? {
        Some(selection) => {
            let new_context = selection.trim_end_matches(" *").to_string();
            config.current_context = new_context;
            Ok(config)
        }
        None => Err(anyhow::anyhow!("Selection cancelled")),
    }
}

pub async fn handle_namespace(
    mut config: Config,
    kube_namespace: &Option<String>,
) -> Result<Config> {
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
    let input = kube::get_namespaces(current_namespace).await?;

    match ui::handle_skim(input)? {
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
