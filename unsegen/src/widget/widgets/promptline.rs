use super::super::{
    Demand2D,
    HorizontalLayout,
    RenderingHints,
    SeparatingStyle,
    Widget,
};
use base::{
    Window,
};
use super::{
    LineEdit,
    LineLabel,
};
use input::{
    Editable,
    Navigatable,
    Writable,
    Scrollable,
    OperationResult,
};

pub struct PromptLine {
    prompt: LineLabel,
    pub line: LineEdit,
    history: Vec<String>,
    history_scroll_position: Option<ScrollBackState>,
    layout: HorizontalLayout,
}

struct ScrollBackState {
    active_line: String,
    pos: usize,
}

impl ScrollBackState {
    fn new(active_line: String, pos: usize) -> Self {
        ScrollBackState {
            active_line: active_line,
            pos: pos,
        }
    }
}

impl PromptLine {
    pub fn with_prompt(prompt: String) -> Self {
        PromptLine {
            prompt: LineLabel::new(prompt),
            line: LineEdit::new(),
            history: Vec::new(),
            history_scroll_position: None, //invariant: let Some(pos) = history_scroll_pos => pos < history.len()
            layout: HorizontalLayout::new(SeparatingStyle::None),
        }
    }

    pub fn previous_line(&self, n: usize) -> Option<&str> {
        self.history.get(self.history.len().checked_sub(n).unwrap_or(0)).map(String::as_str)
    }

    pub fn active_line(&self) -> &str {
        self.line.get()
    }

    pub fn finish_line(&mut self) -> &str {
        if self.history.is_empty() || self.line.get() != self.history.last().expect("history is not empty").as_str() {
            self.history.push(self.line.get().to_owned());
        }
        let _ = self.line.clear();
        &self.history[self.history.len()-1]
    }

    fn sync_line_to_history_scroll_position(&mut self) {
        if let Some(ref state) = self.history_scroll_position {
            // history[pos] is always valid because of the invariant on history_scroll_pos
            self.line.set(&self.history[state.pos]);
        }
    }
}

impl Widget for PromptLine {
    fn space_demand(&self) -> Demand2D {
        let widgets: Vec<&Widget> = vec![&self.prompt, &self.line];
        self.layout.space_demand(widgets.as_slice())
    }
    fn draw(&self, window: Window, hints: RenderingHints) {
        let widgets: Vec<(&Widget, RenderingHints)> = vec![(&self.prompt, hints), (&self.line, hints)];
        self.layout.draw(window, widgets.as_slice());
    }
}

impl Scrollable for PromptLine {
    fn scroll_forwards(&mut self) -> OperationResult {
        let op_result;
        self.history_scroll_position = if let Some(mut state) = self.history_scroll_position.take() {
            op_result = Ok(());
            if state.pos+1 < self.history.len() {
                state.pos += 1;
                Some(state)
            } else {
                self.line.set(&state.active_line);
                None
            }
        } else {
            op_result = Err(());
            None
        };
        self.sync_line_to_history_scroll_position();
        op_result
    }
    fn scroll_backwards(&mut self) -> OperationResult {
        self.history_scroll_position = if let Some(mut state) = self.history_scroll_position.take() {
            if state.pos > 0 {
                state.pos -= 1;
            }
            Some(state)
        } else {
            if self.history.len() > 0 {
                Some(ScrollBackState::new(self.line.get().to_owned(), self.history.len() - 1))
            } else {
                None
            }
        };
        self.sync_line_to_history_scroll_position();
        if self.history_scroll_position .is_some() {
            Ok(())
        } else {
            Err(())
        }
    }
}
impl Navigatable for PromptLine {
    fn move_up(&mut self) -> OperationResult {
        self.scroll_backwards()
    }
    fn move_down(&mut self) -> OperationResult {
        self.scroll_forwards()
    }
    fn move_left(&mut self) -> OperationResult {
        self.line.move_left()
    }
    fn move_right(&mut self) -> OperationResult {
        self.line.move_right()
    }
}

impl Writable for PromptLine {
    fn write(&mut self, c: char) -> OperationResult {
        let op_res = self.line.write(c);
        if op_res.is_ok() {
            self.history_scroll_position = None;
        }
        op_res
    }
}

impl Editable for PromptLine {
    fn delete_symbol(&mut self) -> OperationResult {
        let op_res = self.line.delete_symbol();
        if op_res.is_ok() {
            self.history_scroll_position = None;
        }
        op_res
    }
    fn remove_symbol(&mut self) -> OperationResult {
        let op_res = self.line.remove_symbol();
        if op_res.is_ok() {
            self.history_scroll_position = None;
        }
        op_res
    }
    fn clear(&mut self) -> OperationResult {
        let op_res = self.line.clear();
        if op_res.is_ok() {
            self.history_scroll_position = None;
        }
        op_res
    }
}
