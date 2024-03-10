use std::fs::File;
use std::io::BufReader;
use rodio::OutputStream;

fn main() {
    println!("Getting stream...");
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    println!("Loading file...");
    let file = BufReader::new(File::open("gong.wav").unwrap());
    println!("Playing sound...");
    let sink = stream_handle.play_once(file).unwrap();
    println!("Waiting for finish...");
    sink.sleep_until_end();
    println!("Finished.");
}
