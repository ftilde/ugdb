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

use ::unicode_segmentation::UnicodeSegmentation;

pub struct LineEdit {
    text: String,
    cursor_pos: usize,
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
        self.cursor_pos = ::std::cmp::min(self.cursor_pos + 1, count_grapheme_clusters(&self.text) as usize);
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    pub fn insert(&mut self, text: &str) {
        self.text = {
            let grapheme_iter = self.text.graphemes(true);
            grapheme_iter.clone().take(self.cursor_pos)
                .chain(Some(text))
                .chain(grapheme_iter.skip(self.cursor_pos))
                .collect()
        };
        self.move_cursor_right();
    }

    fn erase_symbol_at(&mut self, pos: usize) {
        self.text = self.text.graphemes(true).enumerate().filter_map(
                |(i, s)|  if i != pos {
                    Some(s)
                } else {
                    None
                }
            ).collect();
    }

    pub fn remove_symbol(&mut self) { //i.e., "backspace"
        if self.cursor_pos > 0 {
            let to_erase = self.cursor_pos - 1;
            self.erase_symbol_at(to_erase);
            self.move_cursor_left();
        }
    }

    pub fn delete_symbol(&mut self) { //i.e., "del" key
        let to_erase = self.cursor_pos;
        self.erase_symbol_at(to_erase);
    }
}

impl super::Widget for LineEdit {
    fn space_demand(&self) -> (super::Demand, super::Demand) {
        //(super::Demand::Const((count_grapheme_clusters(&self.text) + 1) as u32), super::Demand::Const(1)) //TODO this is not really universal
        (super::Demand::MaxPossible, super::Demand::Const(1)) //TODO this is not really universal
    }
    fn draw(&mut self, mut window: Window) {
        let (maybe_cursor_pos_offset, maybe_after_cursor_offset) = {
            let mut grapheme_indices = self.text.grapheme_indices(true);
            let cursor_cluster = grapheme_indices.nth(self.cursor_pos as usize);
            let next_cluster = grapheme_indices.next();
            (cursor_cluster.map(|c: (usize, &str)| c.0), next_cluster.map(|c: (usize, &str)| c.0))
        };
        let num_graphemes = count_grapheme_clusters(&self.text);
        let right_padding = 2;
        let cursor_start_pos = ::std::cmp::min(0, window.get_width() as i32 - num_graphemes as i32 - right_padding);

        let text_style = TextAttribute::default();
        let cursor_style = TextAttribute::new(None, None, self.cursor_style);
        let mut cursor = Cursor::new(&mut window).position(cursor_start_pos, 0);
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
                    self.insert(&c.to_string());
                },
                super::Key::Backspace => {
                    self.remove_symbol();
                },
                super::Key::Delete => {
                    self.delete_symbol();
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

use std::ops::Range;
pub trait LineStorage {
    fn view<'a>(&'a mut self, range: Range<usize>) -> Box<DoubleEndedIterator<Item=String> + 'a>;
}

use std::io;
use std::io::{BufReader, BufRead, SeekFrom, Read, Seek};
use std::fs::{File};
use std::path::{Path};

pub struct FileLineStorage {
    reader: BufReader<File>,
    line_seek_positions: Vec<u64>,
}
impl FileLineStorage {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = try!{File::open(path.as_ref())};
        Ok(FileLineStorage {
            reader: BufReader::new(file),
            line_seek_positions: Vec::new(),
        })
    }

    fn skip_to_newline(&mut self) -> io::Result<u64> {
        let mut buffer = vec![0];
        let mut num_bytes = 0;
        while buffer[0] != b'\n' {
            try!{self.reader.read_exact(&mut buffer)};
            num_bytes += 1;
        }
        Ok(num_bytes)
    }
    fn get_line_seek_pos(&mut self, index: usize) -> Option<SeekFrom> {
        let mut buffer_pos = 0;
        if index >= self.line_seek_positions.len() {
            self.reader.seek(SeekFrom::Start(*self.line_seek_positions.last().unwrap_or(&0))).expect("seek to last known");
            if let Some(&last) = self.line_seek_positions.last() {
                buffer_pos = last + match self.skip_to_newline() {
                    Ok(n) => n,
                    Err(e) => {
                        if io::ErrorKind::UnexpectedEof == e.kind() {
                            return None;
                        } else {
                            panic!("file read error: {}", e);
                        }
                    },
                }
            }
        }
        while index >= self.line_seek_positions.len() {
            self.line_seek_positions.push(buffer_pos);
            buffer_pos += match self.skip_to_newline() {
                Ok(n) => n,
                Err(e) => {
                    if io::ErrorKind::UnexpectedEof == e.kind() {
                        return None;
                    } else {
                        panic!("file read error: {}", e);
                    }
                },
            }
        }
        Some(SeekFrom::Start(self.line_seek_positions[index]))
    }

