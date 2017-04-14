use unsegen;
use gdbmi;

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
    Demand,
    SeparatingStyle,
    VerticalLayout,
    Widget,
};
use unsegen::widget::widgets::{
    LogViewer,
    PromptLine,
};

use input::{
    ConsoleEvent,
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

    pub fn add_message(&mut self, msg: String) {
        use std::fmt::Write;
        write!(self.gdb_log.storage, "{}\n", msg).expect("Write message");
    }

    pub fn add_debug_message(&mut self, msg: String) {
        use std::fmt::Write;
        write!(self.debug_log.storage, " -=- {}\n", msg).expect("Write message");
    }

    pub fn toggle_active_log(&mut self) {
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

    pub fn event(&mut self, event: ::input::ConsoleEvent, gdb: &mut gdbmi::GDB) { //TODO more console events

        match event {
            ConsoleEvent::Raw(e) => self.handle_raw_input(e, gdb),
            ConsoleEvent::ToggleLog => self.toggle_active_log(),
        }
    }

    fn handle_raw_input(&mut self, input: unsegen::input::Input, gdb: &mut gdbmi::GDB) { //TODO more console events
        if input.event == Event::Key(Key::Char('\n')) {
            let line = if self.prompt_line.active_line().is_empty() {
                self.prompt_line.previous_line(1).unwrap_or("").to_owned()
            } else {
                self.prompt_line.finish_line().to_owned()
            };
            match line.as_ref() {
                "!stop" => {
                    gdb.interrupt_execution().expect("interrupted gdb");

                    // This does not always seem to unblock gdb, but only hang it
                    //use gdbmi::input::MiCommand;
                    //gdb.execute(&MiCommand::exec_interrupt()).expect("Interrupt ");
                },
                // Gdb commands
                _ => {
                    self.add_message(format!("(gdb) {}", line));
                    match gdb.execute(&gdbmi::input::MiCommand::cli_exec(line)) {
                        Ok(result) => {
                            self.add_debug_message(format!("Result: {:?}", result));
                        },
                        Err(gdbmi::ExecuteError::Quit) => { self.add_message(format!("quit")); },
                        Err(gdbmi::ExecuteError::Busy) => { self.add_message(format!("GDB is running!")); },
                        //Err(err) => { panic!("Unknown error {:?}", err) },
                    }
                },
            }
        } else {
            let _ = input.chain(
                    |i: Input| if let (&Event::Key(Key::Ctrl('c')), true) = (&i.event, self.prompt_line.line.get().is_empty()) {
                        gdb.interrupt_execution().expect("interrupted gdb");
                        None
                    } else {
                        Some(i)
                    }
                    )
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
                .chain(
                    ScrollBehavior::new(self.get_active_log_viewer_mut())
                        .forwards_on(Key::PageDown)
                        .backwards_on(Key::PageUp)
                    );
        }
    }
}

impl Widget for Console {
    fn space_demand(&self) -> (Demand, Demand) {
        let widgets: Vec<&Widget> = vec![self.get_active_log_viewer(), &self.prompt_line];
        self.layout.space_demand(widgets.as_slice())
    }
    fn draw(&mut self, window: Window) {
        // We cannot use self.get_active_log_viewer_mut(), because it apparently borrows
        // self mutably in its entirety. TODO: Maybe there is another way?
        let active_log_viewer = match self.active_log {
            ActiveLog::Debug => &mut self.debug_log,
            ActiveLog::Gdb => &mut self.gdb_log,
        };
        let mut widgets: Vec<&mut Widget> = vec![active_log_viewer, &mut self.prompt_line];
        self.layout.draw(window, &mut widgets)
    }
}
