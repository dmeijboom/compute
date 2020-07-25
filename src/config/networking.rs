use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub hostname: String,
}
