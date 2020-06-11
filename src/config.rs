use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
	pub storage_class_name: String,
	pub nodes: Vec<Node>,
    pub min_count: u16,
	pub capacity: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
	pub host: Option<String>,
    pub selector: Option<String>,
	pub root_path: String,
}
