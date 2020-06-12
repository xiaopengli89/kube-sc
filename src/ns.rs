use super::config::Config;
use anyhow::Result;
use k8s_openapi::api::core::v1::Namespace;
use kube::Client;
use kube::api::{Api, PatchParams, Meta};

pub async fn apply(kube_client: Client, cfg: &Config) -> Result<Client> {
    let namespaces: Api<Namespace> = Api::all(kube_client);

    let patch = format!(
        r#"kind: Namespace
apiVersion: v1
metadata:
  name: {}
"#, cfg.job_namespace,
    ).into_bytes();
    let ss_apply = PatchParams::default_apply().force();

    let o_patched = namespaces.patch(&cfg.job_namespace, &ss_apply, patch).await?;
    println!("Namespace [{}] applied", o_patched.name());

    Ok(namespaces.into_client())
}
