use crate::kubernetes;
use crate::ui;
use anyhow::Result;
use kube::config::Kubeconfig;
use std::env;
use std::path::PathBuf;

pub fn get_kubeconfig_path() -> Result<PathBuf> {
    match std::env::var("KUBECONFIG") {
        Ok(kubeconfig) => Ok(PathBuf::from(kubeconfig)),
        Err(_) => {
            let home = env::var("HOME").map_err(|_| anyhow::anyhow!("HOME env var is not set"))?;
            Ok(PathBuf::from(home).join(".kube/config"))
        }
    }
}

pub async fn select_context(
    mut config: Kubeconfig,
    kube_context: &Option<String>,
) -> Result<Kubeconfig> {
    if let Some(ctx) = kube_context {
        config.current_context = Some(ctx.clone());
        return Ok(config);
    }

    let current = &config.current_context;
    let input = config
        .contexts
        .iter()
        .map(|c| {
            if Some(&c.name) == current.as_ref() {
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
            config.current_context = Some(new_context);
            Ok(config)
        }
        None => Err(anyhow::anyhow!("Selection cancelled")),
    }
}

pub async fn select_namespace(
    mut config: Kubeconfig,
    kube_namespace: &Option<String>,
) -> Result<Kubeconfig> {
    if let Some(ns) = kube_namespace {
        set_current_namespace(&mut config, ns.clone());
        return Ok(config);
    }

    let current_context = &config.current_context;
    let current_namespace = config
        .contexts
        .iter()
        .find(|ctx| Some(&ctx.name) == current_context.as_ref())
        .and_then(|ctx| ctx.context.as_ref())
        .and_then(|c| c.namespace.as_ref());
    let input = kubernetes::get_namespaces(current_namespace).await?;

    match ui::handle_skim(input)? {
        Some(new_namespace) => {
            let new_namespace = new_namespace.trim_end_matches(" *").to_string();
            if new_namespace.is_empty() {
                println!("No namespace selected");
                return Ok(config);
            }

            set_current_namespace(&mut config, new_namespace);
            Ok(config)
        }
        None => Err(anyhow::anyhow!("Selection cancelled")),
    }
}

fn set_current_namespace(config: &mut Kubeconfig, namespace: String) {
    if let Some(ctx) = config
        .contexts
        .iter_mut()
        .find(|c| Some(&c.name) == config.current_context.as_ref())
        && let Some(context) = &mut ctx.context
    {
        context.namespace = Some(namespace);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use kube::config::{Context, Kubeconfig, NamedContext};

    fn make_test_config(contexts: Vec<(&str, Option<&str>)>, current: Option<&str>) -> Kubeconfig {
        Kubeconfig {
            current_context: current.map(String::from),
            contexts: contexts
                .into_iter()
                .map(|(name, ns)| NamedContext {
                    name: name.to_string(),
                    context: Some(Context {
                        cluster: "test-cluster".to_string(),
                        user: Some("test-user".to_string()),
                        namespace: ns.map(String::from),
                        ..Default::default()
                    }),
                })
                .collect(),
            ..Default::default()
        }
    }

    fn get_generic_test_config() -> Kubeconfig {
        make_test_config(
            vec![("ctx1", Some("default")), ("ctx2", None)],
            Some("ctx1"),
        )
    }

    #[tokio::test]
    async fn test_select_context_with_context() {
        let config = get_generic_test_config();

        let config = select_context(config, &Some("ctx2".to_string()))
            .await
            .expect("select_context should return");

        assert_eq!(config.current_context, Some("ctx2".to_string()));
    }

    #[tokio::test]
    async fn test_select_namespace_with_namespace() {
        let config = get_generic_test_config();

        let config = select_namespace(config, &Some("given-ns".to_string()))
            .await
            .expect("select_namespace should return");

        let current_ns = config
            .contexts
            .iter()
            .find(|c| &c.name == "ctx1")
            .and_then(|ctx| ctx.context.as_ref())
            .and_then(|c| c.namespace.as_ref())
            .expect("ctx1 should have a namespace");

        assert_eq!(current_ns, "given-ns");
    }

    #[test]
    fn test_set_namespace() {
        let mut config = get_generic_test_config();

        set_current_namespace(&mut config, "new-ns".to_string());

        let current_ns = config
            .contexts
            .iter()
            .find(|c| &c.name == "ctx1")
            .and_then(|ctx| ctx.context.as_ref())
            .and_then(|c| c.namespace.as_ref())
            .expect("ctx1 should have a namespace");

        assert_eq!(current_ns, "new-ns");
    }
}
