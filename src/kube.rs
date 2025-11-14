use anyhow::Result;
use k8s_openapi::api::core::v1::Namespace;
use kube::{Api, Client, api::ListParams};

pub async fn get_namespaces(current_namespace: Option<&String>) -> Result<String> {
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
