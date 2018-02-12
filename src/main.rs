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
mod ipc;
mod logging;
mod tui;

use std::ffi::OsString;
use std::ops::{Deref, DerefMut};
use std::time::{Duration};

use chan::{Sender, Receiver};
use chan_signal::Signal;

use gdb::GDB;
use logging::{Logger};
use gdbmi::OutOfBandRecordSink;
use gdbmi::output::OutOfBandRecord;
use unsegen::base::{Color, StyleModifier, Terminal};
use unsegen::container::{Application, HSplit, VSplit, Leaf};
use unsegen::input::{Input, Key, NavigateBehavior, ToEvent};
use tui::{Tui, TuiContainerType};


const EVENT_BUFFER_DURATION_MS: u64 = 10;
const FOCUS_ESCAPE_MAX_DURATION_MS: u64 = 200;


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
struct MpscTimer {
    receiver: Receiver<()>,
    sender: Option<Sender<()>>,
}
impl MpscTimer {
    fn new() -> Self {
        let (sender, receiver) = chan::sync(0);
        MpscTimer {
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

    fn has_been_started(&self) -> bool {
        self.sender.is_none()
    }

    fn reset(&mut self) {
        let (sender, receiver) = chan::sync(0);
        self.receiver = receiver;
        self.sender = Some(sender);
    }
}
impl Deref for MpscTimer {
    type Target = Receiver<()>;

    fn deref(&self) -> &Self::Target {
        &self.receiver
    }
}
impl DerefMut for MpscTimer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.receiver
    }
}

#[derive(Clone, Copy, Debug)]
enum InputMode {
    Normal,
    Focused,
    ContainerSelect,
}

impl InputMode {
    fn associated_border_style(&self) -> StyleModifier {
        match self {
            &InputMode::Normal => StyleModifier::none(),
            &InputMode::Focused => StyleModifier::new().fg_color(Color::Red),
            &InputMode::ContainerSelect => StyleModifier::new().fg_color(Color::LightYellow),
        }
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

    let theme_set = syntect::highlighting::ThemeSet::load_defaults();

    let left_pane = VSplit::new(vec![
          Box::new(Leaf::new(TuiContainerType::SrcView)),
          Box::new(Leaf::new(TuiContainerType::Console)),
    ]);
    let right_pane = VSplit::new(vec![
          Box::new(Leaf::new(TuiContainerType::ExpressionTable)),
          Box::new(Leaf::new(TuiContainerType::Terminal)),
    ]);
    let layout = HSplit::new(vec![
          Box::new(left_pane),
          Box::new(right_pane),
    ]);

    let mut update_parameters = UpdateParametersStruct {
        gdb: gdb,
        logger: Logger::new(),
    };

    {
        let mut terminal = Terminal::new(stdout.lock());
        let mut tui = Tui::new(tui_terminal, &theme_set.themes["base16-ocean.dark"]);

        let mut app = Application::<Tui>::from_layout(Box::new(layout));
        let mut input_mode = InputMode::Normal;
        let mut focus_esc_timer = MpscTimer::new();

        // Somehow ipc.requests does not work in the chan_select macro...
        let ipc_requests = &mut ipc.requests;

        'runloop: loop {

            let mut render_delay_timer = MpscTimer::new();
            let mut esc_timer_needs_reset = false;
            'displayloop: loop {
                let mut esc_in_focused_context_pressed = false;
                #[allow(unused_mut)] { // Not sure where the unused mut in the chan_select macro is coming from...
                chan_select! {
                    render_delay_timer.recv() => {
                        break 'displayloop;
                    },
                    focus_esc_timer.recv() => {
                        Input { event: Key::Esc.to_event(), raw: vec![0x1bu8] }.chain(app.active_container_behavior(&mut tui, &mut update_parameters));
                        esc_timer_needs_reset = true;
                        break 'displayloop;
                    },
                    keyboard_source.recv() -> input => {
                        let sig_behavior = ::unsegen_signals::SignalBehavior::new().sig_default::<::unsegen_signals::SIGTSTP>();
                        let input = input.expect("read keyboard event")
                            .chain(sig_behavior);
                        match input_mode {
                            InputMode::ContainerSelect => {
                                input
                                    .chain(NavigateBehavior::new(&mut app.navigatable(&mut tui))
                                           .up_on(Key::Char('k'))
                                           .up_on(Key::Up)
                                           .down_on(Key::Char('j'))
                                           .down_on(Key::Down)
                                           .left_on(Key::Char('h'))
                                           .left_on(Key::Left)
                                           .right_on(Key::Char('l'))
                                           .right_on(Key::Right)
                                          )
                                    .chain((Key::Char('i'), || { input_mode = InputMode::Normal; app.set_active(TuiContainerType::Console); }))
                                    .chain((Key::Char('e'), || { input_mode = InputMode::Normal; app.set_active(TuiContainerType::ExpressionTable); }))
                                    .chain((Key::Char('s'), || { input_mode = InputMode::Normal; app.set_active(TuiContainerType::SrcView); }))
                                    .chain((Key::Char('t'), || { input_mode = InputMode::Normal; app.set_active(TuiContainerType::Terminal); }))
                                    .chain((Key::Char('T'), || { input_mode = InputMode::Focused; app.set_active(TuiContainerType::Terminal); }))
                                    .chain((Key::Char('\n'), || input_mode = InputMode::Normal ))
                            }
                            InputMode::Normal => {
                                input
                                    .chain((Key::Esc, || input_mode = InputMode::ContainerSelect ))
                                    .chain(app.active_container_behavior(&mut tui, &mut update_parameters))
                            }
                            InputMode::Focused => {
                                input
                                    .chain((Key::Esc, || esc_in_focused_context_pressed = true ))
                                    .chain(app.active_container_behavior(&mut tui, &mut update_parameters))
                            }
                        }.finish();
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
                        update_parameters.logger.log_debug(format!("received signal {:?}", sig));
                    },
                }
                }
                if esc_in_focused_context_pressed {
                    if focus_esc_timer.has_been_started() {
                        input_mode = InputMode::ContainerSelect;
                    } else {
                        focus_esc_timer.try_start(Duration::from_millis(FOCUS_ESCAPE_MAX_DURATION_MS));
                    }
                }
                tui.update_after_event(&mut update_parameters);
                render_delay_timer.try_start(Duration::from_millis(EVENT_BUFFER_DURATION_MS));
            }
            if esc_timer_needs_reset {
                focus_esc_timer.reset();
            }
            tui.console.display_log(&mut update_parameters.logger);
            app.draw(terminal.create_root_window(), &mut tui, input_mode.associated_border_style());
            terminal.present();
        }
    }

    //keyboard_input.stop_loop(); //TODO make sure all loops stop?

    let child_exit_status = update_parameters.gdb.mi.process.wait().expect("gdb exited");
    println!("GDB exited with status {}.", child_exit_status);
}
