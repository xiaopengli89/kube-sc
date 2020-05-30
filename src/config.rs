use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
	pub storage_class_name: String,
	pub nodes: Vec<Node>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
	pub host: String,
	pub pvs: Vec<Pv>,
}

#[derive(Debug, Deserialize)]
pub struct Pv {
	pub name: String,
	pub path: String,
	pub capacity: String,
}