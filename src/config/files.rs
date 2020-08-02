use tera::{Map, Value};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3File {
    pub bucket_name: String,
    pub path: String,
}

pub enum TemplateSource<'a> {
    Local(&'a str),
    S3(&'a S3File),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateFile {
    pub path: String,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub s3: Option<S3File>,
    pub owner: Option<(u32, u32)>,
    pub mode: Option<u32>,
    #[serde(default)]
    pub context: Map<String, Value>,
}

impl TemplateFile {
    pub fn source(&self) -> TemplateSource {
        if let Some(template) = &self.template {
            return TemplateSource::Local(template);
        }

        if let Some(s3) = &self.s3 {
            return TemplateSource::S3(s3);
        }

        panic!("no source for path: {}", self.path);
    }
}

pub type Config = Vec<TemplateFile>;
