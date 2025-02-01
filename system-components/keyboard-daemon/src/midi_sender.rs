use std::thread;

use crossbeam::channel::Receiver;
use midir::MidiOutput;
use midir::os::unix::VirtualOutput;
use midly::live::LiveEvent;
use midly::MidiMessage;
use midly::num::u4;

const MIDI_CLIENT_NAME: &str = "keystation";
const MIDI_PORT_NAME: &str = "midi_out";
const MIDI_CHANNEL: u8 = 0; // 0-15
const MIDI_NOTE_ON: u8 = 0x90 + MIDI_CHANNEL;
const MIDI_NOTE_OFF: u8 = 0x80 + MIDI_CHANNEL;

const MIDI_CC: u8 = 0xB0 + MIDI_CHANNEL;
const MIDI_SUSTAIN_PEDAL: u8 = 0x40;

pub type MidiEvent = MidiMessage;

// Start a new thread to send midi events to the OS
pub fn start_midi_sink(midi_channel: Receiver<MidiEvent>) -> anyhow::Result<()> {
    thread::spawn(move || {
        let mut midi_out = MidiOutput::new(MIDI_CLIENT_NAME)
            .unwrap()
            .create_virtual(MIDI_PORT_NAME)
            .unwrap();

        let mut buf = [0u8; 3];

        for e in midi_channel {
            let live_event = LiveEvent::Midi {
                channel: u4::new(0),
                message: e,
            };

            live_event.write_std(&mut buf[..]).unwrap();

            midi_out.send(&buf).unwrap()
        }
    });

    Ok(())
}
