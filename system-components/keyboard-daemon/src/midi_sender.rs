use crossbeam::channel::Receiver;
use midir::os::unix::VirtualOutput;
use midir::MidiOutput;
use std::thread;

const MIDI_CLIENT_NAME: &str = "keystation";
const MIDI_PORT_NAME: &str = "midi_out";
const MIDI_CHANNEL: u8 = 0; // 0-15
const MIDI_NOTE_ON: u8 = 0x90 + MIDI_CHANNEL;
const MIDI_NOTE_OFF: u8 = 0x80 + MIDI_CHANNEL;

const MIDI_CC: u8 = 0xB0 + MIDI_CHANNEL;
const MIDI_SUSTAIN_PEDAL: u8 = 0x40;

pub enum MidiEvent {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8 },
    SustainOn,
    SustainOff,
}

// Start a new thread to send midi events to the OS
pub fn start_midi_sink(midi_channel: Receiver<MidiEvent>) -> anyhow::Result<()> {
    let mut midi_out = MidiOutput::new(MIDI_CLIENT_NAME)?.create_virtual(MIDI_PORT_NAME)?;

    thread::spawn(move || {
        for e in midi_channel {
            let message = match e {
                MidiEvent::NoteOn { note, velocity } => {
                    [MIDI_NOTE_ON, note.min(127), velocity.min(127)]
                }
                MidiEvent::NoteOff { note } => [MIDI_NOTE_OFF, note.min(127), 0],
                MidiEvent::SustainOn => [MIDI_CC, MIDI_SUSTAIN_PEDAL, 0x7F],
                MidiEvent::SustainOff => [MIDI_CC, MIDI_SUSTAIN_PEDAL, 0x00],
            };

            midi_out.send(&message).unwrap();
        }
    });

    Ok(())
}
