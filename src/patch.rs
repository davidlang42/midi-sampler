use rodio::Sink;
use wmidi::Note;
use std::{collections::HashMap, error::Error, fs};

pub struct Patch {
    data: [Option<Vec<u8>>; Self::NOTE_MAX],
    playing: HashMap<Note, Vec<Sink>>
}

impl Patch {
    const NOTE_MAX: usize = 127;

    pub fn from(settings: Settings) -> Self {
        todo!()
    }

    pub fn play(note: Note) {
        todo!()
    }

    pub fn stop(note: Note) {
        todo!()
    }
}