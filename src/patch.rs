use rodio::Sink;
use wmidi::Note;
use std::collections::HashMap;

use crate::settings::Settings;

pub struct Patch {
    data: [Option<Vec<u8>>; Self::NOTE_MAX],
    playing: HashMap<Note, Vec<Sink>>
}

impl Patch {
    const NOTE_MAX: usize = 127;

    pub fn from(settings: &Settings) -> Self {
        todo!()//TODO
    }

    pub fn play(&mut self, note: Note) {
        todo!()//TODO
    }

    pub fn finish_all_sounds(self) {
        todo!()//TODO
    }
}