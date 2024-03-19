use std::{collections::HashMap, error::Error};

use wmidi::{ControlFunction, MidiMessage};

use crate::{midi::InputDevice, patch::Patch, settings::Settings};

pub struct Sampler {
    midi_in: InputDevice,
    settings: HashMap<(u8, u8, u8), Settings>,
    msb: u8,
    lsb: u8,
    pc: u8
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
                    self.pc = pc.into();
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
                //TODO handle pedal
                // MidiMessage::ControlChange(_, ControlFunction::DAMPER_PEDAL, value) => {
                //     let new_pedal = u8::from(value) >= 64;
                //     if self.pedal != new_pedal {
                //         self.pedal = new_pedal;
                //         if !self.pedal {
                //             // pedal released
                //             let notes_to_release: Vec<Note> = self.pedal_notes_off.drain().collect();
                //             for n in notes_to_release {
                //                 self.release_note(n);
                //             }
                //         }
                //     }
                // },
                // MidiMessage::NoteOn(c, n, v) => {
                //     if self.pedal_notes_off.remove(&n) { // this implies self.pedal
                //         // we are re-pressing a note which isn't actually off yet, because we're holding the pedal
                //         // so we just removed it from what will be released when the pedal is released
                //     } else {
                //         self.held_notes.insert(n, (Instant::now(), NoteDetails::new(c, n, v, settings.fixed_velocity)));
                //     }
                // },
                // MidiMessage::NoteOff(_, n, _) => {
                //     if self.pedal {
                //         // if the pedal is down, we don't actually release the note, just add it to a list
                //         // when the pedal is released, all the notes in the list get "released"
                //         self.pedal_notes_off.insert(n);
                //     } else {
                //         self.release_note(n);
                //     }
                // },
                // MidiMessage::TimingClock => {
                // },
                //TODO handle midi reset
                // MidiMessage::Reset => {
                //     self.held_notes.clear();
                //     drain_and_force_stop_vec(&mut self.arpeggios)?;
                // },
            }
        }
    }
}