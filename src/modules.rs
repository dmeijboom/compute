use std::env;
use std::path::PathBuf;
use std::io::{Result, Error, ErrorKind};

use tokio::fs;
use tera::{Map, Value};
use serde::Deserialize;

use super::config::Config;

pub async fn load_module(name: &str) -> Result<(PathBuf, Module)> {
    let mut config_path = PathBuf::from(format!(
        "{}/{}.json5",
        env::var("COMPUTE_MODULES_ROOT")
            .unwrap_or("/usr/share/compute/modules".to_string()),
        name,
    ));

    if !config_path.exists() {
        return Err(Error::new(ErrorKind::Other, format!("failed to load module: {}", name)));
    }

    let contents = fs::read_to_string(&config_path).await?;
    let module = json5::from_str(&contents)
        .map_err(|e| Error::new(
            ErrorKind::Other,
            format!("failed to deserialize module config: {}", e),
        ))?;

    config_path.pop();

    Ok((config_path, module))
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
