mod config;
mod node;
mod pv;
mod storage_class;

use anyhow::Result;
use clap::clap_app;
use config::Config;
use kube::api::Api;
use kube::Client;
use std::fs::File;

fn main() -> Result<()> {
    let matches = clap_app!("kube-sc" =>
    (version: "0.1")
    (author: "Xiaopeng Li <x.friday@outlook.com>")
    (about: "A simple template generator for kubernetes local storage class and persistent volume")
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
    let kube_client = Client::try_default().await?;

    // create StorageClass
    // let kube_client = storage_class::create(kube_client, &cfg).await?;

    // pv::list_pvs(kube_client, &cfg).await
    let o_nodes = node::list(kube_client, &cfg).await?;
    for n in &o_nodes.0 {
        println!("{:?}", n);
    }
    Ok(())
}
