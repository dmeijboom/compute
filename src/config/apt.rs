use serde::Deserialize;

fn main_component() -> Vec<String> {
    vec!["main".to_string()]
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AptRepository {
    pub name: String,
    pub url: String,
    pub distro: String,
    pub key_url: String,
    #[serde(default = "main_component")]
    pub components: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Apt {
    pub packages: Vec<String>,
    #[serde(default)]
    pub repositories: Vec<AptRepository>,
}
