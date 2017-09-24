#![recursion_limit = "200"] // See https://github.com/pest-parser/pest

#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate time;
extern crate gdbmi;

// For ipc
#[macro_use]
extern crate json;
extern crate rand;
extern crate unix_socket;

extern crate unsegen;

extern crate unsegen_signals;
extern crate unsegen_terminal;
extern crate unsegen_jsonviewer; // For ExpressionTable
#[macro_use]
extern crate pest; // For ExpressionTable (gdb structure parsing)

extern crate unicode_width; // For AssemblyLineDecorator

// These are used because (due to lifetime issues) we have to manage SyntaxSet, TermRead etc. ourselves
// TODO: maybe reexport types in unsegen?
extern crate syntect;

mod gdb;
mod input;
mod ipc;
mod tui;

use ::std::ffi::OsString;

use chan::Sender;
use chan_signal::Signal;

use gdb::GDB;
use gdbmi::{
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

type UpdateParameters<'a, 'b: 'a> = &'b mut UpdateParametersStruct<'a>;

pub struct UpdateParametersStruct<'a> {
    pub gdb: &'a mut GDB,
}

fn main() {
    // Setup signal piping:
    // NOTE: This has to be set up before the creation of any other threads!
    // (See chan_signal documentation)
    let signal_event_source = chan_signal::notify(&[Signal::WINCH, Signal::TSTP, Signal::TERM]);
    chan_signal::block(&[Signal::CONT]);

    // Create terminal and setup slave input piping
    let (pts_sink, pts_source) = chan::async();
    let tui_terminal = ::unsegen_terminal::Terminal::new(MpscSlaveInputSink(pts_sink));

    // Setup ipc
    let mut ipc = ipc::IPC::setup().expect("Setup ipc");

    // Start gdb and setup output event piping
    let (oob_sink, oob_source) = chan::async();
    let all_args: Vec<OsString> = ::std::env::args_os().collect();
    let gdb_arguments = &all_args[1..];
    let mut gdb = GDB::new(gdbmi::GDB::spawn(gdb_arguments, tui_terminal.get_slave_name(), MpscOobRecordSink(oob_sink)).expect("spawn gdb"));

    // Setup input piping
    let (keyboard_sink, keyboard_source) = chan::async();
    use input::InputSource;
    /* let keyboard_input = */ input::ViKeyboardInput::start_loop(keyboard_sink);

    macro_rules! update_parameters {
        () => {
            &mut UpdateParametersStruct {
                gdb: &mut gdb
            }
        }
    }

    let stdout = std::io::stdout();
    {
        let mut terminal = Terminal::new(stdout.lock());
        let theme_set = syntect::highlighting::ThemeSet::load_defaults();
        let mut tui = tui::Tui::new(tui_terminal, &theme_set.themes["base16-ocean.dark"]);

        tui.draw(terminal.create_root_window());
        terminal.present();

        // Somehow ipc.requests does not work in the chan_select macro...
        let ipc_requests = &mut ipc.requests;

        loop {
            chan_select! {
                oob_source.recv() -> oob_evt => {
                    if let Some(record) = oob_evt {
                        tui.add_out_of_band_record(record, update_parameters!());
                    } else {
                        // OOB pipe has closed. => gdb will be stopping soon
                        break;
                    }
                },
                ipc_requests.recv() -> request => {
                    request.expect("receive request").respond(update_parameters!());
                },
                keyboard_source.recv() -> evt => {
                    match evt.expect("read keyboard event") {
                        input::InputEvent::Quit => {
                            gdb.kill();
                        },
                        event => {
                            tui.event(event, update_parameters!());
                        },
                    }
                },
                pts_source.recv() -> pty_output => {
                    tui.add_pty_input(pty_output.expect("get pty input"));
                },
                signal_event_source.recv() -> signal_event => {
                    let sig = signal_event.expect("get signal event");
                    match sig {
                        Signal::WINCH => { /* Ignore, we just want to redraw */ },
                        Signal::TSTP => { terminal.handle_sigtstp() },
                        Signal::TERM => { gdb.kill() },
                        _ => {}
                    }
                    tui.console.add_debug_message(format!("received signal {:?}", sig));
                }
            }
            tui.update_after_event(update_parameters!());
            tui.draw(terminal.create_root_window());
            terminal.present();
        }
    }

    //keyboard_input.stop_loop(); //TODO make sure all loops stop?

    let child_exit_status = gdb.mi.process.wait().expect("gdb exited");
    println!("GDB exited with status {}.", child_exit_status);
}
