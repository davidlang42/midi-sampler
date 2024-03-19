use wmidi::{U7, Note};
use std::collections::HashMap;

pub struct Data(HashMap<Note, ?>);

pub struct Settings {
    lsb: U7,
    msb: U7,
    pc: U7,
    files: HashMap<Note, String>
}

impl Settings {
    pub fn load(file: String) -> Result<Vec<Self>, Box<dyn Error>> {
        let json = fs::read_to_string(&file).map_err(|e| format!("Cannot read from '{}': {}", file, e))?;
        let settings: Vec<Settings> = serde_json::from_str(&format!("[{}]", json)).map_err(|e| format!("Cannot parse settigs from '{}': {}", file, e))?;
        Ok(settings)
    }
}