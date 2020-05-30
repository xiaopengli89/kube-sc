use super::config::Config;
use anyhow::Result;
use k8s_openapi::api::storage::v1::StorageClass;

pub fn storage_class_yaml(cfg: &Config) -> Result<String> {
	let storage_class: StorageClass = serde_yaml::from_str(format!(
		r#"kind: StorageClass
apiVersion: storage.k8s.io/v1
metadata:
  name: {}
provisioner: kubernetes.io/no-provisioner
volumeBindingMode: WaitForFirstConsumer
"#, cfg.storage_class_name,
	).as_ref())?;

	Ok(serde_yaml::to_string(&storage_class)?)
}