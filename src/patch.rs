use rodio::{OutputStream, OutputStreamHandle, Sink};
use wmidi::Note;
use std::{collections::HashMap, fs::File, io::{Cursor, Read}};

use crate::{settings::Settings, shared_vec::SharedVec};

pub struct Patch {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    data: [Option<SharedVec<u8>>; Self::NOTE_MAX],
    playing: [Vec<Sink>; Self::NOTE_MAX]
}

impl Patch {
    const NOTE_MAX: usize = 127;

    pub fn from(settings: &Settings) -> Self {
        const NONE_VEC_U8: Option<SharedVec<u8>> = None;
        const EMPTY_VEC_SINK: Vec<Sink> = Vec::new();
        let mut data = [NONE_VEC_U8; Self::NOTE_MAX];
        for (note, path) in &settings.samples {
            let n = Note::from_u8_lossy(*note as u8);
            let mut f = File::open(path).unwrap();
            let mut bytes = Vec::new();
            f.read_to_end(&mut bytes).unwrap();
            data[n as usize] = Some(bytes.into());
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
            let cursor = Cursor::new(bytes.clone());
            let new = self.handle.play_once(cursor).unwrap();
            let sinks = &mut self.playing[note as usize];
            sinks.retain(|s| !s.empty());
            sinks.push(new);
            true
        } else {
            false
        }
    }

    pub fn trigger_notes(&self) -> Vec<Note> {
        let mut v = Vec::new();
        for d in 0..self.data.len() {
            if self.data[d].is_some() {
                v.push(Note::from_u8_lossy(d as u8));
            }
        }
        v
    }

    pub fn playing_sounds(&self) -> HashMap<Note, usize> {
        let mut map = HashMap::new();
        for d in 0..self.data.len() {
            if let Some(v) = &self.data[d] {
                map.insert(Note::from_u8_lossy(d as u8), v.instances() - 1); // 1 instance is owned by the patch itself
            }
        }
        map
    }

    pub fn finish_all_sounds(self) {
        for vec in self.playing {
            for sink in vec {
                sink.sleep_until_end();
            }
        }
    }
}