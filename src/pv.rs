use super::config::Config;
use anyhow::{Result};
use k8s_openapi::api::core::v1::{PersistentVolume, Volume, VolumeMount, HostPathVolumeSource, Pod};
use std::collections::HashSet;
use std::iter;
use kube::Client;
use kube::api::{Api, Meta, ListParams, PostParams, DeleteParams};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use rand::prelude::ThreadRng;
use smol::Timer;
use std::time::Duration;
use crate::node::NodePv;

pub struct PvManager<'a> {
    kube_cli: Option<Client>,
    cfg: &'a Config,
    names: HashSet<String>,
    rng: ThreadRng,
    node_pvs: Option<Vec<NodePv>>,
    last_job: Option<Pod>,
}

impl<'a> PvManager<'a> {
    pub fn new(kube_client: Client, cfg: &'a Config, node_pvs: Vec<NodePv>) -> Self {
        Self {
            kube_cli: Some(kube_client),
            cfg,
            names: HashSet::new(),
            rng: thread_rng(),
            node_pvs: Some(node_pvs),
            last_job: None,
        }
    }

    pub async fn current_pvs(&mut self) -> Result<()> {
        let lp = ListParams::default();
        // take client
        let pvs: Api<PersistentVolume> = Api::all(self.kube_cli.take().unwrap());
        let pv_list = pvs.list(&lp).await?;
        // put back client
        self.kube_cli = Some(pvs.into_client());
        self.names = pv_list.iter().map(|p| p.name()).collect();

        Ok(())
    }

    pub async fn create_pvs(&mut self) -> Result<()> {
        let node_pvs = self.node_pvs.take().unwrap();
        for n in &node_pvs {
            let mut volumes = vec![];
            let mut volume_mounts = vec![];
            let mut commands = vec![];
            let mut pv_paths = vec![];

            for (i, p) in n.root_paths.iter().enumerate() {
                let volume_name = format!("data-{}", i);
                let mount_path = format!("/data/{}", i);

                // add volume
                volumes.push(Volume {
                    name: volume_name.clone(),
                    host_path: Some(HostPathVolumeSource {
                        path: p.0.clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                });

                // add mount
                volume_mounts.push(VolumeMount {
                    name: volume_name,
                    mount_path: mount_path.clone(),
                    ..Default::default()
                });

                let pv_names = self.prepare_n_pv(p.1)?;

                for name in pv_names {
                    commands.push(format!("mkdir -m 0777 -p {}/{}", mount_path, name));
                    pv_paths.push(PvPath {
                        path: p.0.clone() + "/" + &name,
                        name,
                    });
                }
            }

            // mkdir
            self.job_mkdir(&n.host, volumes, volume_mounts, commands.as_ref()).await?;

            // take client
            let pvs: Api<PersistentVolume> = Api::all(self.kube_cli.take().unwrap());
            // create pv
            for p in pv_paths {
                let pv = self.get_pv(&p.name, &p.path, &n.host)?;
                let _ = pvs.create(&PostParams::default(), &pv).await?;
                println!("PersistentVolume [{}] created", p.name);
            }
            // put back client
            self.kube_cli = Some(pvs.into_client());
        }
        self.node_pvs = Some(node_pvs);
        Ok(())
    }

    fn prepare_n_pv(&mut self, count: usize) -> Result<Vec<String>> {
        let mut pv_names = vec![];

        for _ in 0..count {
            loop {
                let name = self.rand_name();
                if self.names.contains(&name) {
                    continue;
                }
                self.names.insert(name.clone());
                pv_names.push(name);
                break;
            }
        }

        Ok(pv_names)
    }

    fn rand_name(&mut self) -> String {
        let chars: String = iter::repeat(())
            .map(|()| self.rng.sample(Alphanumeric))
            .take(8)
            .collect();
        let name = self.cfg.storage_class_name.clone() + "-" + &chars;
        name.to_lowercase()
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

    async fn job_mkdir(&mut self, host: &str, volumes: Vec<Volume>, volume_mounts: Vec<VolumeMount>, commands: &[String]) -> Result<()> {
        // take client
        let jobs: Api<Pod> = Api::namespaced(self.kube_cli.take().unwrap(), &self.cfg.job_namespace);

        // define job
        let mut job: Pod = serde_yaml::from_str(format!(
            r#"apiVersion: v1
kind: Pod
metadata:
  name: {name}
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

        let template_spec = job.spec.as_mut().unwrap();

        // define volumes
        template_spec.volumes = Some(volumes);

        // mount volumes
        template_spec.containers[0].volume_mounts = Some(volume_mounts);

        // run job
        if self.last_job.is_some() {
            jobs.delete(&self.cfg.job_name, &DeleteParams::default()).await?;
        } else {
            if jobs.get(&self.cfg.job_name).await.is_ok() {
                jobs.delete(&self.cfg.job_name, &DeleteParams::default()).await?;
            }
        };
        let mut o_job = jobs.create(&PostParams::default(), &job).await?;

        // wait for job complete
        let mut completed = false;
        for _ in 0..8 {
            o_job = jobs.get(&self.cfg.job_name).await?;
            let status = o_job.status.as_ref().unwrap().phase.as_ref().unwrap();

            if status == "Succeeded" {
                completed = true;
                break;
            }
            if status != "Pending" && status != "Running" {
                return Err(anyhow::anyhow!("job failed"));
            }
            Timer::after(Duration::from_secs(1)).await;
        }
        if !completed {
            return Err(anyhow::anyhow!("job timeout"));
        }

        self.last_job = Some(o_job);

        // put back client
        self.kube_cli = Some(jobs.into_client());

        Ok(())
    }
}

struct PvPath {
    name: String,
    path: String,
}