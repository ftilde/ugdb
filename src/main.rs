#[macro_use]
extern crate chan;
extern crate chan_signal;
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

mod pty;

mod tui;
mod input;

use std::thread;

use chan::Sender;
use chan_signal::Signal;

use gdbmi::{
    GDB,
    OutOfBandRecordSink,
};

use gdbmi::output::{
    OutOfBandRecord,
};

use unsegen::base::{
    Style,
    Terminal,
};

fn pty_output_loop(sink: Sender<Vec<u8>>, mut reader: pty::PTYOutput) {
    use ::std::io::Read;

    let mut buffer = [0; 1024];
    while let Ok(n) = reader.read(&mut buffer) {
        let mut bytes = vec![0; n];
        bytes.copy_from_slice(&mut buffer[..n]);
        sink.send(bytes);
    }
}

struct MpscOobRecordSink(Sender<OutOfBandRecord>);

impl OutOfBandRecordSink for MpscOobRecordSink {
    fn send(&self, data: OutOfBandRecord) {
        self.0.send(data);
    }
}

fn main() {
    // Setup signal piping:
    // NOTE: This has to be set up before the creation of any other threads!
    // (See chan_signal documentation)
    let signal_event_source = chan_signal::notify(&[Signal::WINCH]);

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
    let (oob_sink, oob_source) = chan::async();
    let mut gdb = GDB::spawn(executable_path, process_pty.name(), MpscOobRecordSink(oob_sink)).expect("spawn gdb");

    // Setup pty piping
    let (pty_input, pty_output) = process_pty.split_io();
    let (pty_output_sink, pty_output_source) = chan::async();
    /*let ptyThread = */ thread::spawn(move || {
        pty_output_loop(pty_output_sink, pty_output);
    });

    // Setup input piping
    let (keyboard_sink, keyboard_source) = chan::async();
    use input::InputSource;
    /* let keyboard_input = */ input::ViKeyboardInput::start_loop(keyboard_sink);

    let stdout = std::io::stdout();
    {

        let mut terminal = Terminal::new(stdout.lock());
        let theme_set = syntect::highlighting::ThemeSet::load_defaults();
        let mut tui = tui::Tui::new(pty_input, &theme_set.themes["base16-ocean.dark"]);
        tui.add_debug_message(&ptyname);

        tui.draw(terminal.create_root_window(Style::default()));
        terminal.present();

        loop {
            chan_select! {
                oob_source.recv() -> oob_evt => {
                    if let Some(record) = oob_evt {
                        tui.add_out_of_band_record(record, &mut gdb);
                    } else {
                        break; // TODO why silent fail/break?
                    }
                },
                keyboard_source.recv() -> evt => {
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
                pty_output_source.recv() -> pty_output => {
                    tui.add_pty_input(pty_output.expect("get pty input"));
                },
                signal_event_source.recv() -> signal_event => {
                    match signal_event.expect("get signal event") {
                        Signal::WINCH => { /* Ignore, we just want to redraw */ },
                        sig => { panic!(format!("unexpected {:?}", sig)) },
                    }
                }
            }
            tui.draw(terminal.create_root_window(Style::default()));
            terminal.present();
        }
    }

    //keyboard_input.stop_loop(); //TODO make sure all loops stop?

    let child_exit_status = gdb.process.wait().expect("gdb exited");
    println!("GDB exited with status {}.", child_exit_status);
}
