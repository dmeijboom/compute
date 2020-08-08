use std::env;
use std::path::PathBuf;

use tokio::fs;
use tera::{Map, Value};
use serde::Deserialize;

use super::config::Config;
use super::result::{Result, Error};

pub async fn load_module(name: &str) -> Result<(PathBuf, Module)> {
    let mut config_path = PathBuf::from(format!(
        "{}/{}.json5",
        env::var("COMPUTE_MODULES_ROOT")
            .unwrap_or("/usr/share/compute/modules".to_string()),
        name,
    ));

    if !config_path.exists() {
        return Err(Error::Custom(format!("failed to load module {}", name)));
    }

    let contents = fs::read_to_string(&config_path).await?;

    config_path.pop();

    Ok((config_path, json5::from_str(&contents)?))
}

#[derive(Deserialize, Debug)]
pub struct ModuleConfig {
    pub name: String,
    pub vars: Map<String, Value>,
}

#[derive(Deserialize, Debug)]
pub struct Module {
    pub module: ModuleConfig,
    pub config: Config,
}
