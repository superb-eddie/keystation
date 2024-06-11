use midir::{MidiOutput, MidiOutputConnection};
use midir::os::unix::VirtualOutput;

const MIDI_CLIENT_NAME: &str = "keystation";
const MIDI_PORT_NAME: &str = "midi_out";
const MIDI_CHANNEL: u8 = 0; // 0-15

const MIDI_NOTE_ON: u8 = 0x90 + MIDI_CHANNEL;
const MIDI_NOTE_OFF: u8 = 0x80 + MIDI_CHANNEL;

pub fn start() -> MidiOutputConnection {
    let midi_out = MidiOutput::new(MIDI_CLIENT_NAME).unwrap();

    midi_out.create_virtual(MIDI_PORT_NAME).unwrap()
}

pub fn note_on(midi_out: &mut MidiOutputConnection, note: u8, velocity: u8) {
    // notes 0-127
    // velocity 0-127

    let message = &[MIDI_NOTE_ON, note.min(127), velocity.min(127)];

    midi_out.send(message).unwrap();
}

pub fn note_off(midi_out: &mut MidiOutputConnection, note: u8) {
    let message = &[MIDI_NOTE_OFF, note.min(127), 0];

    midi_out.send(message).unwrap();
}
