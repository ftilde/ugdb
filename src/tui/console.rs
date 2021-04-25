use tui::commands::CommandState;

use unsegen::base::GraphemeCluster;
use unsegen::container::Container;
use unsegen::input::{EditBehavior, Input, Key, ScrollBehavior};
use unsegen::widget::builtin::{LogViewer, PromptLine};
use unsegen::widget::{VLayout, Widget};

use completion::{CmdlineCompleter, Completer, CompletionState};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum GDBState {
    Running,
    Stopped,
}

pub struct Console {
    gdb_log: LogViewer,
    prompt_line: PromptLine,
    last_gdb_state: GDBState,
    command_state: CommandState,
    completion_state: Option<CompletionState>,
}

static STOPPED_PROMPT: &'static str = "(gdb) ";
static RUNNING_PROMPT: &'static str = "(↻↻↻) ";
static SCROLL_PROMPT: &'static str = "(↑↓) ";
static SEARCH_PROMPT: &'static str = "(🔍) ";

impl Console {
    pub fn new() -> Self {
        let mut prompt_line = PromptLine::with_prompt(STOPPED_PROMPT.into());
        prompt_line.set_search_prompt(SEARCH_PROMPT.to_owned());
        prompt_line.set_scroll_prompt(SCROLL_PROMPT.to_owned());
        Console {
            gdb_log: LogViewer::new(),
            prompt_line,
            last_gdb_state: GDBState::Stopped,
            command_state: CommandState::Idle,
            completion_state: None,
        }
    }

    pub fn write_to_gdb_log<S: AsRef<str>>(&mut self, msg: S) {
        use std::fmt::Write;
        write!(self.gdb_log, "{}", msg.as_ref()).expect("Write Message");
    }

    fn handle_newline(&mut self, p: &mut ::Context) {
        let line = if self.prompt_line.active_line().is_empty() {
            self.prompt_line.previous_line(1).unwrap_or("").to_owned()
        } else {
            self.prompt_line.finish_line().to_owned()
        };
        self.write_to_gdb_log(format!("{}{}\n", STOPPED_PROMPT, line));
        self.command_state.handle_input_line(&line, p);
    }
    pub fn update_after_event(&mut self, p: &mut ::Context) {
        if p.gdb.mi.is_running() {
            if self.last_gdb_state != GDBState::Running {
                self.last_gdb_state = GDBState::Running;
                self.prompt_line.set_edit_prompt(RUNNING_PROMPT.to_owned());
            }
        } else {
            if self.last_gdb_state != GDBState::Stopped {
                self.last_gdb_state = GDBState::Stopped;
                self.prompt_line.set_edit_prompt(STOPPED_PROMPT.to_owned());
            }
        }
    }
}

impl Container<::Context> for Console {
    fn input(&mut self, input: Input, p: &mut ::Context) -> Option<Input> {
        let set_completion = |completion_state: &Option<CompletionState>,
                              prompt_line: &mut PromptLine| {
            let completion = completion_state.as_ref().unwrap();
            let (begin, option, after) = completion.current_line_parts();
            prompt_line.set(&format!("{}{}{}", begin, option, after));
            prompt_line
                .set_cursor_pos(begin.len() + option.len())
                .unwrap();
        };
        let after_completion = input
            .chain((&[Key::Ctrl('p'), Key::Char('\t')][..], || {
                if let Some(s) = &mut self.completion_state {
                    s.select_next_option();
                } else {
                    self.completion_state = Some(CmdlineCompleter(p).complete(
                        self.prompt_line.active_line(),
                        self.prompt_line.cursor_pos(),
                    ));
                }
                set_completion(&self.completion_state, &mut self.prompt_line);
            }))
            .chain((Key::Ctrl('n'), || {
                if let Some(s) = &mut self.completion_state {
                    s.select_prev_option();
                } else {
                    self.completion_state = Some(CmdlineCompleter(p).complete(
                        self.prompt_line.active_line(),
                        self.prompt_line.cursor_pos(),
                    ));
                }
                set_completion(&self.completion_state, &mut self.prompt_line);
            }))
            .finish();
        if let Some(input) = after_completion {
            self.completion_state = None;
            input
                .chain((Key::Char('\n'), || self.handle_newline(p)))
                .chain((Key::Ctrl('r'), || self.prompt_line.enter_search()))
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
        } else {
            None
        }
    }
    fn as_widget<'a>(&'a self) -> Box<dyn Widget + 'a> {
        Box::new(
            VLayout::new()
                .separator(GraphemeCluster::try_from('=').unwrap())
                .widget(self.gdb_log.as_widget())
                .widget(self.prompt_line.as_widget()),
        )
    }
}
