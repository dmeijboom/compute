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

    let mut context = Context::new();
    context.insert("vars", &vars);

    let contents = fs::read_to_string(&config_path).await?;
    let contents = Tera::one_off(
        &contents,
        &context,
        false,
    ).map_err(|e| Error::from_template_err(format!("{}.json5", name), e))?;

    config_path.pop();

    let module: Module = json5::from_str(&contents)?;

    module.validate(vars)?;

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
    fn validate(&self, vars: Value) -> Result<()> {
        if let Value::Object(vars) = vars {
            for (name, var) in &self.module.vars {
                if var.required && !vars.contains_key(name) {
                    return Err(Error::Custom(
                        format!("missing required var {} for module {}", name, self.module.name),
                    ));
                }
            }

            for name in vars.keys() {
                if !self.module.vars.contains_key(name) {
                    return Err(Error::Custom(
                        format!("unknown var {} for module {}", name, self.module.name),
                    ));
                }
            }

            return Ok(())
        }

        Err(Error::Custom(
            format!("invalid type for vars of module {}", self.module.name),
        ))
    }
}
