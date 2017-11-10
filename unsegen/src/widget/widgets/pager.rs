use super::super::{
    Demand,
    Demand2D,
    layout_linearly,
    LineIndex,
    LineNumber,
    LineStorage,
    RenderingHints,
    Widget,
};
use base::{
    Color,
    Cursor,
    GraphemeCluster,
    ModifyMode,
    StyleModifier,
    TextFormatModifier,
    Window,
    WrappingMode,
};
use input::{
    Scrollable,
    OperationResult,
};

use std::cmp::{
    min,
    max,
};

use syntect::parsing::{
    ScopeStack,
    ParseState,
    SyntaxDefinition,
};
use syntect::highlighting;
use syntect::highlighting::{
    Theme,
};

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

pub trait Highlighter {
    type Instance: HighlightingInstance;
    fn create_instance(&self) -> Self::Instance;
}

pub trait HighlightingInstance {
    fn highlight<'a>(&mut self, line: &'a str) -> Box<Iterator<Item=(StyleModifier, &'a str)> + 'a>;
    fn default_style(&self) -> StyleModifier;
}

pub struct NoHighlighter;

impl Highlighter for NoHighlighter {
    type Instance = NoopHighlightingInstance;
    fn create_instance(&self) -> Self::Instance {
        NoopHighlightingInstance
    }
}

pub struct NoopHighlightingInstance;

impl HighlightingInstance for NoopHighlightingInstance {
    fn highlight<'a>(&mut self, line: &'a str) -> Box<Iterator<Item=(StyleModifier, &'a str)> + 'a> {
        Box::new(Some((StyleModifier::none(), line)).into_iter())
    }
    fn default_style(&self) -> StyleModifier {
        StyleModifier::none()
    }
}

pub struct SyntectHighlighter<'a> {
    base_state: ParseState,
    theme: &'a Theme,
}

impl<'a> SyntectHighlighter<'a> {
    pub fn new(syntax: &SyntaxDefinition, theme: &'a highlighting::Theme) -> Self {
        SyntectHighlighter {
            base_state: ParseState::new(syntax),
            theme: theme,
        }
    }
}

impl<'a> Highlighter for SyntectHighlighter<'a> {
    type Instance = SyntectHighlightingInstance<'a>;
    fn create_instance(&self) -> Self::Instance {
        SyntectHighlightingInstance::new(self.base_state.clone(), self.theme)
    }
}

pub struct SyntectHighlightingInstance<'a> {
    highlighter: highlighting::Highlighter<'a>,
    parse_state: ParseState,
    highlight_state: highlighting::HighlightState,
}

impl<'a> SyntectHighlightingInstance<'a> {
    fn new(base_state: ParseState, theme: &'a highlighting::Theme) -> Self {
        let highlighter = highlighting::Highlighter::new(theme);
        let hstate = highlighting::HighlightState::new(&highlighter, ScopeStack::new());
        SyntectHighlightingInstance {
            highlighter: highlighter,
            parse_state: base_state,
            highlight_state: hstate,
        }
    }
}

impl<'b> HighlightingInstance for SyntectHighlightingInstance<'b> {
    fn highlight<'a>(&mut self, line: &'a str) -> Box<Iterator<Item=(StyleModifier, &'a str)> + 'a> {
        let ops = self.parse_state.parse_line(line);
        let iter: Vec<(highlighting::Style, &'a str)> = highlighting::HighlightIterator::new(&mut self.highlight_state, &ops[..], line, &self.highlighter).collect();
        Box::new(iter.into_iter().map(|(style, line)| (to_unsegen_style_modifier(&style), line)))
    }
    fn default_style(&self) -> StyleModifier {
        to_unsegen_style_modifier(&self.highlighter.get_default())
    }
}

fn to_unsegen_color(color: &highlighting::Color) -> Color {
    Color::Rgb{r: color.r, g: color.g, b: color.b}
}
fn to_unsegen_text_format(style: &highlighting::FontStyle) -> TextFormatModifier {
    TextFormatModifier {
        bold: style.contains(highlighting::FontStyle::BOLD).into(),
        italic: style.contains(highlighting::FontStyle::ITALIC).into(),
        invert: ModifyMode::LeaveUnchanged,
        underline: style.contains(highlighting::FontStyle::UNDERLINE).into(),
    }
}
fn to_unsegen_style_modifier(style: &highlighting::Style) -> StyleModifier {
    StyleModifier::new().fg_color(to_unsegen_color(&style.foreground)).bg_color(to_unsegen_color(&style.background)).format(to_unsegen_text_format(&style.font_style))
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
        write!(cursor, " {:width$} ", line_number, width = width).unwrap();
    }
}


// PagerContent -------------------------------------------------------------------------------------------

pub struct PagerContent<S: LineStorage, H: Highlighter, D: LineDecorator> {
    pub storage: S,
    highlighter: H,
    pub decorator: D,
}

impl <S> PagerContent<S, NoHighlighter, NoDecorator<S::Line>>
    where S: LineStorage, S::Line: PagerLine {

    pub fn create(storage: S) -> Self {
        PagerContent {
            storage: storage,
            highlighter: NoHighlighter,
            decorator: NoDecorator::default(),
        }
    }
}

impl <S, D> PagerContent<S, NoHighlighter, D>
    where S: LineStorage, S::Line: PagerLine, D: LineDecorator<Line=S::Line> {

    pub fn with_highlighter<HN: Highlighter>(self, highlighter: HN) -> PagerContent<S, HN, D> {
        PagerContent {
            storage: self.storage,
            highlighter: highlighter,
            decorator: self.decorator,
        }
    }
}

