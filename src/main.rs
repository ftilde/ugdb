extern crate backtrace;
extern crate flexi_logger;
extern crate log;
extern crate nix;
extern crate structopt;
extern crate termion;
extern crate time;
#[macro_use]
extern crate derive_more;
extern crate parse_int;
extern crate unicode_segmentation;

// For ipc
#[macro_use]
extern crate json;
extern crate rand;
extern crate unix_socket;
extern crate unsegen;

extern crate unsegen_jsonviewer;
extern crate unsegen_pager;
extern crate unsegen_signals;
extern crate unsegen_terminal;

// gdbmi
#[macro_use]
extern crate nom;

mod completion;
mod gdb;
mod gdb_expression_parsing;
mod gdbmi;
mod ipc;
mod layout;
mod tui;

use ipc::IPCRequest;
use std::ffi::OsString;
use std::time::Duration;

use std::sync::mpsc::Sender;

use gdb::GDB;
use gdbmi::output::OutOfBandRecord;
use gdbmi::{GDBBuilder, OutOfBandRecordSink};
use log::{debug, warn};
use nix::sys::signal::Signal;
use nix::sys::termios;
use std::path::PathBuf;
use structopt::StructOpt;
use tui::{Tui, TuiContainerType};
use unsegen::base::{Color, StyleModifier, Terminal};
use unsegen::container::ContainerManager;
use unsegen::input::{Input, Key, NavigateBehavior, ToEvent};
use unsegen::widget::{Blink, RenderingHints};

const EVENT_BUFFER_DURATION_MS: u64 = 10;
const FOCUS_ESCAPE_MAX_DURATION_MS: u64 = 200;
const CURSOR_BLINK_PERIOD_MS: u64 = 500;
const CURSOR_BLINK_TIMES: u8 = 20;

#[derive(StructOpt)]
#[structopt()]
struct Options {
    #[structopt(
        long = "gdb",
        help = "Path to alternative gdb binary.",
        default_value = "gdb",
        parse(from_os_str)
    )]
    gdb_path: PathBuf,
    #[structopt(long = "nh", help = "Do not execute commands from ~/.gdbinit.")]
    nh: bool,
    #[structopt(
        short = "n",
        long = "nx",
        help = "Do not execute commands from any .gdbinit initialization files."
    )]
    nx: bool,
    #[structopt(
        short = "q",
        long = "quiet",
        help = "\"Quiet\".  Do not print the introductory and copyright messages.  These messages are also suppressed in batch mode."
    )]
    quiet: bool,
    #[structopt(
        long = "rr",
        help = "Start ugdb as an interface for rr. Trailing ugdb arguments will be passed to rr replay instead."
    )]
    rr: bool,
    #[structopt(
        long = "rr-path",
        help = "Path to alternative rr binary.",
        default_value = "rr",
        parse(from_os_str)
    )]
    rr_path: PathBuf,
    #[structopt(
        long = "cd",
        help = "Run GDB using directory as its working directory, instead of the current directory.",
        parse(from_os_str)
    )]
    cd: Option<PathBuf>,
    #[structopt(
        short = "b",
        help = "Set the line speed (baud rate or bits per second) of any serial interface used by GDB for remote debugging."
    )]
    bps: Option<u32>,
    #[structopt(
        short = "s",
        long = "symbols",
        help = "Read symbols from the given file.",
        parse(from_os_str)
    )]
    symbol_file: Option<PathBuf>,
    #[structopt(
        short = "c",
        long = "core",
        help = "Use file file as a core dump to examine.",
        parse(from_os_str)
    )]
    core_file: Option<PathBuf>,
    #[structopt(short = "p", long = "pid", help = "Attach to process with given id.")]
    proc_id: Option<u32>,
    #[structopt(
        short = "x",
        long = "command",
        help = "Execute GDB commands from file.",
        parse(from_os_str)
    )]
    command_file: Option<PathBuf>,
    #[structopt(
        short = "d",
        long = "directory",
        help = "Add directory to the path to search for source files.",
        parse(from_os_str)
    )]
    source_dir: Option<PathBuf>,
    #[structopt(
        long = "log_dir",
        help = "Directory in which the log file will be stored.",
        parse(from_os_str),
        default_value = "/tmp"
    )]
    log_dir: PathBuf,
    #[structopt(
        short = "e",
        long = "initial-expression",
        help = "Define initial entries for the expression table."
    )]
    initial_expression_table_entries: Vec<String>,
    #[structopt(
        long = "layout",
        help = "Define the initial tui layout via a format string.",
        default_value = "(1s-1c)|(1e-1t)"
    )]
    layout: String,
    #[structopt(
        help = "Path to program to debug (with arguments).",
        parse(from_os_str)
    )]
    program: Vec<OsString>,
    // Not sure how to mimic gdbs cmdline behavior for the positional arguments...
    //#[structopt(help="Attach to process with given id.")]
    //proc_id: Option<u32>,
    //#[structopt(help="Use file file as a core dump to examine.", parse(from_os_str))]
    //core_file: Option<PathBuf>,
}

