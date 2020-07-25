use serde::Deserialize;

pub mod apt;
pub mod networking;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub networking: networking::Config,
    pub apt: apt::Apt,
}
