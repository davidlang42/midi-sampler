use std::sync::mpsc;
use std::fs;
use std::thread;
use std::io::Read;
use std::error::Error;
use std::thread::JoinHandle;
use wmidi::FromBytesError;
use wmidi::MidiMessage;
use wmidi::U7;

pub struct InputDevice {
    receiver: mpsc::Receiver<MidiMessage<'static>>,
    threads: Vec<JoinHandle<()>>
}

impl InputDevice {
    pub fn open(midi_in: &str, include_clock_ticks: bool) -> Result<Self, Box<dyn Error>> {
        let (tx, rx) = mpsc::channel();
        let mut input = fs::File::options().read(true).open(midi_in).map_err(|e| format!("Cannot open MIDI IN '{}': {}", midi_in, e))?;
        let join_handle = thread::Builder::new().name(format!("midi-in")).spawn(move || Self::read_into_queue(&mut input, tx, include_clock_ticks, true))?;
        Ok(Self {
            receiver: rx,
            threads: vec![join_handle]
        })
    }

    pub fn read(&mut self) -> Result<MidiMessage<'static>, mpsc::RecvError> {
        for thread in &self.threads {
            if thread.is_finished() {
                panic!("InputDevice thread finished");
            }
        }
        self.receiver.recv()
    }

    fn read_into_queue(f: &mut fs::File, tx: mpsc::Sender<MidiMessage>, include_clock_ticks: bool, rewrite_note_zero_as_off: bool) {
        let mut buf: [u8; 1] = [0; 1];
        let mut bytes = Vec::new();
        while f.read_exact(&mut buf).is_ok() {
            bytes.push(buf[0]);
            match MidiMessage::try_from(bytes.as_slice()) {
                Ok(MidiMessage::TimingClock) if !include_clock_ticks => {
                    // skip clock tick if not required
                    bytes.clear();
                },
                Ok(MidiMessage::NoteOn(c, n, U7::MIN)) if rewrite_note_zero_as_off => {
                    // some keyboards send NoteOn(velocity: 0) instead of NoteOff (eg. Kaysound MK-4902)
                    if tx.send(MidiMessage::NoteOff(c, n, U7::MIN)).is_err() {
                        panic!("Error rewriting NoteOn(0) as NoteOff to queue.");
                    }
                    bytes.clear();
                },
                Ok(message) => {
                    // message complete, send to queue
                    if tx.send(message.to_owned()).is_err() {
                        panic!("Error sending to queue.");
                    }
                    bytes.clear();
                },
                Err(FromBytesError::NoBytes) | Err(FromBytesError::NoSysExEndByte) | Err(FromBytesError::NotEnoughBytes) => {
                    // wait for more bytes
                }, 
                _ => {
                    // invalid message, clear and wait for next message
                    bytes.clear();
                }
            }
        }
        panic!("Input device has disconnected.");
    }
}