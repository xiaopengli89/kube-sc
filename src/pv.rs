use super::config::Config;
use anyhow::Result;
use k8s_openapi::api::core::v1::PersistentVolume;

pub fn pv_yamls(cfg: &Config) -> Result<Vec<String>> {
	let mut out_puts = vec![];
	for node in cfg.nodes.iter() {
		for pv_info in node.pvs.iter() {
			let pv: PersistentVolume = serde_yaml::from_str(format!(
				r#"apiVersion: v1
kind: PersistentVolume
metadata:
  name: {}-{}
spec:
  capacity:
    storage: {}
  volumeMode: Filesystem
  accessModes:
  - ReadWriteOnce
  persistentVolumeReclaimPolicy: Retain
  storageClassName: {}
  local:
    path: {}
  nodeAffinity:
    required:
      nodeSelectorTerms:
      - matchExpressions:
        - key: kubernetes.io/hostname
          operator: In
          values:
          - {}
"#, cfg.storage_class_name, pv_info.name, pv_info.capacity, cfg.storage_class_name, pv_info.path, node.host,
			).as_ref())?;

			let pv_output = serde_yaml::to_string(&pv)?;
			out_puts.push(pv_output);
		}
	}
	Ok(out_puts)
}
