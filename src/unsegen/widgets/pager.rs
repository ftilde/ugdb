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
use std::cmp::{
    min,
    max,
};
use std::io;
use std::io::{
    BufReader,
    BufRead,
    SeekFrom,
    Seek,
};
use std::fs::{File};
use std::path::{Path};

use syntect::parsing::syntax_definition::SyntaxDefinition;
use syntect::highlighting;
use syntect::easy::{HighlightLines};

pub trait LineStorage {
    fn view<'a>(&'a mut self, range: Range<usize>) -> Box<Iterator<Item=(usize, String)> + 'a>;
}

pub struct FileLineStorage {
    reader: BufReader<File>,
    line_seek_positions: Vec<usize>,
}
impl FileLineStorage {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = try!{File::open(path.as_ref())};
        Ok(FileLineStorage {
            reader: BufReader::new(file),
            line_seek_positions: vec![0],
        })
    }

    fn get_line(&mut self, index: usize) -> Option<String> {
        let mut buffer = Vec::new();

        loop {
            let current_max_index: usize = self.line_seek_positions[min(index, self.line_seek_positions.len()-1)];
            self.reader.seek(SeekFrom::Start(current_max_index as u64)).expect("seek to line pos");
            let n_bytes = self.reader.read_until(b'\n', &mut buffer).expect("read line");
            if n_bytes == 0 { //We reached EOF
                return None;
            }
            if index < self.line_seek_positions.len() { //We found the desired line
                let mut string = String::from_utf8_lossy(&buffer).into_owned();
                if string.as_str().bytes().last().unwrap_or(b'_') == b'\n' {
                    string.pop();
                }
                return Some(string);
            }
            self.line_seek_positions.push(current_max_index + n_bytes);
        }
    }
}
impl LineStorage for FileLineStorage {
    fn view<'a>(&'a mut self, range: Range<usize>) -> Box<Iterator<Item=(usize, String)> + 'a> {
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
    type Item = (usize, String);
    fn next(&mut self) -> Option<Self::Item> {
        if self.range.start < self.range.end {
            let item_index = self.range.start;
            self.range.start += 1;
            if let Some(line) = self.storage.get_line(item_index) {
                Some((item_index, line)) //TODO: maybe we want to treat none differently here?
            } else {
                None
            }
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

struct PagerContent<S: LineStorage, H: HighLighter> {
    storage: S,
    highlighter: H,
}

pub struct Pager<S: LineStorage, H: HighLighter> {
    content: Option<PagerContent<S,H>>,
    active_line: usize,
}

impl<S: LineStorage, H: HighLighter> Pager<S, H> {
    pub fn new() -> Self {
        Pager {
            content: None,
            active_line: 0,
        }
    }

    pub fn load(&mut self, storage: S, highlighter: H) {
        self.content = Some(PagerContent {
            storage: storage,
            highlighter: highlighter,
        });
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
        if let Some(ref mut content) = self.content {
            // Fill background with correct color
            let (style, _) = content.highlighter.highlight(" ").next().expect("exactly one formatted space");
            window.set_default_format(style);
            window.fill(' ');

            let height = window.get_height() as usize;
            // The highlighter might need a minimum number of lines to figure out the syntax:
            // TODO: make this configurable?
            let min_highlight_context = 40;
            let num_adjacent_lines_to_load = max(height, min_highlight_context/2);
            let min_line = self.active_line.checked_sub(num_adjacent_lines_to_load).unwrap_or(0);
            let active_line = self.active_line;
            let max_line = self.active_line + num_adjacent_lines_to_load;

            let mut cursor = Cursor::new(&mut window)
                .position(0, 0)
                .wrapping_direction(WrappingDirection::Down)
                .wrapping_mode(WrappingMode::Wrap);

            let num_line_wraps_until_active_line: u32 = {
                content.storage
                    .view(min_line..active_line)
                    .map(|(_,line)| {
                        cursor.num_expected_wraps(&line) + 1
                    })
                    .sum()
            };
            let num_line_wraps_from_active_line = {
                content.storage
                    .view(active_line..max_line)
                    .map(|(_,line)| {
                        cursor.num_expected_wraps(&line) + 1
                    })
                    .sum::<u32>()
            };

            let centered_active_line_start_pos = (height/2) as i32;
            let best_active_line_pos_for_bottom = max(centered_active_line_start_pos, height as i32 - num_line_wraps_from_active_line as i32);
            let required_start_pos = min(0, best_active_line_pos_for_bottom as i32 - num_line_wraps_until_active_line as i32);

            cursor.set_position(0, required_start_pos);

            for (line_number, line) in content.storage.view(min_line..max_line) {
                for (mut style, region) in content.highlighter.highlight(&line) {
                    if line_number == self.active_line {
                        style = TextAttribute::new(None, None, Style::new().invert().bold()).or(&style);
                    }
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
        if let Some(ref mut content) = self.content {
            if content.storage.view((self.active_line+1)..(self.active_line+2)).next().is_some() {
                self.active_line += 1;
            }
        }
    }
}
