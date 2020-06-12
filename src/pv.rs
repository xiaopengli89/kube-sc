use super::config::Config;
use anyhow::Result;
use k8s_openapi::api::core::v1::PersistentVolume;
use std::collections::HashSet;
use std::iter;
use kube::Client;
use kube::api::{Api, Meta, ListParams, PostParams};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use rand::prelude::ThreadRng;
use crate::node::NodePv;

pub struct PvManager<'a> {
	pvs: Api<PersistentVolume>,
	cfg: &'a Config,
	names: HashSet<String>,
	rng: ThreadRng,
	node_pvs: Vec<NodePv>,
}

impl <'a> PvManager<'a> {
	pub fn new(kube_client: Client, cfg: &'a Config, node_pvs: Vec<NodePv>) -> Self {
		Self {
			pvs: Api::all(kube_client),
			cfg,
			names: HashSet::new(),
			rng: thread_rng(),
            node_pvs,
		}
	}

	pub async fn current_pvs(&mut self) -> Result<()> {
		let lp = ListParams::default();
		let pv_list = self.pvs.list(&lp).await?;
        self.names = pv_list.iter().map(|p| p.name()).collect();

		Ok(())
	}

	pub async fn create_pvs(&mut self) -> Result<()> {
		for n in self.node_pvs {
			for p in n.root_paths {
				self.create_n_pv(&n.host, &p.0, p.1)?;
			}
		}
		Ok(())
	}

	async fn create_n_pv(&mut self, host: &str, path: &str, count: usize) -> Result<()> {
        let mut pv_names = vec![];

		for _ in 0..count {
			loop {
				let name = self.rand_name();
				if self.names.contains(&name) {
					continue;
				}
				self.names.insert(name.clone());
				pv_names.push(name);

				// let pv = self.get_pv(&name, path, host)?;
				// self.pvs.create(&PostParams::default(), &pv).await?;
				break;
			}
		}

		// create job to node for mkdir
		// TODO

		Ok(())
	}

	fn rand_name(&mut self) -> String {
		let chars: String = iter::repeat(())
			.map(|()| self.rng.sample(Alphanumeric))
			.take(10)
			.collect();
		self.cfg.storage_class_name.clone() + "-" + &chars
	}

	fn rand_path(&self) -> (String, String) {
		("".into(), "".into())
	}

	fn get_pv(&self, name: &str, path: &str, host: &str) -> Result<PersistentVolume> {
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
			path = path,
			host = host,
		).as_ref())?;
        Ok(pv)
	}

    async fn mkdir_job(&self, node_pv: &NodePv) {

	}
}

