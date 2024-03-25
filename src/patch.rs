use rodio::{OutputStream, OutputStreamHandle, Sink};
use wmidi::Note;
use std::{fs::File, io::{Cursor, Read}};

use crate::settings::Settings;

pub struct Patch {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    data: [Option<Vec<u8>>; Self::NOTE_MAX],
    playing: [Vec<Sink>; Self::NOTE_MAX]
}

impl Patch {
    const NOTE_MAX: usize = 127;

    pub fn from(settings: &Settings) -> Self {
        const NONE_VEC_U8: Option<Vec<u8>> = None;
        const EMPTY_VEC_SINK: Vec<Sink> = Vec::new();
        let mut data = [NONE_VEC_U8; Self::NOTE_MAX];
        for (note, path) in &settings.samples {
            let n = Note::from_u8_lossy(*note as u8);
            let mut f = File::open(path).unwrap();
            let mut bytes = Vec::new();
            f.read_to_end(&mut bytes).unwrap();
            data[n as usize] = Some(bytes);
        }
        let (_stream, handle) = OutputStream::try_default().unwrap();
        Self {
            _stream,
            handle,
            data,
            playing: [EMPTY_VEC_SINK; Self::NOTE_MAX]
        }
    }

    pub fn play(&mut self, note: Note) -> bool {
        if let Some(bytes) = &self.data[note as usize] {
            let bytes_copy = bytes.clone(); // because rodio was too lazy to implement lifetimes
            let cursor = Cursor::new(bytes_copy);
            let sink = self.handle.play_once(cursor).unwrap();
            self.playing[note as usize].push(sink);
            true
        } else {
            false
        }
    }

    pub fn finish_all_sounds(self) {
        for vec in self.playing {
            for sink in vec {
                sink.sleep_until_end();
            }
        }
    }
}