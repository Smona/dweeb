use std::io::prelude::*;
use std::{collections::HashMap, fs::File};

use serde::Deserialize;

pub fn get_config() -> Result<Config, String> {
    let mut file = File::open("config.toml").map_err(|_| "Could not find config file.")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Could not read config file: {}", e.to_string()))?;
    Ok(toml::from_str::<Config>(&contents).unwrap())
    // .map_err(|e| format!("Could not parse config file: {}", e.message()))?
}

#[derive(Deserialize)]
pub struct Config {
    pub keys: HashMap<String, KeyConfig>,
    pub pages: HashMap<String, PageConfig>,
}

#[derive(Deserialize)]
pub struct KeyConfig {
    pub classes: Option<String>,
}

#[derive(Deserialize)]
pub struct PageConfig {
    pub keys: Vec<Vec<String>>,
}
