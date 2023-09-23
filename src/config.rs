use std::collections::HashMap;

use serde::Deserialize;

pub fn get_config() -> Result<Config, String> {
    let contents = std::fs::read_to_string("config.toml")
        .map_err(|e| format!("Could not read config file: {}", e.to_string()))?;
    toml::from_str::<Config>(&contents)
        .map_err(|e| format!("Could not parse config file: {}", e.message()))
}

#[derive(Deserialize)]
pub struct Config {
    pub layout: String,
    pub key_spacing: i32,
    pub padding: i32,
    pub width: i32,

    pub layouts: HashMap<String, HashMap<String, String>>,
    pub keys: HashMap<String, KeyConfig>,
    pub pages: HashMap<String, PageConfig>,
}

#[derive(Deserialize, Clone)]
pub struct KeyConfig {
    pub char: String,
    pub upper: Option<String>,
    pub classes: Option<Vec<String>>,
}
impl KeyConfig {
    pub fn new(c: &str) -> Self {
        KeyConfig {
            char: c.to_string(),
            upper: None,
            classes: None,
        }
    }
}

#[derive(Deserialize)]
pub struct PageConfig {
    pub keys: Vec<String>,
}
