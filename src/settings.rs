use std::collections::HashMap;
use std::error::Error;
use std::fs;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Settings {
    name: String,
    pub lsb: u8,
    pub msb: u8,
    pub pc: u8,
    samples: HashMap<String, String>,//TODO HashMap<Note, Path>
}

impl Settings {
    pub fn load(file: String) -> Result<Vec<Self>, Box<dyn Error>> {
        let json = fs::read_to_string(&file).map_err(|e| format!("Cannot read from '{}': {}", file, e))?;
        let settings: Vec<Settings> = serde_json::from_str(&format!("[{}]", json)).map_err(|e| format!("Cannot parse settigs from '{}': {}", file, e))?;
        Ok(settings)
    }
}