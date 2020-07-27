use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct App {
    pub name: String,
    pub url: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub apps: Vec<App>,
}
