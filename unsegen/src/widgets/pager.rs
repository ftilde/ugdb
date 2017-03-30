use super::super::{
    Color,
    Cursor,
    Demand,
    layout_linearly,
    LineIndex,
    LineNumber,
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

// PagerLine ----------------------------------------------------------------------------------------------

pub trait PagerLine {
    fn get_content(&self) -> &str;
}

impl PagerLine for String {
    fn get_content(&self) -> &str {
        self.as_str()
    }
}

// Highlighter --------------------------------------------------------------------------------------------

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

// LineDecorator ------------------------------------------------------------------------------------------

pub trait LineDecorator {
    type Line: PagerLine;
    fn horizontal_space_demand<'a, 'b: 'a>(&'a self, lines: Box<DoubleEndedIterator<Item=(LineIndex, Self::Line)> + 'b>) -> Demand;
    fn decorate(&self, line: &Self::Line, line_index: LineIndex, window: Window);
}

pub struct NoDecorator<L> {
    _dummy: ::std::marker::PhantomData<L>,
}

impl<L> Default for NoDecorator<L> {
    fn default() -> Self {
        NoDecorator {
            _dummy: Default::default(),
        }
    }
}

impl<L: PagerLine> LineDecorator for NoDecorator<L> {
    type Line = L;
    fn horizontal_space_demand<'a, 'b: 'a>(&'a self, _: Box<DoubleEndedIterator<Item=(LineIndex, Self::Line)> + 'b>) -> Demand {
        Demand::exact(0)
    }
    fn decorate(&self, _: &L, _: LineIndex, _: Window) {
    }
}

pub struct LineNumberDecorator<L> {
    _dummy: ::std::marker::PhantomData<L>,
}

impl<L> Default for LineNumberDecorator<L> {
    fn default() -> Self {
        LineNumberDecorator {
            _dummy: Default::default(),
        }
    }
}

impl<L: PagerLine> LineDecorator for LineNumberDecorator<L> {
    type Line = L;
    fn horizontal_space_demand<'a, 'b: 'a>(&'a self, lines: Box<DoubleEndedIterator<Item=(LineIndex, Self::Line)> + 'b>) -> Demand {
        let max_space = lines.last().map(|(i,_)| {
            ::unicode_width::UnicodeWidthStr::width(format!(" {} ", i).as_str())
        }).unwrap_or(0);
        Demand::from_to(0, max_space as u32)
    }
    fn decorate(&self, _: &L, index: LineIndex, mut window: Window) {
        let width = window.get_width() as usize - 2;
        let line_number = LineNumber::from(index);
        let mut cursor = Cursor::new(&mut window).position(0,0);

        use std::fmt::Write;
        let _ = write!(cursor, " {:width$} ", line_number, width = width);
    }
}


// PagerContent -------------------------------------------------------------------------------------------

pub struct PagerContent<S: LineStorage, H: HighLighter, D: LineDecorator> {
    pub storage: S,
    highlighter: H,
    decorator: D,
}

impl <S> PagerContent<S, NoHighLighter, NoDecorator<S::Line>>
    where S: LineStorage, S::Line: PagerLine {

    pub fn create(storage: S) -> Self {
        PagerContent {
            storage: storage,
            highlighter: NoHighLighter,
            decorator: NoDecorator::default(),
        }
    }
}

impl <S, D> PagerContent<S, NoHighLighter, D>
    where S: LineStorage, S::Line: PagerLine, D: LineDecorator<Line=S::Line> {

    pub fn with_highlighter<HN: HighLighter>(self, highlighter: HN) -> PagerContent<S, HN, D> {
        PagerContent {
            storage: self.storage,
            highlighter: highlighter,
            decorator: self.decorator,
        }
    }
}

impl <S, H> PagerContent<S, H, NoDecorator<S::Line>>
    where S: LineStorage, S::Line: PagerLine, H: HighLighter {

    pub fn with_decorator<DN: LineDecorator<Line=S::Line>>(self, decorator: DN) -> PagerContent<S, H, DN> {
        PagerContent {
            storage: self.storage,
            highlighter: self.highlighter,
            decorator: decorator,
        }
    }
}

// Pager --------------------------------------------------------------------------------------------------

pub struct Pager<S, H = NoHighLighter, D = NoDecorator<<S as LineStorage>::Line>>
    where S: LineStorage, D: LineDecorator , H: HighLighter {

    pub content: Option<PagerContent<S,H,D>>,
    current_line: LineIndex,
}

