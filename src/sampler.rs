use std::{collections::HashMap, error::Error, mem};

use wmidi::{ControlFunction, MidiMessage};

use crate::{midi::InputDevice, patch::Patch, settings::Settings};

pub struct Sampler {
    midi_in: InputDevice,
    settings: HashMap<(u8, u8, u8), Settings>
}

const CHANNEL_MAX: usize = 16;

impl Sampler {
    pub fn new(midi_in: InputDevice, settings_list: Vec<Settings>) -> Self {
        let mut settings = HashMap::new();
        for s in settings_list {
            settings.insert((s.msb, s.lsb, s.pc), s);
        }
        Self {
            midi_in,
            settings
        }
    }

    pub fn listen(mut self) -> Result<(), Box<dyn Error>> {
        const NONE: Option<Patch> = None;
        let mut patch: [Option<Patch>; CHANNEL_MAX] = [NONE; CHANNEL_MAX];
        let mut msb = [0; CHANNEL_MAX];
        let mut lsb = [0; CHANNEL_MAX];
        let mut pc = [0; CHANNEL_MAX];
        loop {
            match self.midi_in.read()? {
                MidiMessage::ControlChange(c, ControlFunction::BANK_SELECT, m) => {
                    msb[c as usize] = m.into();
                },
                MidiMessage::ControlChange(c, ControlFunction::BANK_SELECT_LSB, l) => {
                    lsb[c as usize] = l.into();
                },
                MidiMessage::ProgramChange(c, p) => {
                    pc[c as usize] = u8::from(p) + 1; // pc is 1 based
                    let mut temp = None;
                    mem::swap(&mut patch[c as usize], &mut temp);
                    if let Some(old_patch) = temp {
                        old_patch.finish_all_sounds();
                    }
                    if let Some(new_settings) = self.settings.get(&(msb[c as usize], lsb[c as usize], pc[c as usize])) {
                        println!("{:?} - Patch: {}", c, new_settings.name);
                        patch[c as usize] = Some(Patch::from(new_settings));
                    } else {
                        println!("{:?} - No patch", c);
                        patch[c as usize] = None;
                    }
                },
                MidiMessage::NoteOn(c, n, _) => {
                    println!("{:?} - Note: {:?}", c, n);
                    if let Some(current) = &mut patch[c as usize] {
                        current.play(n);
                    }
                },
                _ => ()
            }
        }
    }
}