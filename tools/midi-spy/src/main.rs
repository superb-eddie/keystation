use midir::{MidiInput};

const MIDI_CLIENT_NAME: &str = "midi-spy";

fn main() {
    let midi_in = MidiInput::new(MIDI_CLIENT_NAME).unwrap();

    for input_port in midi_in.ports() {
        let port_name = midi_in.port_name(&input_port).unwrap();
        println!("{}", port_name)
    }
}
