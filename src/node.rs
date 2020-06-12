use kube::{Client, Api};
use k8s_openapi::api::core::v1::Node;
use kube::api::{ListParams, Meta};
use anyhow::Result;
use crate::config::Config;

#[derive(Debug)]
pub struct NodePv {
    pub host: String,
    pub root_paths: Vec<(String, usize)>,
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
            });
            node_pvs.last_mut().unwrap()
        };
        if node.root_paths.iter().find(|p| p.0 == root_path).is_none() {
            node.root_paths.push((root_path.into(), 0));
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

    dispatch_pv_counts(&mut node_pvs, cfg.count);

    Ok((node_pvs, nodes.into_client()))
}

pub fn dispatch_pv_counts(node_pvs: &mut Vec<NodePv>, count: usize) {
    let root_path_count = node_pvs.iter().map(|n| &n.root_paths).flatten().count();
    let base = count / root_path_count;
    let mut remain = count % root_path_count;
    for p in node_pvs.iter_mut().map(|n| &mut n.root_paths).flatten() {
        let p: &mut (String, usize) = p;
        p.1 = base + if remain > 0 {
            remain -= 1;
            1
        } else { 0 };
    }
}