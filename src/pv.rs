use super::config::Config;
use anyhow::Result;
use k8s_openapi::api::core::v1::{PersistentVolume, Volume, VolumeMount};
use k8s_openapi::api::batch::v1::Job;
use std::collections::HashSet;
use std::iter;
use kube::Client;
use kube::api::{Api, Meta, ListParams, PostParams};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use rand::prelude::ThreadRng;
use crate::node::NodePv;

pub struct PvManager<'a> {
	kube_cli: Option<Client>,
	cfg: &'a Config,
	names: HashSet<String>,
	rng: ThreadRng,
	node_pvs: Vec<NodePv>,
	last_job: Option<Job>,
}

impl <'a> PvManager<'a> {
	pub fn new(kube_client: Client, cfg: &'a Config, node_pvs: Vec<NodePv>) -> Self {
		Self {
			kube_cli: Some(kube_client),
			cfg,
			names: HashSet::new(),
			rng: thread_rng(),
            node_pvs,
			last_job: None,
		}
	}

	pub async fn current_pvs(&mut self) -> Result<()> {
		let lp = ListParams::default();
		// take client
		let pvs = Api::all(self.kube_cli.take().unwrap());
		let pv_list = pvs.list(&lp).await?;
		// put back client
        self.kube_cli = Some(pvs.into_client());
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

    // TODO
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

    async fn job_mkdir(&mut self, host: &str, volumes: Vec<Volume>, volume_mounts: Vec<VolumeMount>, commands: &[str]) -> Result<()> {
		// take client
		let jobs: Api<Job> = Api::namespaced(self.kube_cli.take().unwrap(), &self.cfg.job_namespace);

		// define job
		let mut job: Job = serde_yaml::from_str(format!(
			r#"apiVersion: batch/v1
kind: Job
metadata:
  name: {name}
spec:
  template:
    spec:
      nodeSelector:
        kubernetes.io/hostname: {host}
      containers:
      - name: {name}
        image: {image}
        command: ["/bin/sh","-c"]
        args: ["{commands}"]
      restartPolicy: Never
"#,
			name = self.cfg.job_name,
			host = host,
			image = self.cfg.job_image,
			commands = commands.join(" && "),
		).as_ref())?;

		let template_spec = job.spec.as_mut().unwrap().template.spec.as_mut().unwrap();

		// define volumes
		template_spec.volumes = Some(volumes);

		// mount volumes
        template_spec.containers[0].volume_mounts = Some(volume_mounts);

        // run job
		let o_job = if let Some(job_0) = self.last_job.as_ref() {
            job.metadata.as_mut().unwrap().resource_version = Meta::resource_ver(job_0);

			jobs.replace(&self.cfg.job_name, &PostParams::default(), &job).await?
		} else {
			if let Ok(job_0) = jobs.get(&self.cfg.job_name).await {
				job.metadata.as_mut().unwrap().resource_version = Meta::resource_ver(&job_0);

				jobs.replace(&self.cfg.job_name, &PostParams::default(), &job).await?
			} else {
				jobs.create(&PostParams::default(), &job).await?
			}
		};
		self.last_job = Some(o_job);

		// put back client
		self.kube_cli = Some(jobs.into_client());

		Ok(())
	}

}