    fn get_line(&mut self, index: usize) -> Option<String> {
        self.get_line_seek_pos(index).map(|p| {
            self.reader.seek(p).expect("seek to line pos");
            let mut buffer = Vec::new();
            self.reader.read_until(b'\n', &mut buffer).expect("read from buffer");
            String::from_utf8_lossy(&buffer).into_owned()
        })
    }
}
impl LineStorage for FileLineStorage {
    fn view<'a>(&'a mut self, range: Range<usize>) -> Box<DoubleEndedIterator<Item=String> + 'a> {
        Box::new(FileLineIterator::new(self, range))
    }
}
struct FileLineIterator<'a> {
    storage: &'a mut FileLineStorage,
    range: Range<usize>,
}
impl<'a> FileLineIterator<'a> {
    fn new(storage: &'a mut FileLineStorage, range: Range<usize>) -> Self {
        FileLineIterator {
            storage: storage,
            range: range,
        }
    }
}
impl<'a> Iterator for FileLineIterator<'a> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        if self.range.start < self.range.end {
            let res = self.storage.get_line(self.range.start); //TODO: maybe we want to treat none differently here?
            self.range.start += 1;
            res
        } else {
            None
        }
    }
}

impl<'a> DoubleEndedIterator for FileLineIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.range.start < self.range.end {
            let res = self.storage.get_line(self.range.end - 1); //TODO: maybe we want to treat none differently here?
            self.range.end -= 1;
            res
        } else {
            None
        }
    }
}

pub trait HighLighter {
    fn highlight<'a>(&mut self, line: &'a str) -> Box<Iterator<Item=(TextAttribute, &'a str)> + 'a>;
}

pub struct NoHighLighter;

impl HighLighter for NoHighLighter {
    fn highlight<'a>(&mut self, line: &'a str) -> Box<Iterator<Item=(TextAttribute, &'a str)> + 'a> {
        Box::new(Some((TextAttribute::plain(), line)).into_iter())
    }
}

//use syntect::parsing::SyntaxSet;
use syntect::parsing::syntax_definition::SyntaxDefinition;
use syntect::highlighting;
use syntect::easy::{HighlightFile, HighlightLines};

struct SyntectHighLighter<'a> {
    highlighter: HighlightLines<'a>,
    //_syntax_set: SyntaxSet,
}

impl<'a> SyntectHighLighter<'a> {
    pub fn new(syntax: &SyntaxDefinition, theme: &'a highlighting::Theme) -> Self {
        SyntectHighLighter {
            highlighter: HighlightLines::new(syntax, theme),
        }
    }
}

impl<'b> HighLighter for SyntectHighLighter<'b> {
    fn highlight<'a>(&mut self, line: &'a str) -> Box<Iterator<Item=(TextAttribute, &'a str)> + 'a> {
        Box::new(
            self.highlighter.highlight(line).into_iter().map(|(h, s)| (to_text_attribute(&h), s))
            )
    }
}

pub struct FileViewer<S: LineStorage, H: HighLighter> {
    storage: S,
    highlighter: H,
}

impl<S: LineStorage, H: HighLighter> FileViewer<S, H> {
    pub fn new(storage: S, highlighter: H) -> Self {
        FileViewer {
            storage: storage,
            highlighter: highlighter,
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

impl<S: LineStorage, H: HighLighter> super::Widget for FileViewer<S, H> {
    fn space_demand(&self) -> (super::Demand, super::Demand) {
        (super::Demand::MaxPossible /*TODO?*/, super::Demand::MaxPossible)
    }
    fn draw(&mut self, mut window: super::Window) {
        let height = window.get_height() as usize;
        let mut cursor = Cursor::new(&mut window)
            .position(0, 0)
            .wrapping_direction(WrappingDirection::Down)
            .wrapping_mode(WrappingMode::Wrap);

        for line in self.storage.view(0..height) {
            for (style, region) in  self.highlighter.highlight(&line) {
                cursor.set_text_attribute(style);
                cursor.write(&region);
            }
            cursor.wrap_line();
        }
    }
    fn input(&mut self, _: super::Event) {
        unimplemented!();
    }
}
