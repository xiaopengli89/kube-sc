mod config;
mod storage_class;
mod pv;

use std::fs::File;
use anyhow::Result;
use clap::clap_app;
use config::Config;

fn main() -> Result<()> {
	let matches = clap_app!("kube-sc" =>
	(version: "0.1")
	(author: "Xiaopeng Li <x.friday@outlook.com>")
	(about: "A simple template generator for kubernetes local storage class and persistent volume")
	(@arg FILE: -f --file +takes_value +required "Specify a config file")
	).get_matches();

	let file_name = matches.value_of("FILE").unwrap();

	let cfg: Config = {
		let fi = File::open(file_name)?;
		serde_yaml::from_reader(fi)?
	};

	let mut output: String;
	output = storage_class::storage_class_yaml(&cfg)?;
	let pv_yamls = pv::pv_yamls(&cfg)?;
	for pv in pv_yamls.iter() {
		output.push('\n');
		output.push_str(pv.as_ref());
	}

	println!("{}", output);
	Ok(())
}

