use super::{
    Window,
    WrappingDirection,
    WrappingMode,
    Cursor,
    TextAttribute,
    Style,
    Color,
};

// LineEdit --------------------------------------------------------------------------------------

fn count_grapheme_clusters(text: &str) -> u32 {
    use ::unicode_segmentation::UnicodeSegmentation;
    text.grapheme_indices(true).count() as u32
}
pub struct LineLabel {
    text: String,
}
impl LineLabel {
    pub fn new(text: String) -> Self {
        LineLabel {
            text: text,
        }
    }

    /*
    pub fn set(&mut self, text: String) {
        self.text = text
    }
    */
}

impl super::Widget for LineLabel {
    fn space_demand(&self) -> (super::Demand, super::Demand) {
        (super::Demand::Const(count_grapheme_clusters(&self.text)), super::Demand::Const(1)) //TODO this is not really universal
    }
    fn draw(&mut self, mut window: Window) {
        let mut cursor = Cursor::new(&mut window);
        cursor.write(&self.text);
    }
    fn input(&mut self, _: super::Event) {
        unimplemented!();
    }
}
pub struct LineEdit {
    text: String,
    cursor_pos: u32,
    cursor_style: Style,
}

impl LineEdit {
    pub fn new() -> Self {
        LineEdit {
            text: String::new(),
            cursor_pos: 0,
            cursor_style: Style::new().invert(),
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

impl super::Widget for LineEdit {
    fn space_demand(&self) -> (super::Demand, super::Demand) {
        (super::Demand::Const((count_grapheme_clusters(&self.text) + 1) as u32), super::Demand::Const(1)) //TODO this is not really universal
    }
    fn draw(&mut self, mut window: Window) {
        let (maybe_cursor_pos_offset, maybe_after_cursor_offset) = {
            use ::unicode_segmentation::UnicodeSegmentation;
            let mut grapheme_indices = self.text.grapheme_indices(true);
            let cursor_cluster = grapheme_indices.nth(self.cursor_pos as usize);
            let next_cluster = grapheme_indices.next();
            (cursor_cluster.map(|c: (usize, &str)| c.0), next_cluster.map(|c: (usize, &str)| c.0))
        };
        let text_style = TextAttribute::default();
        let cursor_style = TextAttribute::new(None, None, self.cursor_style);
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
    prompt: LineLabel,
    line: LineEdit,
    history: Vec<String>,
    layout: super::HorizontalLayout,
}

impl PromptLine {

    pub fn with_prompt(prompt: String) -> Self {
        PromptLine {
            prompt: LineLabel::new(prompt),
            line: LineEdit::new(),
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
    fn draw(&mut self, window: Window) {
        let widgets: Vec<&mut super::Widget> = vec![&mut self.prompt, &mut self.line];
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
    fn draw(&mut self, mut window: super::Window) {
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

// FileViewer
use syntect::parsing::SyntaxSet;
use syntect::highlighting;
use syntect::easy::HighlightFile;

pub struct FileViewer<'a> {
    file: HighlightFile<'a>,
    _syntax_set: SyntaxSet,
}

impl<'a> FileViewer<'a> {
    pub fn new(file_path: &str, theme: &'a highlighting::Theme) -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        FileViewer {
            file: HighlightFile::new(file_path, &syntax_set, theme).expect("create highlighter"),
            _syntax_set: syntax_set,
        }
    }
}

fn to_unsegen_color(color: &highlighting::Color) -> Color {
    Color::new(color.r, color.g, color.b)
}
fn to_unsegen_style(style: &highlighting::FontStyle) -> Style {
    Style {
        bold: style.contains(highlighting::FONT_STYLE_BOLD),
        italic: style.contains(highlighting::FONT_STYLE_ITALIC),
        invert: false,
        underline: style.contains(highlighting::FONT_STYLE_UNDERLINE),
    }
}
fn to_text_attribute(style: &highlighting::Style) -> TextAttribute {
    TextAttribute::new(to_unsegen_color(&style.foreground), to_unsegen_color(&style.background), to_unsegen_style(&style.font_style))
}

impl<'a> super::Widget for FileViewer<'a> {
    fn space_demand(&self) -> (super::Demand, super::Demand) {
        (super::Demand::MaxPossible /*TODO?*/, super::Demand::MaxPossible)
    }
    fn draw(&mut self, mut window: super::Window) {
        let mut cursor = Cursor::new(&mut window)
            .position(0, 0)
            .wrapping_direction(WrappingDirection::Down)
            .wrapping_mode(WrappingMode::Wrap);
        cursor.set_text_attribute(TextAttribute::new(None, Color::green(), None));
        let mut line = String::new();
        use std::io::{BufRead, Seek};
        self.file.reader.seek(::std::io::SeekFrom::Start(0)).expect("seek to start of file");
        while self.file.reader.read_line(&mut line).expect("read line") > 0 {

            for (style, region) in  self.file.highlight_lines.highlight(&line) {
                cursor.set_text_attribute(to_text_attribute(&style));
                cursor.write(&region);
            }

            cursor.wrap_line();
            line.clear();
        }
    }
    fn input(&mut self, _: super::Event) {
        unimplemented!();
    }
}
