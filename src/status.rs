use std::io;

use gpio::{sysfs::SysFsGpioOutput, GpioOut};

pub trait Status {
    fn patch_cleared(&mut self);
    fn patch_loading(&mut self);
    fn patch_unloading(&mut self);
    fn patch_ready(&mut self);
    fn sound_played(&mut self);
}

pub struct GpioStatus {
    red: SysFsGpioOutput,
    green: SysFsGpioOutput,
    state: TriColour
}

#[derive(Copy, Clone, PartialEq)]
enum TriColour {
    Off,
    Red,
    Orange,
    Green
}

impl GpioStatus {
    pub fn init(red_pin: u16, green_pin: u16) -> io::Result<Self> {
        let mut s = Self {
            red: gpio::sysfs::SysFsGpioOutput::open(red_pin)?,
            green: gpio::sysfs::SysFsGpioOutput::open(green_pin)?,
            state: TriColour::Off
        };
        s.update()?;
        Ok(s)
    }

    fn update(&mut self) -> io::Result<()> {
        let (r, g) = match self.state {
            TriColour::Off => (false, false),
            TriColour::Red => (true, false),
            TriColour::Orange => (true, true),
            TriColour::Green => (false, true)
        };
        self.red.set_value(r)?;
        self.green.set_value(g)?;
        Ok(())
    }
}

impl Status for GpioStatus {
    fn patch_cleared(&mut self) {
        if self.state == TriColour::Red {
            //TODO flash off then red
        } else {
            self.state = TriColour::Red;
            self.update().unwrap();
        }
    }

    fn patch_loading(&mut self) {
        self.state = TriColour::Orange;
        self.update().unwrap();
    }

    fn patch_unloading(&mut self) {
        //TODO could flash continuously instead
        self.patch_loading();
    }

    fn patch_ready(&mut self) {
        self.state = TriColour::Green;
        self.update().unwrap();
    }
    
    fn sound_played(&mut self) {
        //TODO flash off then green
    }
}