use std::io::Write;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::{io, sync::mpsc::Receiver};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use gpio::{sysfs::SysFsGpioOutput, GpioOut};
use wmidi::{Channel, Note};

use crate::patch::Patch;
use crate::settings::Settings;

pub trait Status {
    fn no_patch(&mut self, channel: Option<Channel>);
    fn patch_loading(&mut self, channel: Channel, settings: &Settings);
    fn patch_ready(&mut self, channel: Channel, patch: &Patch);
    fn patch_unloading(&mut self, channel: Channel, patch: &Patch);
    fn sound_played(&mut self, channel: Channel, note: Note);
}

pub struct GpioStatus {
    _thread: JoinHandle<()>,
    tx: Sender<State>,
    last: Option<TriColour>
}

#[derive(Copy, Clone, PartialEq)]
enum State {
    Off,
    Solid(TriColour),
    Flashing {
        colour: TriColour,
        on: bool,
        count: Option<usize>
    }
}

impl State {
    fn flashing(colour: TriColour) -> Self {
        Self::Flashing { colour, on: true, count: None }
    }

    fn flash_once(colour: TriColour) -> Self {
        Self::Flashing { colour, on: false, count: Some(1) }
    }

    const FOREVER: Duration = Duration::from_millis(60000);
    const FLASH: Duration = Duration::from_millis(300);

    fn set(self, red: &mut SysFsGpioOutput, green: &mut SysFsGpioOutput) -> (State, Duration) {
        match self {
            State::Off => {
                Self::set_colour(red, green, None).unwrap();
                (self, Self::FOREVER)
            }
            State::Solid(colour) => {
                Self::set_colour(red, green, Some(colour)).unwrap();
                (self, Self::FOREVER)
            },
            State::Flashing { colour, on, count } => {
                Self::set_colour(red, green, if on { Some(colour) } else { None }).unwrap();
                match count {
                    Some(0) => (State::Solid(colour), Self::FLASH),
                    Some(remaining) => (State::Flashing { colour, on: !on, count: Some(remaining - 1) }, Self::FLASH),
                    None => (State::Flashing { colour, on: !on, count: None }, Self::FLASH)
                }
            }
        }
    }

    fn set_colour(red: &mut SysFsGpioOutput, green: &mut SysFsGpioOutput, colour: Option<TriColour>) -> io::Result<()> {
        let (r, g) = match colour {
            None => (false, false),
            Some(TriColour::Red) => (true, false),
            Some(TriColour::Orange) => (true, true),
            Some(TriColour::Green) => (false, true)
        };
        red.set_value(r)?;
        green.set_value(g)?;
        Ok(())
    }

    fn colour(&self) -> Option<TriColour> {
        match self {
            State::Off => None,
            State::Solid(colour) => Some(*colour),
            State::Flashing { colour, on: _, count: _ } => Some(*colour)
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum TriColour {
    Red,
    Orange,
    Green
}

impl GpioStatus {
    pub fn init(red_pin: u16, green_pin: u16) -> io::Result<Self> {
        let red = gpio::sysfs::SysFsGpioOutput::open(red_pin)?;
        let green = gpio::sysfs::SysFsGpioOutput::open(green_pin)?;
        let (tx, rx) = mpsc::channel();
        let thread = thread::spawn(move || Self::update_loop(red, green, rx));
        Ok(Self {
            _thread: thread,
            tx,
            last: None
        })
    }

    fn send(&mut self, state: State) {
        self.last = state.colour();
        self.tx.send(state).unwrap();
    }

    fn update_loop(mut red: SysFsGpioOutput, mut green: SysFsGpioOutput, rx: Receiver<State>) {
        let (mut next_state, mut timeout) = State::Off.set(&mut red, &mut green);
        loop {
            let result = rx.recv_timeout(timeout);
            match result {
                Ok(new_state) => {
                    (next_state, timeout) = new_state.set(&mut red, &mut green);
                },
                Err(RecvTimeoutError::Timeout) => {
                    (next_state, timeout) = next_state.set(&mut red, &mut green);
                },
                Err(RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }
        State::Off.set(&mut red, &mut green);
    }
}

impl Status for GpioStatus {
    fn no_patch(&mut self, _channel: Option<Channel>) {
        if self.last == Some(TriColour::Red) {
            self.send(State::flash_once(TriColour::Red));
        } else {
            self.send(State::Solid(TriColour::Red));
        }
    }

    fn patch_loading(&mut self, _channel: Channel, _settings: &Settings) {
        self.send(State::Solid(TriColour::Orange));
    }

    fn patch_ready(&mut self, _channel: Channel, _patch: &Patch) {
        self.send(State::Solid(TriColour::Green));
    }

    fn patch_unloading(&mut self, _channel: Channel, _patch: &Patch) {
        self.send(State::flashing(TriColour::Orange));
    }
    
    fn sound_played(&mut self, _channel: Channel, _note: Note) {
        self.send(State::flash_once(TriColour::Green));
    }
}

pub struct TextStatus<W: Write>(pub W);

impl<W: Write> Status for TextStatus<W> {
    fn no_patch(&mut self, channel: Option<Channel>) {
        if let Some(c) = channel {
            writeln!(self.0, "[{:?}] No patch", c).unwrap();
        } else {
            writeln!(self.0, "No patch on any channel").unwrap();
        }
    }

    fn patch_loading(&mut self, channel: Channel, settings: &Settings) {
        writeln!(self.0, "[{:?}] Loading patch: {}", channel, settings.name).unwrap();
    }

    fn patch_ready(&mut self, channel: Channel, patch: &Patch) {
        writeln!(self.0, "[{:?}] Patch ready: {:?}", channel, patch.trigger_notes()).unwrap();
    }

    fn patch_unloading(&mut self, channel: Channel, patch: &Patch) {
        writeln!(self.0, "[{:?}] Unloading patch: {:?}", channel, patch.playing_sounds()).unwrap();
    }

    fn sound_played(&mut self, channel: Channel, note: Note) {
        writeln!(self.0, "[{:?}] Sound played: {:?}", channel, note).unwrap();
    }
}