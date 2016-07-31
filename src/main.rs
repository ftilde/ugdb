#![feature(mpsc_select)]

// For unsegen
extern crate termion;
extern crate ndarray;
//extern crate pty;

// For pty
extern crate libc;
extern crate nix;

// For gdbmi
#[macro_use]
extern crate nom;


mod unsegen;
mod gui;
mod gdbmi;
mod pty;

use gdbmi::*;
use std::sync::mpsc;
use std::thread;

#[derive(Eq, PartialEq, Clone, Copy)]
enum Input {
    Event(termion::event::Event),
    Quit,
}

macro_rules! nc_printw_attr{
    ($att:expr, $fmt:expr, $($arg:tt)*) => {{
        //ncurses::attron($att);
        ncurses::printw(format!($fmt $(, $arg)*).as_ref());
        //ncurses::attroff($att);
    }}
}
macro_rules! nc_printw{
    ($fmt:expr, $($arg:tt)*) => {{
        ncurses::printw(format!($fmt $(, $arg)*).as_ref());
    }}
}

fn keyboard_input_loop(output: mpsc::Sender<Input>) {
    use termion::input::TermRead;

    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    for c in stdin.keys() {
        output.send(Input::Event(termion::event::Event::Key(c.unwrap()))).unwrap();
    }
    output.send(Input::Quit).unwrap();
}

fn pty_output_loop(sink: mpsc::Sender<String>, reader: pty::PTYOutput) {
    use ::std::io::Read;
    let mut reader = std::io::BufReader::new(reader);
    loop { //TODO how to stop that loop?
        let mut buf = String::new();
        reader.read_to_string(&mut buf);
        sink.send(buf);
    }
}

fn main() {
    use unsegen::Widget;
    let process_pty = pty::PTY::open().expect("Could not create pty.");
    let executable_path = "/home/dominik/test2";

    //println!("PTY: {}", process_pty.name());

    let (mut gdb, out_of_band_pipe)  = GDB::spawn(executable_path, process_pty.name()).unwrap();

    let (pty_input, pty_output) = process_pty.split_io();

    let (pty_output_sink, pty_output_source) = mpsc::channel();
    /*let ptyThread = */ thread::spawn(move || {
        pty_output_loop(pty_output_sink, pty_output);
    });

    let (keyboard_sink, keyboard_source) = mpsc::channel();
    /*let inputThread = */ thread::spawn(move || {
        keyboard_input_loop(keyboard_sink);
    });

    let stdout = std::io::stdout();
    {
        let mut terminal = unsegen::Terminal::new(stdout.lock());
        let mut gui = gui::Gui::new(pty_input);

        gui.draw(terminal.create_root_window(unsegen::TextAttribute::default()));
        terminal.present();

        loop {
            select! {
                oob_evt = out_of_band_pipe.recv() => {
                    if let Ok(record) = oob_evt {
                        gui.add_out_of_band_record(record);
                    } else {
                        break;
                    }
                },
                keyboard_evt = keyboard_source.recv() => {
                    let evt = keyboard_evt.unwrap();
                    match evt {
                        Input::Quit => break,
                        Input::Event(event) => { gui.event(event, &mut gdb); },
                    }
                },
                pty_output = pty_output_source.recv() => {
                    gui.add_pty_output(pty_output.unwrap());
                }
            }
            gui.draw(terminal.create_root_window(unsegen::TextAttribute::default()));
            terminal.present();
        }
    }

    let child_exit_status = gdb.process.wait().unwrap();
    println!("GDB exited with status {}.", child_exit_status);
}
