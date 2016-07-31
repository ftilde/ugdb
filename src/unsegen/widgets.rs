
// TextLine --------------------------------------------------------------------------------------

pub struct TextLine {
    text: String
}

impl TextLine {
    pub fn new(text: String) -> Self {
        TextLine {
            text: text,
        }
    }

    pub fn get(&mut self) -> &str {
        &self.text
    }

    pub fn set(&mut self, text: String) {
        self.text = text
    }

    pub fn clear(&mut self) {
        self.text.clear()
    }
}

impl super::Widget for TextLine {
    fn space_demand(&self) -> (super::Demand, super::Demand) {
        (super::Demand::Const(self.text.len() as u32), super::Demand::Const(1)) //TODO?
    }
    fn draw(&self, mut window: super::Window) {
        window.write(0, 0, &self.text, &super::TextAttribute::plain());
    }
    fn input(&mut self, event: super::Event) {
        if let super::Event::Key(key) = event {
            match key {
                super::Key::Char(c) => { self.text.push(c); },
                super::Key::Backspace => { self.text.pop(); },
                super::Key::Ctrl('c') => { self.text.clear(); },
                _ => {},
            }
        }
    }
}

// PromptLine --------------------------------------------------------------------------------------

pub struct PromptLine {
    prompt: TextLine,
    line: TextLine,
    history: Vec<String>,
    layout: super::HorizontalLayout,
}

impl PromptLine {
    pub fn new() -> Self {
        PromptLine::with_prompt(" > ".into())
    }

    pub fn with_prompt(prompt: String) -> Self {
        PromptLine {
            prompt: TextLine::new(prompt),
            line: TextLine::new("".into()),
            history: Vec::new(),
            layout: super::HorizontalLayout::new(super::SeparatingStyle::None),
        }
    }
    pub fn finish_line(&mut self) -> &str {
        self.history.push(self.line.get().to_owned());
        self.line.clear();
        &self.history[self.history.len()-1]
    }
}

impl super::Widget for PromptLine {
    fn space_demand(&self) -> (super::Demand, super::Demand) {
        let widgets: Vec<&super::Widget> = vec![&self.prompt, &self.line];
        self.layout.space_demand(widgets.into_iter())
    }
    fn draw(&self, window: super::Window) {
        let widgets: Vec<&super::Widget> = vec![&self.prompt, &self.line];
        let windows = self.layout.draw(window, widgets.into_iter());
    }
    fn input(&mut self, event: super::Event) {
        self.line.input(event);
    }
}


// TextArea --------------------------------------------------------------------------------------

pub struct TextArea {
    lines: Vec<String>,
} //TODO support incomplete lines

impl super::Widget for TextArea {
    fn space_demand(&self) -> (super::Demand, super::Demand) {
        //return (super::Demand::MaxPossible /*TODO?*/, super::Demand::Const(self.lines.len() as u32));
        return (super::Demand::MaxPossible /*TODO?*/, super::Demand::MaxPossible);
    }
    fn draw(&self, mut window: super::Window) {
        for (i, ref line) in self.lines.iter().rev().enumerate() {
            let y = (window.get_height() as i32) - 1 - (i as i32);
            if y < 0 {
                break;
            }
            window.write(0, y, &line, &super::TextAttribute::plain());
        }
    }
    fn input(&mut self, event: super::Event) {
        unimplemented!();
    }
}

impl TextArea {
    pub fn new() -> Self {
        TextArea {
            lines: Vec::new(),
        }
    }

    pub fn add_line(&mut self, line: String) {
        self.lines.push(line);
    }
}
