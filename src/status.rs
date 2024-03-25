use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::{io, sync::mpsc::Receiver};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use gpio::{sysfs::SysFsGpioOutput, GpioOut};

pub trait Status {
    fn patch_cleared(&mut self);
    fn patch_loading(&mut self);
    fn patch_unloading(&mut self);
    fn patch_ready(&mut self);
    fn sound_played(&mut self);
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
    fn patch_cleared(&mut self) {
        if self.last == Some(TriColour::Red) {
            self.send(State::flash_once(TriColour::Red));
        } else {
            self.send(State::Solid(TriColour::Red));
        }
    }

    fn patch_loading(&mut self) {
        self.send(State::Solid(TriColour::Orange));
    }

    fn patch_unloading(&mut self) {
        self.send(State::flashing(TriColour::Orange));
    }

    fn patch_ready(&mut self) {
        self.send(State::Solid(TriColour::Green));
    }
    
    fn sound_played(&mut self) {
        self.send(State::flash_once(TriColour::Green));
    }
}