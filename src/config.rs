use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
	pub storage_class_name: String,
	pub namespace: String,
	pub nodes: Vec<Node>,
    pub count: usize,
	pub capacity: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
	pub host: Option<String>,
    pub selector: Option<String>,
	pub root_path: String,
}
