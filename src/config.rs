use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub storage_class_name: String,
    pub job_namespace: String,
    pub job_name: String,
    pub job_image: String,
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
