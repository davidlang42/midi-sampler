use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::time::Instant;

use serde_derive::{Deserialize, Serialize};
use wmidi::{MidiMessage, ControlFunction, U7, Channel};

use crate::midi::{MidiReceiver, self};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Settings {
    name: String,
    lsb: u8,
    msb: u8,
    pc: u8,
    samples: HashMap<String, String>//TODO HashMap<Note, Path>
}

impl Settings {
    pub fn load(file: String) -> Result<Vec<Self>, Box<dyn Error>> {
        let json = fs::read_to_string(&file).map_err(|e| format!("Cannot read from '{}': {}", file, e))?;
        let settings: Vec<Settings> = serde_json::from_str(&format!("[{}]", json)).map_err(|e| format!("Cannot parse settigs from '{}': {}", file, e))?;
        Ok(settings)
    }
}

pub trait SettingsGetter: MidiReceiver {
    fn get(&self) -> &Settings;
}

pub struct PredefinedProgramChanges {
    predefined: Vec<Settings>,
    index: usize,
    msb: u8,
    lsb: u8,
    pc: u8
}

impl MidiReceiver for PredefinedProgramChanges {
    fn passthrough_midi(&mut self, message: MidiMessage<'static>) -> Option<MidiMessage<'static>> {
        match message {
            MidiMessage::ControlChange(_, ControlFunction::BANK_SELECT, msb) => {
                self.msb = msb.into();
                None
            },
            MidiMessage::ControlChange(_, ControlFunction::BANK_SELECT_LSB, lsb) => {
                self.lsb = lsb.into();
                None
            },
            MidiMessage::ProgramChange(_, pc) => {
                self.pc = pc.into();
                self.index = ((self.msb as usize * u8::from(U7::MAX) as usize + self.lsb as usize) * u8::from(U7::MAX) as usize + self.pc as usize) % self.predefined.len();
                None
            },
            _ => Some(message)
        }
    }
}

impl SettingsGetter for PredefinedProgramChanges {
    fn get(&self) -> &Settings {
        &self.predefined[self.index]
    }
}

impl PredefinedProgramChanges {
    pub fn new(predefined: Vec<Settings>) -> Self {
        if predefined.len() > u8::from(U7::MAX) as usize * u8::from(U7::MAX) as usize * u8::from(U7::MAX) as usize {
            panic!("Too many predefined program changes for 3 U7s");
        }
        Self {
            predefined,
            msb: 0,
            lsb: 0,
            pc: 0,
            index: 0
        }
    }
}

pub struct BpmDetector {
    ticks: usize,
    last_beat: Instant,
    last_bpm: usize
}

impl BpmDetector {
    pub fn _new() -> Self {
        Self {
            ticks: 0,
            last_beat: Instant::now(),
            last_bpm: 0
        }
    }

    pub fn _get(&self) -> usize {
        self.last_bpm
    }
}

impl MidiReceiver for BpmDetector {
    fn passthrough_midi(&mut self, message: MidiMessage<'static>) -> Option<MidiMessage<'static>> {
        if let MidiMessage::TimingClock = message {
            self.ticks += 1;
            if self.ticks == 24 {
                self.ticks = 0;
                let now = Instant::now();
                let ns = now.duration_since(self.last_beat).as_nanos();
                self.last_beat = now;
                let bpm = (60000000000.0 / ns as f64).round() as usize;
                if bpm != self.last_bpm {
                    self.last_bpm = bpm;
                }
            }
        }
        Some(message)
    }
}

pub struct NoteCounter {
    midi_channel: Channel,
    notes: [usize; Self::COUNT_PERIOD],
    ticks: usize,
    last_note_count: usize
}

impl NoteCounter {
    const COUNT_PERIOD: usize = midi::TICKS_PER_BEAT; // 1 quarter note

    pub fn _new(midi_channel: Channel) -> Self {
        Self {
            midi_channel,
            ticks: 0,
            notes: [0; Self::COUNT_PERIOD],
            last_note_count: 0,
        }
    }

    pub fn _get(&self) -> usize {
        self.last_note_count
    }
}

impl MidiReceiver for NoteCounter {
    fn passthrough_midi(&mut self, message: MidiMessage<'static>) -> Option<MidiMessage<'static>> {
        match message {
            MidiMessage::TimingClock => {
                self.ticks += 1;
                if self.ticks == self.notes.len() {
                    self.ticks = 0;
                    let note_count = self.notes.iter().filter(|&&c| c > 0).count();
                    if note_count != self.last_note_count {
                        self.last_note_count = note_count;
                    }
                    for i in 0..Self::COUNT_PERIOD {
                        self.notes[i] = 0;
                    }
                }
                Some(message)
            },
            MidiMessage::NoteOn(c, _, _) if c == self.midi_channel => {
                self.notes[self.ticks] += 1;
                None // don't forward notes on this channel
            },
            _ => Some(message)
        }
    }
}