use std::thread;
use std::thread::JoinHandle;

use anyhow::Result;
use crossbeam::channel::Receiver;
use midir::MidiOutput;
use midir::os::unix::VirtualOutput;
use midly::live::LiveEvent;
use midly::MidiMessage;
use midly::num::u4;

const MIDI_CLIENT_NAME: &str = "keystation";
const MIDI_PORT_NAME: &str = "midi_out";

pub type MidiEvent = MidiMessage;

// Start a new thread to send midi events to the OS
pub fn start_midi_sink(midi_channel: Receiver<MidiEvent>) -> JoinHandle<Result<()>> {
    thread::spawn(move || -> Result<()> {
        let mut midi_out = MidiOutput::new(MIDI_CLIENT_NAME)?
            .create_virtual(MIDI_PORT_NAME)
            .expect("couldn't create virtual midi port");

        let mut buf = [0u8; 3];

        for e in midi_channel {
            let live_event = LiveEvent::Midi {
                channel: u4::new(0),
                message: e,
            };

            live_event.write_std(&mut buf[..]).unwrap();

            midi_out.send(&buf).unwrap()
        }

        Ok(())
    })
}
