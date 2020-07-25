use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Apt {
    pub packages: Vec<String>,
}
