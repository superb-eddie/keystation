use crate::boot_animation::do_logo_scroll;
use crate::io::{init_io, IO};
use crate::midi_sender::start_midi_sink;
use crate::user_interface::do_ui;
use anyhow::Result;
use crossbeam::channel::unbounded;
use std::panic;
use std::thread::JoinHandle;

mod boot_animation;
mod midi_sender;
mod user_interface;
mod io;

pub type Threads = Vec<JoinHandle<Result<()>>>;

fn main() -> Result<()> {
    let mut threads = vec![];
    let (midi_sender, midi_receiver) = unbounded();
    let (ui_sender, ui_receiver) = unbounded();

    let mut io = init_io(
        &mut threads,
        midi_sender,
        ui_sender,
    )?;
    threads.push(start_midi_sink(midi_receiver));

    println!("IO initialized");

    do_logo_scroll(io.get_display());

    do_ui(io, ui_receiver, || {
        join_finished_threads(&mut threads)
    });
}

fn join_finished_threads(threads: &mut Threads) {
    let finished_threads = threads.iter().enumerate().filter_map(|(i, thread)| {
        if thread.is_finished() {
            return Some(i);
        } else {
            None
        }
    });

    for i in finished_threads.rev() {
        match threads.remove(i).join() {
            Err(p) => panic::resume_unwind(p),
            Ok(Err(e)) => panic!("{}", e),
            Ok(Ok(_)) => return,
        }
    }
}
