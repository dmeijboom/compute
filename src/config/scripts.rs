use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Script {
    pub name: String,
    pub test: String,
    pub cmd: String,
    #[serde(default)]
    pub privileged: bool,
}

pub type Config = Vec<Script>;
