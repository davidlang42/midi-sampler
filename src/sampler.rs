use std::error::Error;

use crate::{midi::{InputDevice, MidiReceiver}, settings::SettingsGetter};

pub struct Sampler<SG: SettingsGetter> {
    pub midi_in: InputDevice,
    pub settings: SG
}

impl<SG: SettingsGetter> Sampler<SG> {
    pub fn listen(self) -> Result<(), Box<dyn Error>> {
        self.listen_with_midi_receivers(Vec::new())
    }

    pub fn listen_with_midi_receivers(mut self, mut extra_midi_receivers: Vec<&mut dyn MidiReceiver>) -> Result<(), Box<dyn Error>> {
        let mut current = None;
        loop {
            let mut m = Some(self.midi_in.read()?);
            // pass message through extra receivers
            for midi_receiver in extra_midi_receivers.iter_mut() {
                m = midi_receiver.passthrough_midi(m.unwrap());
                if m.is_none() { break; }
            }
            // pass message through settings
            if m.is_none() { continue; }
            m = self.settings.passthrough_midi(m.unwrap());
            // handle settings changes
            self.status.update_settings(self.settings.get());
            let new_mode = self.settings.get().mode;
            if new_mode != mode {
                mode = new_mode;
                current.stop_arpeggios()?;
                current = new_mode.create(&self.midi_out);
                self.status.update_count(current.count_arpeggios());
            }
            // pass message through status
            if m.is_none() { continue; }
            m = self.status.passthrough_midi(m.unwrap());
            // process message in arp
            if m.is_none() { continue; }
            current.process(m.unwrap(), self.settings.get(), &mut self.status)?;
            self.status.update_count(current.count_arpeggios());
        }
    }
}