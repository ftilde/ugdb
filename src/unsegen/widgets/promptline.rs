use super::super::{
    Demand,
    Event,
    HorizontalLayout,
    SeparatingStyle,
    Widget,
    Window,
};
use super::{
    LineEdit,
    LineLabel,
};

pub struct PromptLine {
    prompt: LineLabel,
    line: LineEdit,
    history: Vec<String>,
    layout: HorizontalLayout,
}

impl PromptLine {

    pub fn with_prompt(prompt: String) -> Self {
        PromptLine {
            prompt: LineLabel::new(prompt),
            line: LineEdit::new(),
            history: Vec::new(),
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
    fn input(&mut self, event: Event) {
        self.line.input(event);
    }
}
