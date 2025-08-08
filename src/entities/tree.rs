use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepositoryTree {
    pub sha: String,
    pub url: String,
    pub tree: Vec<TreeEntry>,
    pub truncated: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreeEntry {
    pub path: String,
    pub mode: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub sha: String,
    pub url: String,
    pub size: Option<i64>,
}
