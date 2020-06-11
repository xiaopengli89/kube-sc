use super::config::Config;
use anyhow::Result;
use k8s_openapi::api::core::v1::PersistentVolume;
use std::collections::HashSet;
use std::iter;
use kube::Client;
use kube::api::{Api, PatchParams, PatchStrategy, Meta, ListParams, PostParams};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use rand::prelude::ThreadRng;

pub struct PvManager<'a> {
	pvs: Api<PersistentVolume>,
	cfg: &'a Config,
	names: HashSet<String>,
	rng: ThreadRng,
}

impl <'a> PvManager<'a> {
	pub fn new(kube_client: Client, cfg: &'a Config) -> Self {
		Self {
			pvs: Api::all(kube_client),
			cfg,
			names: HashSet::new(),
			rng: thread_rng(),
		}
	}

	pub async fn current_pvs(&mut self) -> Result<()> {
		let lp = ListParams::default();
		let pv_list = self.pvs.list(&lp).await?;
        self.names = pv_list.iter().map(|p| p.name()).collect();

		Ok(())
	}

	async fn create_pv(&mut self) -> Result<()> {
		loop {
			let name = self.rand_name();
			if self.names.contains(&name) {
				continue;
			}
			// self.pvs.create(&PostParams::default())
		}

	}

	fn rand_name(&mut self) -> String {
		let chars: String = iter::repeat(())
			.map(|()| self.rng.sample(Alphanumeric))
			.take(6)
			.collect();
		self.cfg.storage_class_name.clone() + "-" + &chars
	}

	fn rand_path(&self) -> (String, String) {
		("".into(), "".into())
	}
	/*
	fn get_pv(&self, name: &str) -> PersistentVolume {
		let pv: PersistentVolume = serde_yaml::from_str(format!(
			r#"apiVersion: v1
kind: PersistentVolume
metadata:
  name: {name}
spec:
  capacity:
    storage: {capacity}
  volumeMode: Filesystem
  accessModes:
  - ReadWriteOnce
  persistentVolumeReclaimPolicy: Retain
  storageClassName: {storage_class_name}
  local:
    path: {path}
  nodeAffinity:
    required:
      nodeSelectorTerms:
      - matchExpressions:
        - key: kubernetes.io/hostname
          operator: In
          values:
          - {host}
"#,
            name = name,
			capacity = self.cfg.capacity,
			storage_class_name = self.cfg.storage_class_name,
		).as_ref())?;
        pv
	}

	 */
}