impl Options {
    fn create_gdb_builder(self) -> GDBBuilder {
        let mut gdb_builder = GDBBuilder::new(self.gdb_path);
        if self.nh {
            gdb_builder = gdb_builder.nh();
        }
        if self.nx {
            gdb_builder = gdb_builder.nx();
        }
        if self.quiet {
            gdb_builder = gdb_builder.quiet();
        }
        if let Some(cd) = self.cd {
            gdb_builder = gdb_builder.working_dir(cd);
        }
        if let Some(bps) = self.bps {
            gdb_builder = gdb_builder.bps(bps);
        }
        if let Some(symbol_file) = self.symbol_file {
            gdb_builder = gdb_builder.symbol_file(symbol_file);
        }
        if let Some(core_file) = self.core_file {
            gdb_builder = gdb_builder.core_file(core_file);
        }
        if let Some(proc_id) = self.proc_id {
            gdb_builder = gdb_builder.proc_id(proc_id);
        }
        if let Some(command_file) = self.command_file {
            gdb_builder = gdb_builder.command_file(command_file);
        }
        if let Some(src_dir) = self.source_dir {
            gdb_builder = gdb_builder.source_dir(src_dir);
        }
        if self.rr {
            gdb_builder = gdb_builder.rr_args(self.rr_path, self.program);
        } else {
            let (program, args) = self
                .program
                .split_first()
                .map(|(p, a)| (Some(p), a))
                .unwrap_or_else(|| (None, &[]));
            gdb_builder = gdb_builder.args(args);
            if let Some(program) = program {
                gdb_builder = gdb_builder.program(PathBuf::from(program));
            }
        }

        gdb_builder
    }
}

struct MpscOobRecordSink(Sender<Event>);

impl OutOfBandRecordSink for MpscOobRecordSink {
    fn send(&self, data: OutOfBandRecord) {
        self.0.send(Event::OutOfBandRecord(data)).unwrap();
    }
}

impl Drop for MpscOobRecordSink {
    fn drop(&mut self) {
        self.0.send(Event::GdbShutdown).unwrap();
    }
}

struct MpscSlaveInputSink(Sender<Event>);

impl ::unsegen_terminal::SlaveInputSink for MpscSlaveInputSink {
    fn receive_bytes_from_pty(&mut self, data: Box<[u8]>) {
        self.0.send(Event::Pty(data)).unwrap();
    }
}

pub struct MessageSink {
    messages: Vec<String>,
}

impl MessageSink {
    pub fn send<S: Into<String>>(&mut self, msg: S) {
        self.messages.push(msg.into());
    }
    pub fn drain_messages(&mut self) -> Vec<String> {
        let mut alt_buffer = Vec::new();
        ::std::mem::swap(&mut self.messages, &mut alt_buffer);
        alt_buffer
    }
}

pub struct Context {
    pub gdb: GDB,
    event_sink: Sender<Event>,
}

impl Context {
    fn log(&mut self, msg: impl AsRef<str>) {
        self.event_sink
            .send(Event::Log(format!("{}\n", msg.as_ref())))
            .unwrap();
    }

    fn try_change_layout(&mut self, layout_str: String) {
        self.event_sink
            .send(Event::ChangeLayout(layout_str))
            .unwrap();
    }

    fn show_file(&mut self, file: String, line: unsegen::base::LineNumber) {
        self.event_sink.send(Event::ShowFile(file, line)).unwrap();
    }
}

