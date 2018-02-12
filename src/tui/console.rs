use gdbmi;
use logging::{
    Logger,
    LogMsgType,
};

use unsegen::base::{
    GraphemeCluster,
    Window,
};
use unsegen::input::{
    EditBehavior,
    Input,
    Key,
    ScrollBehavior,
};
use unsegen::widget::{
    Demand2D,
    SeparatingStyle,
    VerticalLayout,
    RenderingHints,
    Widget,
};
use unsegen::widget::widgets::{
    LogViewer,
    PromptLine,
};
use unsegen::container::Container;

enum ActiveLog {
    Debug,
    Gdb,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum GDBState {
    Running,
    Stopped,
}

pub struct Console {
    debug_log: LogViewer,
    gdb_log: LogViewer,
    active_log: ActiveLog,
    prompt_line: PromptLine,
    layout: VerticalLayout,
    last_gdb_state: GDBState,
}

static STOPPED_PROMPT: &'static str = "(gdb) ";
static RUNNING_PROMPT: &'static str = "(↻↻↻) ";

impl Console {
    pub fn new() -> Self {
        Console {
            debug_log: LogViewer::new(),
            gdb_log: LogViewer::new(),
            active_log: ActiveLog::Gdb,
            prompt_line: PromptLine::with_prompt(STOPPED_PROMPT.into()),
            layout: VerticalLayout::new(SeparatingStyle::Draw(GraphemeCluster::try_from('=').unwrap())),
            last_gdb_state: GDBState::Stopped,
        }
    }

    pub fn display_log(&mut self, logger: &mut Logger) {
        use std::fmt::Write;
        for (msg_type, msg) in logger.drain_messages() {
            match msg_type {
                LogMsgType::Debug => {
                    writeln!(self.debug_log.storage, " -=- {}", msg).expect("Write Debug Message");
                },
                LogMsgType::Message => {
                    writeln!(self.gdb_log.storage, "{}", msg).expect("Write Message");
                },
            }
        }
    }

    pub fn write_to_gdb_log<S: AsRef<str>>(&mut self, msg: S) {
        use std::fmt::Write;
        write!(self.gdb_log.storage, "{}", msg.as_ref()).expect("Write Message");
    }

    fn toggle_active_log(&mut self) {
        self.active_log = match self.active_log {
            ActiveLog::Debug => ActiveLog::Gdb,
            ActiveLog::Gdb => ActiveLog::Debug,
        };
    }

    fn get_active_log_viewer_mut(&mut self) -> &mut LogViewer {
        match self.active_log {
            ActiveLog::Debug => &mut self.debug_log,
            ActiveLog::Gdb => &mut self.gdb_log,
        }
    }

    fn get_active_log_viewer(&self) -> &LogViewer {
        match self.active_log {
            ActiveLog::Debug => &self.debug_log,
            ActiveLog::Gdb => &self.gdb_log,
        }
    }

    fn handle_newline(&mut self, p: ::UpdateParameters) {
        let line = if self.prompt_line.active_line().is_empty() {
            self.prompt_line.previous_line(1).unwrap_or("").to_owned()
        } else {
            self.prompt_line.finish_line().to_owned()
        };
        match line.as_ref() {
            "!stop" => {
                p.gdb.mi.interrupt_execution().expect("interrupted gdb");

                // This does not always seem to unblock gdb, but only hang it
                //use gdbmi::input::MiCommand;
                //gdb.execute(&MiCommand::exec_interrupt()).expect("Interrupt ");
            },
            // Gdb commands
            _ => {
                self.write_to_gdb_log(format!("{}{}\n", STOPPED_PROMPT, line));
                match p.gdb.mi.execute(&gdbmi::input::MiCommand::cli_exec(line)) {
                    Ok(result) => {
                        p.logger.log_debug(format!("Result: {:?}", result));
                    },
                    Err(gdbmi::ExecuteError::Quit) => {
                        self.write_to_gdb_log("quit");
                    },
                    Err(gdbmi::ExecuteError::Busy) => {
                        self.write_to_gdb_log("GDB is running!");
                    },
                    //Err(err) => { panic!("Unknown error {:?}", err) },
                }
            },
        }
    }
    pub fn update_after_event(&mut self, p: ::UpdateParameters) {
        if p.gdb.mi.is_running() {
            if self.last_gdb_state != GDBState::Running {
                self.last_gdb_state = GDBState::Running;
                self.prompt_line.set_prompt(RUNNING_PROMPT.to_owned());
            }
        } else {
            if self.last_gdb_state != GDBState::Stopped {
                self.last_gdb_state = GDBState::Stopped;
                self.prompt_line.set_prompt(STOPPED_PROMPT.to_owned());
            }
        }
    }
}

impl Widget for Console {
    fn space_demand(&self) -> Demand2D {
        let widgets: Vec<&Widget> = vec![self.get_active_log_viewer(), &self.prompt_line];
        self.layout.space_demand(widgets.as_slice())
    }
    fn draw(&self, window: Window, hints: RenderingHints) {
        // We cannot use self.get_active_log_viewer_mut(), because it apparently borrows
        // self mutably in its entirety. TODO: Maybe there is another way?
        let active_log_viewer = match self.active_log {
            ActiveLog::Debug => &self.debug_log,
            ActiveLog::Gdb => &self.gdb_log,
        };
        self.layout.draw(window, &[(active_log_viewer, hints), (&self.prompt_line, hints)])
    }
}
impl Container<::UpdateParametersStruct> for Console {
    fn input(&mut self, input: Input, p: ::UpdateParameters) -> Option<Input> {
        input
            .chain((Key::F(1), || self.toggle_active_log()))
            .chain((Key::Char('\n'), || self.handle_newline(p)))
            .chain(
                EditBehavior::new(&mut self.prompt_line)
                .left_on(Key::Left)
                .right_on(Key::Right)
                .up_on(Key::Up)
                .down_on(Key::Down)
                .delete_forwards_on(Key::Delete)
                .delete_backwards_on(Key::Backspace)
                .go_to_beginning_of_line_on(Key::Home)
                .go_to_end_of_line_on(Key::End)
                .clear_on(Key::Ctrl('c'))
                )
            .chain(
                ScrollBehavior::new(&mut self.prompt_line)
                .to_end_on(Key::Ctrl('r'))
                )
            .chain((Key::Ctrl('c'), || p.gdb.mi.interrupt_execution().expect("interrupted gdb")))
            .chain(
                ScrollBehavior::new(self.get_active_log_viewer_mut())
                .forwards_on(Key::PageDown)
                .backwards_on(Key::PageUp)
                .to_beginning_on(Key::Ctrl('b'))
                .to_end_on(Key::Ctrl('e'))
                )
            .finish()
    }
}
