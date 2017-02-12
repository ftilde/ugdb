use super::super::{
    Cursor,
    Color,
    Demand,
    Style,
    TextAttribute,
    Widget,
    Window,
    WrappingDirection,
    WrappingMode,
};
use super::super::input::{
    Scrollable,
};

use std::ops::Range;
use std::io;
use std::io::{BufReader, BufRead, SeekFrom, Read, Seek};
use std::fs::{File};
use std::path::{Path};

use syntect::parsing::syntax_definition::SyntaxDefinition;
use syntect::highlighting;
use syntect::easy::{HighlightLines};

pub trait LineStorage {
    fn view<'a>(&'a mut self, range: Range<usize>) -> Box<DoubleEndedIterator<Item=String> + 'a>;
}

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

pub struct SyntectHighLighter<'a> {
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

pub struct Pager<S: LineStorage, H: HighLighter> {
    storage: S,
    highlighter: H,
    active_line: usize,
}

impl<S: LineStorage, H: HighLighter> Pager<S, H> {
    pub fn new(storage: S, highlighter: H) -> Self {
        Pager {
            storage: storage,
            highlighter: highlighter,
            active_line: 0,
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

impl<S: LineStorage, H: HighLighter> Widget for Pager<S, H> {
    fn space_demand(&self) -> (Demand, Demand) {
        (Demand::at_least(1), Demand::at_least(1))
    }
    fn draw(&mut self, mut window: Window) {
        let height = window.get_height() as usize;
        {
            let mut cursor = Cursor::new(&mut window)
                .position(0, 0)
                .wrapping_direction(WrappingDirection::Down)
                .wrapping_mode(WrappingMode::Wrap);

            for line in self.storage.view(self.active_line..(self.active_line+height)) {
                for (style, region) in  self.highlighter.highlight(&line) {
                    cursor.set_text_attribute(style);
                    cursor.write(&region);
                }
                cursor.wrap_line();
            }
        }
    }
}
impl<S: LineStorage, H: HighLighter> Scrollable for Pager<S, H> {
    fn scroll_backwards(&mut self) {
        if self.active_line > 0 {
            self.active_line -= 1;
        }
    }
    fn scroll_forwards(&mut self) {
        self.active_line += 1; //TODO: check bounds
    }
}
