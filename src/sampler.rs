use std::{collections::HashMap, error::Error};

use wmidi::{ControlFunction, MidiMessage};

use crate::{midi::InputDevice, patch::Patch, settings::Settings};

pub struct Sampler {
    midi_in: InputDevice,
    settings: HashMap<(u8, u8, u8), Settings>,
    msb: u8, // 0-127
    lsb: u8, // 0-127
    pc: u8 // 1-128
}

impl Sampler {
    pub fn new(midi_in: InputDevice, settings_list: Vec<Settings>) -> Self {
        let mut settings = HashMap::new();
        for s in settings_list {
            settings.insert((s.msb, s.lsb, s.pc), s);
        }
        Self {
            midi_in,
            settings,
            msb: 0,
            lsb: 0,
            pc: 0
        }
    }

    pub fn listen(mut self) -> Result<(), Box<dyn Error>> {
        let mut patch: Option<Patch> = None;
        loop {
            match self.midi_in.read()? {
                MidiMessage::ControlChange(_, ControlFunction::BANK_SELECT, msb) => {
                    self.msb = msb.into();
                },
                MidiMessage::ControlChange(_, ControlFunction::BANK_SELECT_LSB, lsb) => {
                    self.lsb = lsb.into();
                },
                MidiMessage::ProgramChange(_, pc) => {
                    self.pc = u8::from(pc) + 1; // pc is 1 based
                    if let Some(old_patch) = patch {
                        old_patch.finish_all_sounds();
                    }
                    if let Some(new_settings) = self.settings.get(&(self.msb, self.lsb, self.pc)) {
                        println!("Patch: {}", new_settings.name);
                        patch = Some(Patch::from(new_settings));
                    } else {
                        println!("No patch");
                        patch = None;
                    }
                },
                MidiMessage::NoteOn(_, n, _) => {
                    //TODO handle channel, velocity
                    println!("Note: {:?}", n);
                    if let Some(current) = &mut patch {
                        current.play(n);
                    }
                },
                _ => ()
            }
        }
    }
}