impl <S, H> PagerContent<S, H, NoDecorator<S::Line>>
    where S: LineStorage, S::Line: PagerLine, H: Highlighter {

    pub fn with_decorator<DN: LineDecorator<Line=S::Line>>(self, decorator: DN) -> PagerContent<S, H, DN> {
        PagerContent {
            storage: self.storage,
            highlighter: self.highlighter,
            decorator: decorator,
        }
    }
}

#[derive(Debug)]
pub enum PagerError {
    NoLineWithIndex(LineIndex),
    NoLineWithPredicate,
    NoContent
}

// Pager --------------------------------------------------------------------------------------------------

pub struct Pager<S, H = NoHighlighter, D = NoDecorator<<S as LineStorage>::Line>>
    where S: LineStorage, D: LineDecorator , H: Highlighter {

    pub content: Option<PagerContent<S,H,D>>,
    current_line: LineIndex,
}

impl<S, H, D> Pager<S, H, D>
    where S: LineStorage, S::Line: PagerLine, D: LineDecorator<Line=S::Line>, H: Highlighter {

    pub fn new() -> Self {
        Pager {
            content: None,
            current_line: LineIndex(0),
        }
    }

    pub fn load(&mut self, content: PagerContent<S, H, D>) {
        self.content = Some(content);

        // Go back to last available line
        let mut new_current_line = self.current_line;
        while !self.line_exists(new_current_line) {
            new_current_line -= 1;
        }
        self.current_line = new_current_line;
    }
    fn line_exists<L: Into<LineIndex>>(&mut self, line: L) -> bool {
        let line: LineIndex = line.into();
        if let Some(ref mut content) = self.content {
            content.storage.view(line..(line+1)).next().is_some()
        } else {
            false
        }
    }

    pub fn go_to_line<L: Into<LineIndex>>(&mut self, line: L) -> Result<(), PagerError> {
        let line: LineIndex = line.into();
        if self.line_exists(line) {
            self.current_line = line;
            Ok(())
        } else {
            Err(PagerError::NoLineWithIndex(line))
        }
    }

    pub fn go_to_line_if<F: Fn(LineIndex, &S::Line) -> bool>(&mut self, predicate: F) -> Result<(), PagerError> {
        let line = if let Some(ref mut content) = self.content {
            content.storage.view(LineIndex(0)..).find(|&(index, ref line)| predicate(index.into(), line)).ok_or(PagerError::NoLineWithPredicate)
        } else {
            Err(PagerError::NoContent)
        };
        line.and_then(|(index, _)| self.go_to_line(index))
    }

    pub fn current_line_index(&self) -> LineIndex {
        self.current_line
    }

    pub fn current_line(&self) -> Option<S::Line> {
        if let Some(ref content) = self.content {
            content.storage.view_line(self.current_line_index())
        } else {
            None
        }
    }
}

impl<S, H, D> Widget for Pager<S, H, D>
    where S: LineStorage, S::Line: PagerLine, H: Highlighter, D: LineDecorator<Line=S::Line> {

    fn space_demand(&self) -> Demand2D {
        Demand2D {
            width: Demand::at_least(1),
            height: Demand::at_least(1)
        }
    }
    fn draw(&mut self, window: Window, _: RenderingHints) {
        if let Some(ref mut content) = self.content {
            let mut highlighter = content.highlighter.create_instance();
            let height = window.get_height() as usize;
            // The highlighter might need a minimum number of lines to figure out the syntax:
            // TODO: make this configurable?
            let min_highlight_context = 40;
            let num_adjacent_lines_to_load = max(height, min_highlight_context/2);
            let min_line = self.current_line.checked_sub(num_adjacent_lines_to_load).unwrap_or(LineIndex(0));
            let max_line = self.current_line + num_adjacent_lines_to_load;


            // Split window
            let decorator_demand = content.decorator.horizontal_space_demand(content.storage.view(min_line..max_line));
            let split_pos = layout_linearly(window.get_width(), 0, &[decorator_demand, Demand::at_least(1)])[0];

            let (mut decoration_window, mut content_window) = window.split_h(split_pos).expect("valid split pos");

            // Fill background with correct color
            let bg_style = highlighter.default_style();
            content_window.set_default_style(bg_style.apply_to_default());
            content_window.fill(GraphemeCluster::space());

            let mut cursor = Cursor::new(&mut content_window)
                .position(0, 0)
                .wrapping_mode(WrappingMode::Wrap);

            let num_line_wraps_until_current_line = {
                content.storage
                    .view(min_line..self.current_line)
                    .map(|(_,line)| {
                        cursor.num_expected_wraps(line.get_content()) + 1
                    })
                    .sum::<u32>()
            };
            let num_line_wraps_from_current_line = {
                content.storage
                    .view(self.current_line..max_line)
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
                    StyleModifier::new().invert(ModifyMode::Toggle).bold(true)
                } else {
                    StyleModifier::none()
                };

                let (_, start_y) = cursor.get_position();
                for (style, region) in highlighter.highlight(line.get_content()) {
                    cursor.set_style_modifier(style.on_top_of(&base_style));
                    cursor.write(&region);
                }
                cursor.set_style_modifier(base_style);
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
    where S: LineStorage, S::Line: PagerLine, H: Highlighter, D: LineDecorator<Line=S::Line> {

    fn scroll_backwards(&mut self) -> OperationResult {
        if self.current_line > LineIndex(0) {
            self.current_line -= 1;
            Ok(())
        } else {
            Err(())
        }
    }
    fn scroll_forwards(&mut self) -> OperationResult {
        let new_line = self.current_line + 1;
        self.go_to_line(new_line).map_err(|_| ())
    }
}
