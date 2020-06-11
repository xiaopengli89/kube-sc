use kube::{Client, Api};
use k8s_openapi::api::core::v1::Node;
use kube::api::{ListParams, Meta};
use anyhow::Result;
use crate::config::Config;
use std::collections::HashMap;

pub struct NodePv {
    pub host: String,
    pub root_paths: Vec<String>,
    pub pv_needs: u16,
}

pub async fn list(kube_client: Client, cfg: &Config) -> Result<(Vec<NodePv>, Client)> {
    let nodes: Api<Node> = Api::all(kube_client);
    let mut node_pvs: Vec<NodePv> = vec![];

    fn put_root_path(node_pvs: &mut Vec<NodePv>, host: &str, root_path: &str) {
        let node = if let Some(node) = node_pvs.iter_mut().find(|n| n.host == host) {
           node
        } else {
            node_pvs.push(NodePv {
                host: host.into(),
                root_paths: vec![],
                pv_needs: 0,
            });
            node_pvs.last_mut().unwrap()
        };
        if node.root_paths.iter().find(|p| *p == root_path).is_none() {
            node.root_paths.push(root_path.into());
        }
    }

    for n in &cfg.nodes {
        if let Some(h) = n.host.as_ref() {
            put_root_path(&mut node_pvs, h, &n.root_path);
            continue;
        }
        if let Some(s) = n.selector.as_ref() {
            let lp = ListParams::default().labels(s);
            let o_nodes = nodes.list(&lp).await?;
            for o in o_nodes {
                put_root_path(&mut node_pvs, &o.name(), &n.root_path);
            }
        }
    }

    Ok((node_pvs, nodes.into_client()))
}