// A timer that can be used to receive an event at any time,
// but will never send until started via try_start_ms.
struct MpscTimer {
    next_sender: Option<Sender<Event>>,
    sender: Sender<Event>,
    evt_fn: Box<dyn Fn() -> Event>,
    counter: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl MpscTimer {
    fn new(sender: Sender<Event>, evt_fn: Box<dyn Fn() -> Event>) -> Self {
        MpscTimer {
            next_sender: Some(sender.clone()),
            sender,
            evt_fn,
            counter: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    // Try to start the timer if it has not been started already.
    fn try_start(&mut self, duration: Duration) {
        if let Some(sender) = self.next_sender.take() {
            let start_number = self.counter.load(std::sync::atomic::Ordering::SeqCst);
            let counter = self.counter.clone();
            let evt = (self.evt_fn)();
            let _ = std::thread::spawn(move || {
                std::thread::sleep(duration);
                let current = counter.load(std::sync::atomic::Ordering::SeqCst);
                if current == start_number {
                    sender.send(evt).unwrap();
                }
            });
        }
    }

    fn has_been_started(&self) -> bool {
        self.next_sender.is_none()
    }

    fn reset(&mut self) {
        self.next_sender = Some(self.sender.clone());
        self.counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Drop for MpscTimer {
    fn drop(&mut self) {
        self.counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Clone, Copy, Debug)]
enum InputMode {
    Normal,
    Focused,
    ContainerSelect,
}

impl InputMode {
    fn associated_border_style(self) -> StyleModifier {
        match self {
            InputMode::Normal => StyleModifier::new(),
            InputMode::Focused => StyleModifier::new().fg_color(Color::Red),
            InputMode::ContainerSelect => StyleModifier::new().fg_color(Color::LightYellow),
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Signal(nix::sys::signal::Signal),
    Input(Input),
    Pty(Box<[u8]>),
    CursorTimer,
    RenderTimer,
    FocusEscTimer,
    OutOfBandRecord(OutOfBandRecord),
    Log(String),
    ChangeLayout(String),
    ShowFile(String, unsegen::base::LineNumber),
    GdbShutdown,
    Ipc(IPCRequest),
}

fn run() -> i32 {
    // Setup signal piping:
    let mut signals_to_wait = nix::sys::signal::SigSet::empty();
    signals_to_wait.add(Signal::SIGWINCH);
    signals_to_wait.add(Signal::SIGTSTP);
    signals_to_wait.add(Signal::SIGTERM);
    let mut signals_to_block = signals_to_wait.clone();
    signals_to_block.add(Signal::SIGCONT);

    // We block the signals for the current (and so far only thread). This mask will be inherited
    // by all other threads spawned subsequently, so that we retrieve signals using sigwait.
    signals_to_block.thread_block().unwrap();

    let (event_sink, event_source) = std::sync::mpsc::channel();

    let signal_sink = event_sink.clone();
    ::std::thread::spawn(move || loop {
        if let Ok(signal) = signals_to_wait.wait() {
            signal_sink.send(Event::Signal(signal)).unwrap();
        }
    });

    // Set up a panic hook that ALWAYS displays panic information (including stack) to the main
    // terminal screen.
    const STDOUT: std::os::unix::io::RawFd = 0;
    let orig_attr = std::sync::Mutex::new(
        termios::tcgetattr(STDOUT).expect("Failed to get terminal attributes"),
    );

    let options = Options::from_args();
    let log_dir = options.log_dir.to_owned();
    let initial_expression_table_entries = options.initial_expression_table_entries.clone();
    let layout = options.layout.clone();

    ::std::panic::set_hook(Box::new(move |info| {
        // Switch back to main screen
        println!("{}{}", termion::screen::ToMainScreen, termion::cursor::Show);
        // Restore old terminal behavior (will be restored later automatically, but we want to be
        // able to properly print the panic info)
        let _ = termios::tcsetattr(STDOUT, termios::SetArg::TCSANOW, &orig_attr.lock().unwrap());

        println!("Oh no! ugdb crashed!");
        println!(
            "Consider filing an issue including the log file located in {} and the following backtrace at {}:\n",
            log_dir.to_string_lossy(),
            env!("CARGO_PKG_REPOSITORY"),
        );

        println!("{}", info);
        println!("{:?}", backtrace::Backtrace::new());
    }));

    if let Err(e) = flexi_logger::Logger::with_env_or_str("info")
        .log_to_file()
        .directory(options.log_dir.to_owned())
        .start()
    {
        eprintln!("Unable to initialize Logger: {}", e);
        return 0xfe;
    }

    // Create terminal and setup slave input piping
    let tui_terminal = ::unsegen_terminal::Terminal::new(MpscSlaveInputSink(event_sink.clone()))
        .expect("Create PTY");

    // Setup ipc
    let _ipc = ipc::IPC::setup(event_sink.clone()).expect("Setup ipc");

    // Start gdb and setup output event piping
    let gdb_path = options.gdb_path.to_string_lossy().to_string();
    let mut gdb_builder = options.create_gdb_builder();
    gdb_builder = gdb_builder.tty(tui_terminal.slave_name().into());
    let gdb = GDB::new(
        match gdb_builder.try_spawn(MpscOobRecordSink(event_sink.clone())) {
            Ok(gdb) => gdb,
            Err(e) => {
                eprintln!("Failed to spawn gdb process (\"{}\"): {}", gdb_path, e);
                return 0xfc;
            }
        },
    );

    let stdout = std::io::stdout();

    let theme_set = unsegen_pager::ThemeSet::load_defaults();

    let layout = match layout::parse(layout) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{}", e);
            return 0xfb;
        }
    };

    let mut context = Context {
        gdb,
        event_sink: event_sink.clone(),
    };

    {
        let mut terminal = match Terminal::new(stdout.lock()) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Unable to setup Terminal: {}", e);
                return 0xfd;
            }
        };
        let mut tui = Tui::new(tui_terminal, &theme_set.themes["base16-ocean.dark"]);
        for entry in initial_expression_table_entries {
            tui.expression_table.add_entry(entry);
        }

        // Start stdin thread _after_ building terminal (and setting the actual terminal to raw
        // mode to avoid race condition where the first 'set of input' is buffered
        /* let keyboard_input = */
        let keyboard_sink = event_sink.clone();
        ::std::thread::spawn(move || {
            let stdin = ::std::io::stdin();
            let stdin = stdin.lock();
            for e in Input::read_all(stdin) {
                keyboard_sink.send(Event::Input(e.expect("event"))).unwrap();
            }
        });

        let mut app = ContainerManager::<Tui>::from_layout(layout);
        let mut input_mode = InputMode::Normal;
        let mut focus_esc_timer =
            MpscTimer::new(event_sink.clone(), Box::new(|| Event::FocusEscTimer));
        let mut cursor_status = Blink::On;
        let mut cursor_blinks_since_last_input = 0;

        'runloop: loop {
            let mut cursor_update_timer =
                MpscTimer::new(event_sink.clone(), Box::new(|| Event::CursorTimer));
            if cursor_blinks_since_last_input < CURSOR_BLINK_TIMES {
                cursor_update_timer.try_start(Duration::from_millis(CURSOR_BLINK_PERIOD_MS));
            }

            let mut render_delay_timer =
                MpscTimer::new(event_sink.clone(), Box::new(|| Event::RenderTimer));
            let mut esc_timer_needs_reset = false;
            'displayloop: loop {
                let mut esc_in_focused_context_pressed = false;
                match event_source.recv().unwrap() {
                    Event::CursorTimer => {
                        cursor_status.toggle();
                        cursor_blinks_since_last_input += 1;
                        break 'displayloop;
                    }
                    Event::RenderTimer => {
                        cursor_status = Blink::On;
                        cursor_blinks_since_last_input = 0;
                        break 'displayloop;
                    }
                    Event::FocusEscTimer => {
                        Input {
                            event: Key::Esc.to_event(),
                            raw: vec![0x1bu8],
                        }
                        .chain(app.active_container_behavior(&mut tui, &mut context));
                        esc_timer_needs_reset = true;
                        break 'displayloop;
                    }
                    Event::Input(input) => {
                        let sig_behavior = ::unsegen_signals::SignalBehavior::new()
                            .on_default::<::unsegen_signals::SIGTSTP>();
                        let input = input.chain(sig_behavior);
                        match input_mode {
                            InputMode::ContainerSelect => input
                                .chain(
                                    NavigateBehavior::new(&mut app.navigatable(&mut tui))
                                        .up_on(Key::Char('k'))
                                        .up_on(Key::Up)
                                        .down_on(Key::Char('j'))
                                        .down_on(Key::Down)
                                        .left_on(Key::Char('h'))
                                        .left_on(Key::Left)
                                        .right_on(Key::Char('l'))
                                        .right_on(Key::Right),
                                )
                                .chain((Key::Char('i'), || {
                                    input_mode = InputMode::Normal;
                                    app.set_active(TuiContainerType::Console);
                                }))
                                .chain((Key::Char('e'), || {
                                    input_mode = InputMode::Normal;
                                    app.set_active(TuiContainerType::ExpressionTable);
                                }))
                                .chain((Key::Char('s'), || {
                                    input_mode = InputMode::Normal;
                                    app.set_active(TuiContainerType::SrcView);
                                }))
                                .chain((Key::Char('t'), || {
                                    input_mode = InputMode::Normal;
                                    app.set_active(TuiContainerType::Terminal);
                                }))
                                .chain((Key::Char('T'), || {
                                    input_mode = InputMode::Focused;
                                    app.set_active(TuiContainerType::Terminal);
                                }))
                                .chain((Key::Char('\n'), || input_mode = InputMode::Normal)),
                            InputMode::Normal => input
                                .chain((Key::Esc, || input_mode = InputMode::ContainerSelect))
                                .chain(app.active_container_behavior(&mut tui, &mut context)),
                            InputMode::Focused => input
                                .chain((Key::Esc, || esc_in_focused_context_pressed = true))
                                .chain(app.active_container_behavior(&mut tui, &mut context)),
                        }
                        .finish();
                    }
                    Event::OutOfBandRecord(record) => {
                        tui.add_out_of_band_record(record, &mut context);
                    }
                    Event::Log(msg) => {
                        tui.console.write_to_gdb_log(msg);
                    }
                    Event::ShowFile(file, line) => {
                        tui.src_view.show_file(file, line, &mut context);
                    }
                    Event::ChangeLayout(layout) => {
                        match layout::parse(layout) {
                            Ok(layout) => {
                                app.set_layout(layout);
                            }
                            Err(e) => {
                                tui.console.write_to_gdb_log(e.to_string());
                            }
                        };
                    }
                    Event::GdbShutdown => {
                        break 'runloop;
                    }
                    Event::Ipc(request) => {
                        request.respond(&mut context);
                    }
                    Event::Pty(pty_output) => {
                        tui.add_pty_input(&pty_output);
                    }
                    Event::Signal(signal_event) => {
                        let sig = signal_event;
                        match sig {
                            Signal::SIGWINCH => { /* Ignore, we just want to redraw */ }
                            Signal::SIGTSTP => {
                                if let Err(e) = terminal.handle_sigtstp() {
                                    warn!("Unable to handle SIGTSTP: {}", e);
                                }
                            }
                            Signal::SIGTERM => context.gdb.kill(),
                            _ => {}
                        }
                        debug!("received signal {:?}", sig);
                    }
                }
                if esc_in_focused_context_pressed {
                    if focus_esc_timer.has_been_started() {
                        input_mode = InputMode::ContainerSelect;
                    } else {
                        focus_esc_timer
                            .try_start(Duration::from_millis(FOCUS_ESCAPE_MAX_DURATION_MS));
                    }
                }
                tui.update_after_event(&mut context);
                render_delay_timer.try_start(Duration::from_millis(EVENT_BUFFER_DURATION_MS));
            }
            if esc_timer_needs_reset {
                focus_esc_timer.reset();
            }
            app.draw(
                terminal.create_root_window(),
                &mut tui,
                input_mode.associated_border_style(),
                RenderingHints::default().blink(cursor_status),
            );
            terminal.present();
        }
    }

    let mut join_retry_counter = 0;
    let join_retry_duration = Duration::from_millis(100);
    let child_exit_status = loop {
        if let Some(ret) = context.gdb.mi.process.try_wait().expect("gdb exited") {
            break ret;
        }
        std::thread::sleep(join_retry_duration);
        if join_retry_counter == 10 {
            println!("Waiting for GDB to exit...");
        }
        join_retry_counter += 1;
    };
    if child_exit_status.success() {
        0
    } else {
        println!("GDB exited with status {}.", child_exit_status);
        child_exit_status.code().unwrap_or(0xff)
    }
}

fn main() {
    let exit_code = run();
    std::process::exit(exit_code);
}
