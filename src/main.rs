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
mod logging;
mod tui;

use std::ffi::OsString;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use chan::{Sender, Receiver};
use chan_signal::Signal;

use gdb::GDB;
use logging::{
    Logger,
    LogMsgType,
};
use gdbmi::OutOfBandRecordSink;
use gdbmi::output::OutOfBandRecord;
use unsegen::base::Terminal;
use unsegen::container::{Application, ApplicationBehavior, LayoutNode};

use unsegen::input::{Input};


const EVENT_BUFFER_DURATION_MS: u64 = 10;


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

type UpdateParameters<'u> = &'u mut UpdateParametersStruct;

pub struct UpdateParametersStruct {
    pub gdb: GDB,
    pub logger: Logger,
}

// A timer that can be used to receive an event at any time,
// but will never send until started via try_start_ms.
struct Timer {
    receiver: Receiver<()>,
    sender: Option<Sender<()>>,
}
impl Timer {
    fn new() -> Self {
        let (sender, receiver) = chan::sync(0);
        Timer {
            receiver: receiver,
            sender: Some(sender),
        }
    }

    // Try to start the timer if it has not been started already.
    fn try_start(&mut self, duration: Duration) {
        if let Some(sender) = self.sender.take() {
            std::thread::spawn(move || {
                std::thread::sleep(duration);
                drop(sender);
            });
        }
    }
}
impl Deref for Timer {
    type Target = Receiver<()>;

    fn deref(&self) -> &Self::Target {
        &self.receiver
    }
}
impl DerefMut for Timer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.receiver
    }
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
    let gdb = GDB::new(gdbmi::GDB::spawn(gdb_arguments, tui_terminal.get_slave_name(), MpscOobRecordSink(oob_sink)).expect("spawn gdb"));

    // Setup input piping
    let (keyboard_sink, keyboard_source) = chan::async();

    /* let keyboard_input = */ ::std::thread::spawn(move || {
        let stdin = ::std::io::stdin();
        let stdin = stdin.lock();
        for e in Input::real_all(stdin) {
            keyboard_sink.send(e.expect("event"));
        }
    });
    let stdout = std::io::stdout();

    let left_pane = LayoutNode::VerticalSplit(vec![
          LayoutNode::Container("srcview".to_owned()),
          LayoutNode::Container("console".to_owned()),
    ]);
    let right_pane = LayoutNode::VerticalSplit(vec![
          LayoutNode::Container("expressiontable".to_owned()),
          LayoutNode::Container("terminal".to_owned()),
    ]);
    let layout = LayoutNode::HorizontalSplit(vec![
          left_pane,
          right_pane,
    ]);

    let mut update_parameters = UpdateParametersStruct {
        gdb: gdb,
        logger: Logger::new(),
    };

    {
        let mut terminal = Terminal::new(stdout.lock());
        let theme_set = syntect::highlighting::ThemeSet::load_defaults();
        let mut tui = tui::Tui::new(tui_terminal, &theme_set.themes["base16-ocean.dark"]);

        let mut app = Application::<tui::Tui>::from_layout_tree(layout).expect("Valid layout tree");

        tui.draw(terminal.create_root_window());
        terminal.present();

        // Somehow ipc.requests does not work in the chan_select macro...
        let ipc_requests = &mut ipc.requests;

        'runloop: loop {

            let mut timer = Timer::new();
            'displayloop: loop {
                chan_select! {
                    timer.recv() => {
                        break 'displayloop;
                    },
                    keyboard_source.recv() -> evt => {
                        let sig_behavior = ::unsegen_signals::SignalBehavior::new().sig_default::<::unsegen_signals::SIGTSTP>();
                        evt.expect("read keyboard event")
                            .chain(sig_behavior)
                            .chain(ApplicationBehavior::new(&mut app, &mut tui, &mut update_parameters));
                    },
                    oob_source.recv() -> oob_evt => {
                        if let Some(record) = oob_evt {
                            tui.add_out_of_band_record(record, &mut update_parameters);
                        } else {
                            // OOB pipe has closed. => gdb will be stopping soon
                            break 'runloop;
                        }
                    },
                    ipc_requests.recv() -> request => {
                        request.expect("receive request").respond(&mut update_parameters);
                    },
                    pts_source.recv() -> pty_output => {
                        tui.add_pty_input(pty_output.expect("get pty input"));
                    },
                    signal_event_source.recv() -> signal_event => {
                        let sig = signal_event.expect("get signal event");
                        match sig {
                            Signal::WINCH => { /* Ignore, we just want to redraw */ },
                            Signal::TSTP => { terminal.handle_sigtstp() },
                            Signal::TERM => { update_parameters.gdb.kill() },
                            _ => {}
                        }
                        update_parameters.logger.log(LogMsgType::Debug, format!("received signal {:?}", sig));
                    },
                }
                tui.update_after_event(&mut update_parameters);
                timer.try_start(Duration::from_millis(EVENT_BUFFER_DURATION_MS));
            }
            tui.console.display_log(&mut update_parameters.logger);
            //tui.draw(terminal.create_root_window());
            app.draw(terminal.create_root_window(), &mut tui);
            terminal.present();
        }
    }

    //keyboard_input.stop_loop(); //TODO make sure all loops stop?

    let child_exit_status = update_parameters.gdb.mi.process.wait().expect("gdb exited");
    println!("GDB exited with status {}.", child_exit_status);
}
