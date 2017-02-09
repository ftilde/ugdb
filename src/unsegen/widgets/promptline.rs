use super::super::{
    Demand,
    Event,
    HorizontalLayout,
    Scrollable,
    SeparatingStyle,
    Widget,
    Window,
};
use super::{
    LineEdit,
    LineLabel,
};
use super::super::input::{
    Editable,
    Navigatable,
    Writable,
};

pub struct PromptLine {
    prompt: LineLabel,
    pub line: LineEdit,
    history: Vec<String>,
    history_scroll_position: Option<usize>,
    layout: HorizontalLayout,
}

impl PromptLine {
    pub fn with_prompt(prompt: String) -> Self {
        PromptLine {
            prompt: LineLabel::new(prompt),
            line: LineEdit::new(),
            history: Vec::new(),
            history_scroll_position: None,
            layout: HorizontalLayout::new(SeparatingStyle::None),
        }
    }
    pub fn finish_line(&mut self) -> &str {
        self.history.push(self.line.get().to_owned());
        self.line.clear();
        &self.history[self.history.len()-1]
    }
}

impl Widget for PromptLine {
    fn space_demand(&self) -> (Demand, Demand) {
        let widgets: Vec<&Widget> = vec![&self.prompt, &self.line];
        self.layout.space_demand(widgets.as_slice())
    }
    fn draw(&mut self, window: Window) {
        let mut widgets: Vec<&mut Widget> = vec![&mut self.prompt, &mut self.line];
        self.layout.draw(window, widgets.as_mut_slice());
    }
}

impl Scrollable for PromptLine {
    fn scroll_forwards(&mut self) {
        self.history_scroll_position = if let Some(pos) = self.history_scroll_position {
            if pos+1 < self.history.len() {
                Some(pos + 1)
            } else {
                Some(pos)
            }
        } else {
            self.history_scroll_position
        };
    }
    fn scroll_backwards(&mut self) {
        self.history_scroll_position = if let Some(pos) = self.history_scroll_position {
            if pos > 0 {
                Some(pos - 1)
            } else {
                Some(pos)
            }
        } else {
            Some(self.history.len() - 1)
        };
    }
}
impl Navigatable for PromptLine {
    fn move_up(&mut self) {
        self.scroll_backwards();
    }
    fn move_down(&mut self) {
        self.scroll_forwards();
    }
    fn move_left(&mut self) {
        self.line.move_cursor_left();
    }
    fn move_right(&mut self) {
        self.line.move_cursor_right();
    }
}

impl Writable for PromptLine {
    fn write(&mut self, c: char) {
        self.line.insert(&c.to_string());
        self.history_scroll_position = None;
    }
}

impl Editable for PromptLine {
    fn delete_symbol(&mut self) {
        self.line.delete_symbol();
    }
    fn remove_symbol(&mut self) {
        self.line.remove_symbol();
    }
    fn clear(&mut self) {
        self.line.clear();
    }
}
