use std::fs::File;
use std::io::BufReader;
use rodio::OutputStream;

mod midi;
mod sampler;
mod patch;

// fn main() {
//     // println!("Getting stream...");
//     // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
//     // println!("Loading file...");
//     // let file = BufReader::new(File::open("gong.wav").unwrap());
//     // println!("Playing sound...");
//     // let sink = stream_handle.play_once(file).unwrap();
//     // println!("Waiting for finish...");
//     // sink.sleep_until_end();
//     // println!("Finished.");   
// }

const DEFAULT_SETTINGS_FILE: &str = "settings.json";

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let predefined = Settings::load(args.next().unwrap_or(DEFAULT_SETTINGS_FILE.to_owned()))?;
    if let Some(specified_device) = args.next() {
        run(specified_device, predefined);
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
    MultiArpeggiator {
        midi_in: InputDevice::open_with_external_clock(&midi_in, &midi_out)?,
        midi_out: OutputDevice::open(&midi_out)?,
        settings: PredefinedProgramChanges::new(predefined),
        status: LedStatus::<8>::new(18)
    }.listen()
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