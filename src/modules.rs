use std::env;
use std::path::PathBuf;
use std::collections::HashMap;

use tokio::fs;
use serde::Deserialize;
use tera::{Tera, Context, Value};

use super::config::Config;
use super::result::{Result, Error};

pub async fn load_module(name: &str, vars: Value) -> Result<(PathBuf, Module)> {
    let mut config_path = PathBuf::from(format!(
        "{}/{}.json5",
        env::var("COMPUTE_MODULES_ROOT")
            .unwrap_or("/usr/share/compute/modules".to_string()),
        name,
    ));

    if !config_path.exists() {
        return Err(Error::Custom(format!("failed to load module {}", name)));
    }

    let context = Context::from_value(vars)?;
    let contents = fs::read_to_string(&config_path).await?;
    let contents = Tera::one_off(
        &contents,
        &context,
        false,
    )?;

    config_path.pop();

    let module: Module = json5::from_str(&contents)?;

    module.validate(context)?;

    Ok((config_path, module))
}

#[derive(Deserialize, Debug)]
pub struct VarDef {
    required: bool,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize, Debug)]
pub struct ModuleConfig {
    pub name: String,
    pub vars: HashMap<String, VarDef>,
}

#[derive(Deserialize, Debug)]
pub struct Module {
    pub module: ModuleConfig,
    pub config: Config,
}

impl Module {
    fn validate(&self, context: Context) -> Result<()> {
        for (name, var) in &self.module.vars {
            if var.required && !context.contains_key(name) {
                return Err(Error::Custom(
                    format!("missing required var {} for module {}", name, self.module.name),
                ));
            }
        }

        Ok(())
    }
}
