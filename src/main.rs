use std::{env, fs};
use std::error::Error;
use settings::Settings;

use crate::midi::InputDevice;
use crate::sampler::Sampler;
use crate::status::GpioStatus;

mod midi;
mod sampler;
mod settings;
mod patch;
mod notename;
mod status;
mod shared_vec;

const DEFAULT_SETTINGS_FILE: &str = "settings.json";

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let predefined = Settings::load(args.next().unwrap_or(DEFAULT_SETTINGS_FILE.to_owned()))?;
    if let Some(specified_device) = args.next() {
        run(&specified_device, predefined)
    } else {
        let devices = list_files("/dev", "midi")?;
        match devices.len() {
            0 => Err(format!("No MIDI devices found").into()),
            1 => run(&devices[0], predefined),
            _ => Err(format!("More than 1 MIDI device found").into())
        }
    }
}

fn run(midi_in: &str, predefined: Vec<Settings>) -> Result<(), Box<dyn Error>> {
    println!("Starting sampler with MIDI-IN: {}", midi_in);
    Sampler::new(InputDevice::open(&midi_in, false)?, predefined, GpioStatus::init(24, 25)?).listen()
}

fn list_files(root: &str, prefix: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let md = fs::metadata(root)?;
    if md.is_dir() {
        let mut files = Vec::new();
        for entry in fs::read_dir(root)? {
            let path = entry?.path();
            if !path.is_dir() && path.file_name().unwrap().to_string_lossy().starts_with(prefix) {
                files.push(path.display().to_string());
            }
        }
        files.sort();
        Ok(files)
    } else {
        Ok(vec![root.to_string()])
    }
}