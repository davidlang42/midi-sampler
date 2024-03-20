use std::collections::HashMap;
use std::error::Error;
use std::fs;
use serde_derive::{Deserialize, Serialize};
use crate::notename::NoteName;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Settings {
    pub name: String,
    pub lsb: u8, // 0-127
    pub msb: u8, // 0-127
    pub pc: u8, // 1-128
    pub samples: HashMap<NoteName, String>
}

impl Settings {
    pub fn load(file: String) -> Result<Vec<Self>, Box<dyn Error>> {
        let json = fs::read_to_string(&file).map_err(|e| format!("Cannot read from '{}': {}", file, e))?;
        let settings: Vec<Settings> = serde_json::from_str(&format!("[{}]", json)).map_err(|e| format!("Cannot parse settigs from '{}': {}", file, e))?;
        Ok(settings)
    }
}