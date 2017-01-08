#![feature(mpsc_select)]

// For unsegen
extern crate termion;
extern crate ndarray;
//extern crate pty;

// For pty
extern crate libc;

// For gdbmi
#[macro_use]
extern crate nom;

//For gdbmi AND pty
extern crate nix;

mod unsegen;
mod gdbmi;
mod pty;

mod gui;
mod input;

use std::sync::mpsc;
use std::thread;

fn pty_output_loop(sink: mpsc::Sender<u8>, reader: pty::PTYOutput) {
    use ::std::io::Read;
    let reader = reader;

    for b in reader.bytes() {
        sink.send(b.unwrap()).unwrap();
    }
}

fn main() {
    let process_pty = pty::PTY::open().expect("Could not create pty.");
    let executable_path = "/home/dominik/gdbmi-test/test";

    //println!("PTY: {}", process_pty.name());
    let ptyname = process_pty.name().to_owned();

    let (mut gdb, out_of_band_pipe)  = gdbmi::GDB::spawn(executable_path, process_pty.name()).unwrap();

    let (pty_input, pty_output) = process_pty.split_io();

    let (pty_output_sink, pty_output_source) = mpsc::channel();
    /*let ptyThread = */ thread::spawn(move || {
        pty_output_loop(pty_output_sink, pty_output);
    });

    let (keyboard_sink, keyboard_source) = mpsc::channel();

    use input::InputSource;
    /* let keyboard_input = */ input::ViKeyboardInput::start_loop(keyboard_sink);

    let stdout = std::io::stdout();
    {
        let mut terminal = unsegen::Terminal::new(stdout.lock());
        let mut gui = gui::Gui::new(pty_input);
        gui.add_debug_message(&ptyname);

        gui.draw(terminal.create_root_window(unsegen::TextAttribute::default()));
        terminal.present();

        loop {
            select! {
                oob_evt = out_of_band_pipe.recv() => {
                    if let Ok(record) = oob_evt {
                        gui.add_out_of_band_record(record);
                    } else {
                        break; // TODO why silent fail/break?
                    }
                },
                keyboard_evt = keyboard_source.recv() => {
                    let evt = keyboard_evt.unwrap();
                    match evt {
                        input::InputEvent::Quit => break,
                        event => { gui.event(event, &mut gdb); },
                    }
                },
                pty_output = pty_output_source.recv() => {
                    gui.add_pty_input(pty_output.unwrap());
                }
            }
            gui.draw(terminal.create_root_window(unsegen::TextAttribute::default()));
            terminal.present();
        }
    }

    //keyboard_input.stop_loop(); //TODO make sure all loops stop?

    let child_exit_status = gdb.process.wait().unwrap();
    println!("GDB exited with status {}.", child_exit_status);
}
