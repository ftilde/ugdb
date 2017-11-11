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
    Event,
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

enum ActiveLog {
    Debug,
    Gdb,
}

pub struct Console {
    debug_log: LogViewer,
    gdb_log: LogViewer,
    active_log: ActiveLog,
    prompt_line: PromptLine,
    layout: VerticalLayout,
}

impl Console {
    pub fn new() -> Self {
        Console {
            debug_log: LogViewer::new(),
            gdb_log: LogViewer::new(),
            active_log: ActiveLog::Gdb,
            prompt_line: PromptLine::with_prompt("(gdb) ".into()),
            layout: VerticalLayout::new(SeparatingStyle::Draw(GraphemeCluster::try_from('=').unwrap())),
        }
    }

    pub fn display_log(&mut self, logger: Logger) {
        use std::fmt::Write;
        for (msg_type, msg) in logger.into_messages() {
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
                self.write_to_gdb_log(format!("(gdb) {}\n", line));
                match p.gdb.mi.execute(&gdbmi::input::MiCommand::cli_exec(line)) {
                    Ok(result) => {
                        p.logger.log(LogMsgType::Debug, format!("Result: {:?}", result));
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

    pub fn event(&mut self, input: Input, p: ::UpdateParameters) {
        input
            .chain(|input: Input| {
                match input.event {
                    Event::Key(Key::F(1)) => {
                        self.toggle_active_log();
                        None
                    },
                    Event::Key(Key::Char('\n')) => {
                        self.handle_newline(p);
                        None
                    }
                    _ => Some(input)
                }
            })
            .chain(
                EditBehavior::new(&mut self.prompt_line)
                .left_on(Key::Left)
                .right_on(Key::Right)
                .up_on(Key::Up)
                .down_on(Key::Down)
                .delete_symbol_on(Key::Delete)
                .remove_symbol_on(Key::Backspace)
                .clear_on(Key::Ctrl('c'))
                )
            .chain(|i: Input| {
                   if let Event::Key(Key::Ctrl('c')) = i.event {
                       p.gdb.mi.interrupt_execution().expect("interrupted gdb");
                       None
                   } else {
                       Some(i)
                   }
            })
            .chain(
                ScrollBehavior::new(self.get_active_log_viewer_mut())
                .forwards_on(Key::PageDown)
                .backwards_on(Key::PageUp)
                );
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
