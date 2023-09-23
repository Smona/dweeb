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
    /// The character to insert when the key is pressed (also shown on button)
    pub char: String,
    /// The character to insert when the key is pressed while shift is active (also shown on button)
    pub upper: Option<String>,
    /// CSS classes to apply to the key's Button
    pub classes: Option<Vec<String>>,
    /// An alternative label to show on the button (regardless of layer)
    pub label: Option<String>,
    /// The icon to display in place of the label.
    /// Icon names found here: https://specifications.freedesktop.org/icon-naming-spec/icon-naming-spec-latest.html
    pub icon: Option<String>,
}
impl KeyConfig {
    pub fn new(c: &str) -> Self {
        KeyConfig {
            char: c.to_string(),
            upper: None,
            classes: None,
            icon: None,
            label: None,
        }
    }
}

#[derive(Deserialize)]
pub struct PageConfig {
    pub keys: Vec<String>,
}
