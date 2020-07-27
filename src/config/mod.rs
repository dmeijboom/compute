use serde::Deserialize;

pub mod apt;
pub mod app_image;
pub mod networking;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub networking: networking::Config,
    pub apt: apt::Config,
    pub app_image: app_image::Config,
}
