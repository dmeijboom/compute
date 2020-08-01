use tera::{Map, Value};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateFile {
    pub path: String,
    pub template: String,
    pub owner: Option<(u32, u32)>,
    pub mode: Option<u32>,
    pub context: Map<String, Value>,
}

pub type Config = Vec<TemplateFile>;
