use super::super::{
    Cursor,
    Color,
    Demand,
    LineStorage,
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

use std::cmp::{
    min,
    max,
};

use syntect::parsing::syntax_definition::SyntaxDefinition;
use syntect::highlighting;
use syntect::easy::{HighlightLines};


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

pub struct PagerContent<S: LineStorage, H: HighLighter> {
    pub storage: S,
    highlighter: H,
}

pub struct Pager<S: LineStorage, H: HighLighter> {
    pub content: Option<PagerContent<S,H>>,
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
    fn line_exists(&mut self, line: usize) -> bool {
        if let Some(ref mut content) = self.content {
            content.storage.view(line..(line+1)).next().is_some()
        } else {
            false
        }
    }
    pub fn go_to_line(&mut self, line: usize) -> Result<(), ()> {
        if self.line_exists(line) {
            self.active_line = line;
            Ok(())
        } else {
            Err(())
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

            let num_line_wraps_until_active_line = {
                content.storage
                    .view(min_line..active_line)
                    .map(|(_,line)| {
                        cursor.num_expected_wraps(&line) + 1
                    })
                    .sum::<u32>()
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
                let base_style = if line_number == self.active_line {
                    TextAttribute::new(None, None, Style::new().invert().bold()).or(&style)
                } else {
                    TextAttribute::default()
                };

                for (style, region) in content.highlighter.highlight(&line) {
                    cursor.set_text_attribute(base_style.or(&style));
                    cursor.write(&region);
                }
                cursor.set_text_attribute(base_style);
                cursor.fill_and_wrap_line();
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
        let new_line = self.active_line + 1;
        let _ = self.go_to_line(new_line);
    }
}