impl<S, H, D> Pager<S, H, D>
    where S: LineStorage, S::Line: PagerLine, D: LineDecorator<Line=S::Line>, H: HighLighter {

    pub fn new() -> Self {
        Pager {
            content: None,
            current_line: 0.into(),
        }
    }

    pub fn load(&mut self, content: PagerContent<S, H, D>) {
        self.content = Some(content);
    }
    fn line_exists<L: Into<LineIndex>>(&mut self, line: L) -> bool {
        let line: LineIndex = line.into();
        if let Some(ref mut content) = self.content {
            content.storage.view(line..(line+1)).next().is_some()
        } else {
            false
        }
    }
    pub fn go_to_line<L: Into<LineIndex>>(&mut self, line: L) -> Result<(), ()> {
        let line: LineIndex = line.into();
        if self.line_exists(line) {
            self.current_line = line;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn current_line(&self) -> LineIndex {
        self.current_line
    }
}

impl<S, H, D> Widget for Pager<S, H, D>
    where S: LineStorage, S::Line: PagerLine, H: HighLighter, D: LineDecorator<Line=S::Line> {

    fn space_demand(&self) -> (Demand, Demand) {
        (Demand::at_least(1), Demand::at_least(1))
    }
    fn draw(&mut self, window: Window) {
        if let Some(ref mut content) = self.content {
            let height = window.get_height() as usize;
            // The highlighter might need a minimum number of lines to figure out the syntax:
            // TODO: make this configurable?
            let min_highlight_context = 40;
            let num_adjacent_lines_to_load = max(height, min_highlight_context/2);
            let current_line: usize = self.current_line.into();
            let min_line = current_line.checked_sub(num_adjacent_lines_to_load).unwrap_or(0);
            let max_line = current_line + num_adjacent_lines_to_load;


            // Split window
            let decorator_demand = content.decorator.horizontal_space_demand(content.storage.view(min_line..max_line));
            let split_pos = layout_linearly(window.get_width(), 0, &[decorator_demand, Demand::at_least(1)])[0];

            let (mut decoration_window, mut content_window) = window.split_h(split_pos); //TODO: make splitting work for zero width windows!

            // Fill background with correct color
            let (style, _) = content.highlighter.highlight(" ").next().expect("exactly one formatted space");
            content_window.set_default_format(style);
            content_window.fill(' ');

            let mut cursor = Cursor::new(&mut content_window)
                .position(0, 0)
                .wrapping_direction(WrappingDirection::Down)
                .wrapping_mode(WrappingMode::Wrap);

            let num_line_wraps_until_current_line = {
                content.storage
                    .view(min_line..current_line)
                    .map(|(_,line)| {
                        cursor.num_expected_wraps(line.get_content()) + 1
                    })
                    .sum::<u32>()
            };
            let num_line_wraps_from_current_line = {
                content.storage
                    .view(current_line..max_line)
                    .map(|(_,line)| {
                        cursor.num_expected_wraps(line.get_content()) + 1
                    })
                    .sum::<u32>()
            };

            let centered_current_line_start_pos = (height/2) as i32;
            let best_current_line_pos_for_bottom = max(centered_current_line_start_pos, height as i32 - num_line_wraps_from_current_line as i32);
            let required_start_pos = min(0, best_current_line_pos_for_bottom as i32 - num_line_wraps_until_current_line as i32);

            cursor.set_position(0, required_start_pos);

            for (line_index, line) in content.storage.view(min_line..max_line) {
                let base_style = if line_index == self.current_line {
                    TextAttribute::new(None, None, Style::new().invert().bold()).or(&style)
                } else {
                    TextAttribute::default()
                };

                let (_, start_y) = cursor.get_position();
                for (style, region) in content.highlighter.highlight(line.get_content()) {
                    cursor.set_text_attribute(base_style.or(&style));
                    cursor.write(&region);
                }
                cursor.set_text_attribute(base_style);
                cursor.fill_and_wrap_line();
                let (_, end_y) = cursor.get_position();

                let range_start_y = min(max(start_y, 0) as u32, height as u32);
                let range_end_y = min(max(end_y, 0) as u32, height as u32);
                content.decorator.decorate(&line, line_index, decoration_window.create_subwindow(.., range_start_y..range_end_y));
                //decoration_window.create_subwindow(.., range_start_y..range_end_y).fill('X');
            }
        }
    }
}
impl<S, H, D> Scrollable for Pager<S, H, D>
    where S: LineStorage, S::Line: PagerLine, H: HighLighter, D: LineDecorator<Line=S::Line> {

    fn scroll_backwards(&mut self) {
        if self.current_line > 0.into() {
            self.current_line -= 1;
        }
    }
    fn scroll_forwards(&mut self) {
        let new_line = self.current_line + 1;
        let _ = self.go_to_line(new_line);
    }
}
