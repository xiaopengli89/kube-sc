use super::config::Config;
use anyhow::Result;
use k8s_openapi::api::storage::v1::StorageClass;
use kube::Client;
use kube::api::{Api, PatchParams, Meta};

pub async fn apply(kube_client: Client, cfg: &Config) -> Result<Client> {
    let storage_classes: Api<StorageClass> = Api::all(kube_client);

    let patch = format!(
        r#"kind: StorageClass
apiVersion: storage.k8s.io/v1
metadata:
  name: {}
provisioner: kubernetes.io/no-provisioner
volumeBindingMode: WaitForFirstConsumer
"#, cfg.storage_class_name,
    ).into_bytes();
    let ss_apply = PatchParams::default_apply().force();

    let o_patched = storage_classes.patch(&cfg.storage_class_name, &ss_apply, patch).await?;
    println!("StorageClass [{}] applied", o_patched.name());

    Ok(storage_classes.into_client())
}
