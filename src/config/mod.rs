use serde::Deserialize;

pub mod s3;
pub mod apt;
pub mod files;
pub mod scripts;
pub mod app_image;
pub mod networking;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub networking: networking::Config,
    #[serde(default)]
    pub apt: apt::Config,
    #[serde(default)]
    pub app_image: app_image::Config,
    #[serde(default)]
    pub files: files::Config,
    #[serde(default)]
    pub scripts: scripts::Config,
    #[serde(default)]
    pub s3: s3::Config,
}
