use super::{
    Window,
    WrappingDirection,
    WrappingMode,
    Cursor,
    TextAttribute,
    Color,
};

// TextLine --------------------------------------------------------------------------------------

pub struct TextLine {
    text: String,
    cursor_pos: u32,
}

impl TextLine {
    pub fn new(text: String) -> Self {
        TextLine {
            text: text,
            cursor_pos: 0,
        }
    }

    pub fn get(&mut self) -> &str {
        &self.text
    }

    /*
    pub fn set(&mut self, text: String) {
        self.text = text
    }
    */

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_pos = 0;
    }

    pub fn move_cursor_right(&mut self) {
        self.cursor_pos += 1; //TODO bounds check
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor_pos = ::std::cmp::max(0, self.cursor_pos - 1);
    }
}

impl super::Widget for TextLine {
    fn space_demand(&self) -> (super::Demand, super::Demand) {
        (super::Demand::Const((self.text.len() + 1) as u32), super::Demand::Const(1)) //TODO this is not really universal
    }
    fn draw(&self, mut window: Window) {
        let (maybe_cursor_pos_offset, maybe_after_cursor_offset) = {
            use ::unicode_segmentation::UnicodeSegmentation;
            let mut grapheme_indices = self.text.grapheme_indices(true);
            let cursor_cluster = grapheme_indices.nth(self.cursor_pos as usize);
            let next_cluster = grapheme_indices.next();
            (cursor_cluster.map(|c: (usize, &str)| c.0), next_cluster.map(|c: (usize, &str)| c.0))
        };
        let text_style = TextAttribute::new(None, None, None);
        let cursor_style = TextAttribute::new(None, Some(Color::green()), None).or(&text_style);
        let mut cursor = Cursor::new(&mut window);
        if let Some(cursor_pos_offset) = maybe_cursor_pos_offset {
            let (until_cursor, from_cursor) = self.text.split_at(cursor_pos_offset);
            cursor.set_text_attribute(text_style);
            cursor.write(until_cursor);
            if let Some(after_cursor_offset) = maybe_after_cursor_offset {
                let (cursor_str, after_cursor) = from_cursor.split_at(after_cursor_offset - cursor_pos_offset);
                cursor.set_text_attribute(cursor_style);
                cursor.write(cursor_str);
                cursor.set_text_attribute(text_style);
                cursor.write(after_cursor);
            } else {
                cursor.set_text_attribute(cursor_style);
                cursor.write(from_cursor);
            }
        } else {
            cursor.set_text_attribute(text_style);
            cursor.write(&self.text);
            cursor.set_text_attribute(cursor_style);
            cursor.write(" ");
        }
    }
    fn input(&mut self, event: super::Event) {
        if let super::Event::Key(key) = event {
            match key {
                super::Key::Char(c) => {
                    self.text.insert(self.cursor_pos as usize, c); //TODO: this might not be in char boundary, use grapheme indices
                    self.move_cursor_right();
                },
                super::Key::Backspace => {
                    if self.cursor_pos > 0 {
                        self.text.remove((self.cursor_pos - 1) as usize);
                        self.move_cursor_left();
                    }
                },
                super::Key::Ctrl('c') => {
                    self.clear();
                },
                super::Key::Left => {
                    self.move_cursor_left();
                },
                super::Key::Right => {
                    self.move_cursor_right();
                },
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
    fn draw(&self, window: Window) {
        let widgets: Vec<&super::Widget> = vec![&self.prompt, &self.line];
        self.layout.draw(window, widgets.into_iter());
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
        (super::Demand::MaxPossible /*TODO?*/, super::Demand::MaxPossible)
    }
    fn draw(&self, mut window: super::Window) {
        let y_start = window.get_height() - 1;
        let mut cursor = Cursor::new(&mut window)
            .position(0, y_start as i32)
            .wrapping_direction(WrappingDirection::Up)
            .wrapping_mode(WrappingMode::Wrap);
        for line in self.lines.iter().rev() {
            cursor.writeln(&line);
        }
    }
    fn input(&mut self, _: super::Event) {
        unimplemented!();
    }
}

impl TextArea {
    pub fn new() -> Self {
        TextArea {
            lines: Vec::new(),
        }
    }

    pub fn active_line_mut(&mut self) -> &mut String {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        return self.lines.last_mut().expect("last line");
    }
}

impl ::std::fmt::Write for TextArea {
    fn write_str(&mut self, s: &str) -> ::std::fmt::Result {
        let mut s = s.to_owned();

        while let Some(newline_offset) = s.find('\n') {
            let line: String = s.drain(..newline_offset).collect();
            s.pop(); //Remove the \n
            self.active_line_mut().push_str(&line);
            self.lines.push(String::new());
        }
        self.active_line_mut().push_str(&s);
        Ok(())
    }
}
