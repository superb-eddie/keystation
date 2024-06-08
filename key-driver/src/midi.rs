use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};
use midir::os::unix::VirtualOutput;

const MIDI_CLIENT_NAME: &str = "keystation";
const MIDI_GADGET_NAME: &str = "keystation-gadget";
const MIDI_VIRTUAL_NAME: &str = "keystation-virtual";
const MIDI_CHANNEL: u8 = 0; // 0-15

const MIDI_NOTE_ON: u8 = 0x90 + MIDI_CHANNEL;
const MIDI_NOTE_OFF: u8 = 0x80 + MIDI_CHANNEL;

const MIDI_GADGET_PORT_NAME: &str = "f_midi:f_midi 24:0";

// We send midi out to the usb gadget and to a virtual midi port for on board instruments
type DualMidiOutputConnection = (MidiOutputConnection, MidiOutputConnection);

fn start_gadget() -> MidiOutputConnection {
    let midi_out = MidiOutput::new(MIDI_CLIENT_NAME).unwrap();
    
    for port in &midi_out.ports() {
        if midi_out.port_name(port).unwrap() == MIDI_GADGET_PORT_NAME {
            return midi_out.connect(port, MIDI_GADGET_NAME).unwrap()
        } 
    }
    
    panic!("Could not find midi port '{}'", MIDI_GADGET_PORT_NAME)
}

fn start_virtual() -> MidiOutputConnection {
    let midi_out = MidiOutput::new(MIDI_CLIENT_NAME).unwrap();

    midi_out.create_virtual(MIDI_VIRTUAL_NAME).unwrap()
}

pub fn start() -> DualMidiOutputConnection {
    (start_gadget(), start_virtual())
}

pub fn note_on(midi_out: &mut DualMidiOutputConnection, note: u8, velocity: u8) {
    // notes 0-127
    // velocity 0-127

    let message = &[MIDI_NOTE_ON, note.min(127), velocity.min(127)];

    midi_out.0.send(message).unwrap();
    midi_out.1.send(message).unwrap();
}

pub fn note_off(midi_out: &mut DualMidiOutputConnection, note: u8) {
    let message = &[MIDI_NOTE_OFF, note.min(127), 0];

    midi_out.0.send(message).unwrap();
    midi_out.1.send(message).unwrap();
}
