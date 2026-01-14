use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct RssConfig {
    pub channel: String,
    pub tag: String,
    pub emoji: String,
    pub rss: String,
    pub category_filter: Option<Vec<String>>,
    pub link_filter: Option<Vec<String>>,
    #[serde(default)]
    pub upload_image: bool,
    pub setup: Option<SetupConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SetupConfig {
    pub title: Option<String>,
    pub link: Option<String>,
    pub content: Option<String>,
    pub author: Option<String>,
    #[serde(rename = "authorLink")]
    pub author_link: Option<String>,
    #[serde(rename = "pubDate")]
    pub pub_date: Option<String>,
}

pub type FullConfig = HashMap<String, RssConfig>;
