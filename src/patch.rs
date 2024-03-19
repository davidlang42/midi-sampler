use rodio::{OutputStream, OutputStreamHandle, Sink};
use wmidi::Note;
use std::{fs::File, io::{BufReader, Cursor, Read}};

use crate::settings::Settings;

pub struct Patch {
    output: OutputStreamHandle,
    data: [Option<Vec<u8>>; Self::NOTE_MAX],
    playing: [Option<Vec<Sink>>; Self::NOTE_MAX]
}

impl Patch {
    const NOTE_MAX: usize = 127;

    pub fn from(settings: &Settings) -> Self {
        const NONE_VEC_U8: Option<Vec<u8>> = None;
        const NONE_VEC_SINK: Option<Vec<Sink>> = None;
        let mut data = [NONE_VEC_U8; Self::NOTE_MAX];
        for (note, path) in &settings.samples {
            let n = Note::from_u8_lossy(*note);
            let mut f = File::open(path).unwrap();
            let mut bytes = Vec::new();
            f.read_to_end(&mut bytes).unwrap();
            data[n as usize] = Some(bytes);
        }
        let (_, output) = OutputStream::try_default().unwrap();
        Self {
            output,
            data,
            playing: [NONE_VEC_SINK; Self::NOTE_MAX]
        }
    }

    pub fn play(&mut self, note: Note) {
        if let Some(bytes) = &self.data[note as usize] {
            let (_, output) = OutputStream::try_default().unwrap();
            let bytes_copy = bytes.clone();//TODO this is fucking stupid
            let cursor = Cursor::new(bytes_copy);
            let sink = output.play_once(cursor).unwrap();
            sink.sleep_until_end();
        } 
    }

    pub fn finish_all_sounds(self) -> OutputStream {
        todo!()//TODO
    }
}