use std::{collections::HashMap, error::Error, mem};

use wmidi::{ControlFunction, MidiMessage};

use crate::{midi::InputDevice, patch::Patch, settings::Settings, status::Status};

pub struct Sampler<S> {
    midi_in: InputDevice,
    settings: HashMap<(u8, u8, u8), Settings>,
    status: S
}

const CHANNEL_MAX: usize = 16;

impl<S: Status> Sampler<S> {
    pub fn new(midi_in: InputDevice, settings_list: Vec<Settings>, status: S) -> Self {
        let mut settings = HashMap::new();
        for s in settings_list {
            settings.insert((s.msb, s.lsb, s.pc), s);
        }
        Self {
            midi_in,
            settings,
            status
        }
    }

    pub fn listen(mut self) -> Result<(), Box<dyn Error>> {
        const NONE: Option<Patch> = None;
        let mut patch: [Option<Patch>; CHANNEL_MAX] = [NONE; CHANNEL_MAX];
        let mut msb = [0; CHANNEL_MAX];
        let mut lsb = [0; CHANNEL_MAX];
        let mut pc = [0; CHANNEL_MAX];
        self.status.no_patch();
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
                        self.status.patch_unloading(&old_patch);
                        old_patch.finish_all_sounds();
                    }
                    if let Some(new_settings) = self.settings.get(&(msb[c as usize], lsb[c as usize], pc[c as usize])) {
                        self.status.patch_loading(new_settings);
                        println!("{:?} - Patch: {}", c, new_settings.name);
                        let new_patch = Patch::from(new_settings);
                        self.status.patch_ready(&new_patch);
                        patch[c as usize] = Some(new_patch);
                    } else {
                        self.status.no_patch();
                    }
                },
                MidiMessage::NoteOn(c, n, _) => {
                    println!("{:?} - Note: {:?}", c, n);
                    if let Some(current) = &mut patch[c as usize] {
                        if current.play(n) {
                            self.status.sound_played(n);
                        }
                    }
                },
                _ => ()
            }
        }
    }
}