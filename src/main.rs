#![allow(dead_code)]

mod config;
mod node;
mod pv;
mod storage_class;
mod ns;

use anyhow::Result;
use clap::clap_app;
use config::Config;
use kube::Client;
use std::fs::File;

fn main() -> Result<()> {
    let matches = clap_app!("kube-sc" =>
    (version: "0.1")
    (author: "Xiaopeng Li <x.friday@outlook.com>")
    (about: "A persistent volume generator for kubernetes local storage class")
    (@arg FILE: -f --file +takes_value +required "Specify a config file")
    )
    .get_matches();

    let file_name = matches.value_of("FILE").unwrap();

    let cfg: Config = {
        let fi = File::open(file_name)?;
        serde_yaml::from_reader(fi)?
    };

    smol::run(run(cfg))
}

async fn run(cfg: Config) -> Result<()> {
    let mut kube_client = Client::try_default().await?;

    // apply StorageClass
    kube_client = storage_class::apply(kube_client, &cfg).await?;

    // apply Namespace
    kube_client = ns::apply(kube_client, &cfg).await?;

    // get meta of node pvs
    let (node_pvs, kube_client) = node::list(kube_client, &cfg).await?;

    // get pv manager
    let mut pm = pv::PvManager::new(kube_client, &cfg, node_pvs);

    // get current pvs
    pm.current_pvs().await?;

    // create pvs
    pm.create_pvs().await?;

    Ok(())
}
