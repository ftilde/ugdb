#![recursion_limit = "200"] // See https://github.com/pest-parser/pest

#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate gdbmi;

extern crate unsegen;

extern crate unsegen_terminal;
extern crate unsegen_jsonviewer; // For ExpressionTable
#[macro_use]
extern crate pest; // For ExpressionTable (gdb structure parsing)

extern crate unicode_width; // For AssemblyLineDecorator

// These are used because (due to lifetime issues) we have to manage SyntaxSet, TermRead etc. ourselves
// TODO: maybe reexport types in unsegen?
extern crate syntect;
extern crate termion;

mod tui;
mod input;

use ::std::ffi::OsString;

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
    Terminal,
};

struct MpscOobRecordSink(Sender<OutOfBandRecord>);

impl OutOfBandRecordSink for MpscOobRecordSink {
    fn send(&self, data: OutOfBandRecord) {
        self.0.send(data);
    }
}

struct MpscSlaveInputSink(Sender<Box<[u8]>>);

impl ::unsegen_terminal::SlaveInputSink for MpscSlaveInputSink {
    fn send(&self, data: Box<[u8]>) {
        self.0.send(data);
    }
}

fn main() {
    // Setup signal piping:
    // NOTE: This has to be set up before the creation of any other threads!
    // (See chan_signal documentation)
    let signal_event_source = chan_signal::notify(&[Signal::WINCH]);

    // Create terminal and setup slave input piping
    let (pts_sink, pts_source) = chan::async();
    let tui_terminal = ::unsegen_terminal::Terminal::new(MpscSlaveInputSink(pts_sink));

    // Start gdb and setup output event piping
    let (oob_sink, oob_source) = chan::async();
    let all_args: Vec<OsString> = ::std::env::args_os().collect();
    let gdb_arguments = &all_args[1..];
    let mut gdb = GDB::spawn(gdb_arguments, tui_terminal.get_slave_name(), MpscOobRecordSink(oob_sink)).expect("spawn gdb");

    // Setup input piping
    let (keyboard_sink, keyboard_source) = chan::async();
    use input::InputSource;
    /* let keyboard_input = */ input::ViKeyboardInput::start_loop(keyboard_sink);

    let stdout = std::io::stdout();
    {

        let mut terminal = Terminal::new(stdout.lock());
        let theme_set = syntect::highlighting::ThemeSet::load_defaults();
        let mut tui = tui::Tui::new(tui_terminal, &theme_set.themes["base16-ocean.dark"]);

        tui.draw(terminal.create_root_window());
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
                pts_source.recv() -> pty_output => {
                    tui.add_pty_input(pty_output.expect("get pty input"));
                },
                signal_event_source.recv() -> signal_event => {
                    match signal_event.expect("get signal event") {
                        Signal::WINCH => { /* Ignore, we just want to redraw */ },
                        sig => { panic!(format!("unexpected {:?}", sig)) },
                    }
                }
            }
            tui.draw(terminal.create_root_window());
            terminal.present();
        }
    }

    //keyboard_input.stop_loop(); //TODO make sure all loops stop?

    let child_exit_status = gdb.process.wait().expect("gdb exited");
    println!("GDB exited with status {}.", child_exit_status);
}
