use tui::commands::CommandState;

use unsegen::base::{GraphemeCluster, Window};
use unsegen::container::Container;
use unsegen::input::{EditBehavior, Input, Key, ScrollBehavior};
use unsegen::widget::builtin::{LogViewer, PromptLine};
use unsegen::widget::{Demand2D, RenderingHints, SeparatingStyle, VerticalLayout, Widget};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum GDBState {
    Running,
    Stopped,
}

pub struct Console {
    gdb_log: LogViewer,
    prompt_line: PromptLine,
    layout: VerticalLayout,
    last_gdb_state: GDBState,
    command_state: CommandState,
}

static STOPPED_PROMPT: &'static str = "(gdb) ";
static RUNNING_PROMPT: &'static str = "(↻↻↻) ";

impl Console {
    pub fn new() -> Self {
        Console {
            gdb_log: LogViewer::new(),
            prompt_line: PromptLine::with_prompt(STOPPED_PROMPT.into()),
            layout: VerticalLayout::new(SeparatingStyle::Draw(
                GraphemeCluster::try_from('=').unwrap(),
            )),
            last_gdb_state: GDBState::Stopped,
            command_state: CommandState::Idle,
        }
    }

    pub fn display_messages(&mut self, sink: &mut ::MessageSink) {
        use std::fmt::Write;
        for msg in sink.drain_messages() {
            writeln!(self.gdb_log, "{}", msg).expect("Write Message");
        }
    }

    pub fn write_to_gdb_log<S: AsRef<str>>(&mut self, msg: S) {
        use std::fmt::Write;
        write!(self.gdb_log, "{}", msg.as_ref()).expect("Write Message");
    }

    fn handle_newline(&mut self, p: ::UpdateParameters) {
        let line = if self.prompt_line.active_line().is_empty() {
            self.prompt_line.previous_line(1).unwrap_or("").to_owned()
        } else {
            self.prompt_line.finish_line().to_owned()
        };
        self.write_to_gdb_log(format!("{}{}\n", STOPPED_PROMPT, line));
        self.command_state.handle_input_line(&line, p);
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
        let widgets: Vec<&dyn Widget> = vec![&self.gdb_log, &self.prompt_line];
        self.layout.space_demand(widgets.as_slice())
    }
    fn draw(&self, window: Window, hints: RenderingHints) {
        self.layout.draw(
            window,
            &[(&self.gdb_log, hints), (&self.prompt_line, hints)],
        )
    }
}
impl Container<::UpdateParametersStruct> for Console {
    fn input(&mut self, input: Input, p: ::UpdateParameters) -> Option<Input> {
        input
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
                    .clear_on(Key::Ctrl('c')),
            )
            .chain(ScrollBehavior::new(&mut self.prompt_line).to_end_on(Key::Ctrl('r')))
            .chain((Key::Ctrl('c'), || {
                p.gdb.mi.interrupt_execution().expect("interrupted gdb")
            }))
            .chain(
                ScrollBehavior::new(&mut self.gdb_log)
                    .forwards_on(Key::PageDown)
                    .backwards_on(Key::PageUp)
                    .to_beginning_on(Key::Ctrl('b'))
                    .to_end_on(Key::Ctrl('e')),
            )
            .finish()
    }
}
