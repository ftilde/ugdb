#![feature(mpsc_select)]

extern crate gdbmi;
extern crate unsegen;
extern crate unicode_width;

// These are used because (due to lifetime issues) we have to manage SyntaxSet, TermRead etc. ourselves
// TODO: maybe reexport types in unsegen?
extern crate syntect;
extern crate termion;

// For pty
extern crate libc;

//For gdbmi AND pty
extern crate nix;

// For signalchannel
#[macro_use]
extern crate lazy_static;

mod pty;

mod tui;
mod input;

mod signalchannel;

use std::sync::mpsc;
use std::thread;

fn pty_output_loop(sink: mpsc::Sender<Vec<u8>>, mut reader: pty::PTYOutput) {
    use ::std::io::Read;

    let mut buffer = [0; 1024];
    while let Ok(n) = reader.read(&mut buffer) {
        let mut bytes = vec![0; n];
        bytes.copy_from_slice(&mut buffer[..n]);
        sink.send(bytes).expect("send bytes");
    }
}

fn main() {
    let process_pty = pty::PTY::open().expect("Could not create pty.");
    let executable_path = "/home/dominik/gdbmi-test/test";

    //println!("PTY: {}", process_pty.name());
    let ptyname = process_pty.name().to_owned();

    // Hack:
    // Open slave terminal, so that it does not get destroyed when a gdb process opens it and
    // closes it afterwards.
    let mut pts = std::fs::OpenOptions::new().write(true).read(true).open(&ptyname).expect("pts file");
    use std::io::Write;
    write!(pts, "").expect("initial write to pts");

    // Start gdb and setup output event piping
    let (mut gdb, out_of_band_pipe)  = gdbmi::GDB::spawn(executable_path, process_pty.name()).expect("spawn gdb");

    // Setup pty piping
    let (pty_input, pty_output) = process_pty.split_io();
    let (pty_output_sink, pty_output_source) = mpsc::channel();
    /*let ptyThread = */ thread::spawn(move || {
        pty_output_loop(pty_output_sink, pty_output);
    });

    // Setup input piping
    let (keyboard_sink, keyboard_source) = mpsc::channel();
    use input::InputSource;
    /* let keyboard_input = */ input::ViKeyboardInput::start_loop(keyboard_sink);

    // Setup signal piping
    let signal_event_source = signalchannel::setup_signal_receiver().expect("took signal_event_source");

    let stdout = std::io::stdout();
    {

        let mut terminal = unsegen::Terminal::new(stdout.lock());
        let theme_set = syntect::highlighting::ThemeSet::load_defaults();
        let mut tui = tui::Tui::new(pty_input, &theme_set.themes["base16-ocean.dark"]);
        tui.add_debug_message(&ptyname);

        //tui.load_in_file_viewer("/home/dominik/test.rs");

        tui.draw(terminal.create_root_window(unsegen::TextAttribute::default()));
        terminal.present();

        loop {
            select! {
                oob_evt = out_of_band_pipe.recv() => {
                    if let Ok(record) = oob_evt {
                        tui.add_out_of_band_record(record, &mut gdb);
                    } else {
                        break; // TODO why silent fail/break?
                    }
                },
                evt = keyboard_source.recv() => {
                    match evt.expect("read keyboard event") {
                        input::InputEvent::Quit => {
                            gdb.interrupt_execution().expect("interrupt worked");
                            gdb.execute_later(&gdbmi::input::MiCommand::exit());
                        },
                        event => {
                            tui.event(event, &mut gdb);
                        },
                    }
                },
                pty_output = pty_output_source.recv() => {
                    tui.add_pty_input(pty_output.expect("get pty input"));
                },
                signal_event = signal_event_source.recv() => {
                    match signal_event.expect("get signal event") {
                        nix::sys::signal::Signal::SIGWINCH => { /* Ignore, we just want to redraw */ },
                        sig => { panic!(format!("unexpected {:?}", sig)) },
                    }
                }
            }
            tui.draw(terminal.create_root_window(unsegen::TextAttribute::default()));
            terminal.present();
        }
    }

    //keyboard_input.stop_loop(); //TODO make sure all loops stop?

    let child_exit_status = gdb.process.wait().expect("gdb exited");
    println!("GDB exited with status {}.", child_exit_status);
}
