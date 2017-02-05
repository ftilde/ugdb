pub use termion::event::{Event, Key};
use termion;
use ::std::cmp::{max, min};

pub mod widgets;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Style {
    bold: bool,
    italic: bool,
    invert: bool,
    underline: bool,
}

impl Style {
    pub fn new() -> Self {
        Style {
            bold: false,
            italic: false,
            invert: false,
            underline: false,
        }
    }
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
    pub fn invert(mut self) -> Self {
        self.invert = true;
        self
    }
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
    pub fn or(&self, other: &Self) -> Self {
        Style {
            bold: self.bold || other.bold,
            italic: self.italic || other.italic,
            invert: self.invert || other.invert,
            underline: self.underline || other.underline,
        }
    }
    fn set_terminal_attributes(&self, terminal: &mut RawTerminal<::std::io::StdoutLock>) {
        use std::io::Write;

        if self.bold {
            write!(terminal, "{}", termion::style::Bold).expect("set bold style");
        } else {
            write!(terminal, "{}", termion::style::NoBold).expect("set no bold style");
        }

        if self.italic {
            write!(terminal, "{}", termion::style::Italic).expect("set italic style");
        } else {
            write!(terminal, "{}", termion::style::NoItalic).expect("set no italic style");
        }

        if self.invert {
            write!(terminal, "{}", termion::style::Invert).expect("set invert style");
        } else {
            write!(terminal, "{}", termion::style::NoInvert).expect("set no invert style");
        }

        if self.underline {
            write!(terminal, "{}", termion::style::Underline).expect("set underline style");
        } else {
            write!(terminal, "{}", termion::style::NoUnderline).expect("set no underline style");
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Style::new()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color {
            r: r,
            g: g,
            b: b,
        }
    }
    /*
    pub fn black() -> Self {
        Color::new(0,0,0)
    }
    pub fn white() -> Self {
        Color::new(255,255,255)
    }
    pub fn red() -> Self {
        Color::new(255,0,0)
    }
    */
    pub fn green() -> Self {
        Color::new(0,255,0)
    }
    pub fn blue() -> Self {
        Color::new(0,0,255)
    }
    fn to_termion_color(&self) -> termion::color::Rgb {
        termion::color::Rgb(self.r, self.g, self.b)
    }
    //TODO more colors...
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TextAttribute {
    fg_color: Option<Color>,
    bg_color: Option<Color>,
    style: Style,
    // for all members: None :<=> Don't care
}

impl Default for TextAttribute {
    fn default() -> Self {
        TextAttribute {
            fg_color: None,
            bg_color: None,
            style: Style::default(),
        }
    }
}

impl TextAttribute {

    pub fn new<T: Into<Option<Style>>, C1: Into<Option<Color>>, C2: Into<Option<Color>>>(fg: C1, bg: C2, style: T) -> TextAttribute {
        TextAttribute {
            fg_color: fg.into(),
            bg_color: bg.into(),
            style: style.into().unwrap_or(Style::default()),
        }
    }


    pub fn plain() -> TextAttribute {
        TextAttribute {
            fg_color: None,
            bg_color: None,
            style: Style::default(),
        }
    }

    fn or(&self, other: &TextAttribute) -> TextAttribute {
        TextAttribute {
            fg_color: self.fg_color.or(other.fg_color),
            bg_color: self.bg_color.or(other.bg_color),
            style: self.style.or(&other.style),
        }
    }


    fn set_terminal_attributes(&self, terminal: &mut RawTerminal<::std::io::StdoutLock>) {
        use std::io::Write;

        if let Some(color) = self.fg_color {
            write!(terminal, "{}", termion::color::Fg(color.to_termion_color())).expect("write fgcolor");
        } else {
            write!(terminal, "{}", termion::color::Fg(termion::color::Reset)).expect("write fg reset");
        }
        if let Some(color) = self.bg_color {
            write!(terminal, "{}", termion::color::Bg(color.to_termion_color())).expect("write bgcolor");
        } else {
            write!(terminal, "{}", termion::color::Bg(termion::color::Reset)).expect("write bg reset");
        }

        self.style.set_terminal_attributes(terminal);
    }
}

#[derive(Clone, Debug, PartialEq)]
struct FormattedChar {
    // Invariant: the contents of graphemeCluster is always valid utf8!
    grapheme_cluster: ::smallvec::SmallVec<[u8; 16]>,
    format: TextAttribute,
}

impl FormattedChar {
    fn new(grapheme_cluster: &str, format: TextAttribute) -> Self {
        let mut vec = ::smallvec::SmallVec::<[u8; 16]>::new();
        for byte in grapheme_cluster.bytes() {
            vec.push(byte);
        }
        FormattedChar {
            grapheme_cluster: vec,
            format: format,
        }
    }

    fn grapheme_cluster_as_str<'a>(&'a self) -> &'a str {
        // This is actually safe because graphemeCluster is always valid utf8.
        unsafe {
            ::std::str::from_utf8_unchecked(&self.grapheme_cluster)
        }
    }
}

impl Default for FormattedChar {
    fn default() -> Self {
        Self::new(" ", TextAttribute::default())
    }
}

use ndarray::{Array, ArrayViewMut, Axis, Ix};

use termion::raw::{IntoRawMode, RawTerminal};
type CharMatrix = Array<FormattedChar, (Ix,Ix)>;
pub struct Terminal<'a> {
    values: CharMatrix,
    terminal: RawTerminal<::std::io::StdoutLock<'a>>,
}

impl<'a> Terminal<'a> {
    pub fn new(stdout: ::std::io::StdoutLock<'a>) -> Self {
        use std::io::Write;
        let mut terminal = stdout.into_raw_mode().expect("raw terminal");
        write!(terminal, "{}", termion::cursor::Hide).expect("write: hide cursor");
        Terminal {
            values: CharMatrix::default((0,0)),
            terminal: terminal
        }
    }

    pub fn create_root_window(&mut self, default_format: TextAttribute) -> Window {
        let (x, y) = termion::terminal_size().expect("get terminal size");
        let dim = (y as Ix, x as Ix);
        //if dim != self.values.dim() {
        self.values = CharMatrix::default(dim);
        //}

        Window::new(self.values.view_mut(), default_format)
    }

    pub fn present(&mut self) {
        use std::io::Write;
        //write!(self.terminal, "{}", termion::clear::All).expect("clear screen"); //Causes flickering and is unnecessary

        let mut current_format = TextAttribute::default();

        for (y, line) in self.values.axis_iter(Axis(0)).enumerate() {
            write!(self.terminal, "{}", termion::cursor::Goto(1, (y+1) as u16)).expect("move cursor");
            let mut buffer = String::with_capacity(line.len());
            for c in line.iter() {
                //TODO style
                if c.format != current_format {
                    current_format.set_terminal_attributes(&mut self.terminal);
                    write!(self.terminal, "{}", buffer).expect("write buffer");
                    buffer.clear();
                    current_format = c.format;
                }
                let grapheme_cluster = match c.grapheme_cluster_as_str() {
                    "\n" | "\r" | "\0" | "\t" => " ",
                    x => x,
                };
                buffer.push_str(grapheme_cluster);
            }
            current_format.set_terminal_attributes(&mut self.terminal);
            write!(self.terminal, "{}", buffer).expect("write leftover buffer contents");
        }
        self.terminal.flush().expect("flush terminal");
    }
}

impl<'a> Drop for Terminal<'a> {
    fn drop(&mut self) {
        use std::io::Write;
        write!(self.terminal, "{}", termion::cursor::Show).expect("show cursor");
    }
}

type CharMatrixView<'w> = ArrayViewMut<'w, FormattedChar, (Ix,Ix)>;
pub struct Window<'w> {
    pos_x: u32,
    pos_y: u32,
    values: CharMatrixView<'w>,
    default_format: TextAttribute,
}

impl<'w> Window<'w> {
    fn new(values: CharMatrixView<'w>, default_format: TextAttribute) -> Self {
        Window {
            pos_x: 0,
            pos_y: 0,
            values: values,
            default_format: default_format,
        }
    }

    pub fn get_width(&self) -> u32 {
        self.values.dim().1 as u32
    }

    pub fn get_height(&self) -> u32 {
        self.values.dim().0 as u32
    }

    pub fn split_v(self, split_pos: u32) -> (Self, Self) {
        assert!(split_pos <= self.get_height(), "Invalid split_pos");
        //let split_pos = min(split_pos, self.get_height());
        let (first_mat, second_mat) = self.values.split_at(Axis(0), split_pos as Ix);
        let w_u = Window {
            pos_x: self.pos_x,
            pos_y: self.pos_y,
            values: first_mat,
            default_format: self.default_format,
        };
        let w_d = Window {
            pos_x: self.pos_x,
            pos_y: self.pos_y+split_pos,
            values: second_mat,
            default_format: self.default_format,
        };
        (w_u, w_d)
    }

    pub fn split_h(self, split_pos: u32) -> (Self, Self) {
        assert!(split_pos <= self.get_width(), "Invalid split_pos");
        //let split_pos = min(split_pos, self.get_height());
        let (first_mat, second_mat) = self.values.split_at(Axis(1), split_pos as Ix);
        let w_l = Window {
            pos_x: self.pos_x,
            pos_y: self.pos_y,
            values: first_mat,
            default_format: self.default_format,
        };
        let w_r = Window {
            pos_x: self.pos_x+split_pos,
            pos_y: self.pos_y,
            values: second_mat,
            default_format: self.default_format,
        };
        (w_l, w_r)
    }

    pub fn fill(&mut self, c: char) {
        let mut line = String::with_capacity(self.get_width() as usize);
        for _ in 0..self.get_width() {
            line.push(c);
        }
        let height = self.get_height();
        let mut cursor = Cursor::new(self);
        for _ in 0..height {
            cursor.writeln(&line);
        }
    }

    pub fn set_default_format(&mut self, format: TextAttribute) {
        self.default_format = format;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WrappingDirection {
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WrappingMode {
    Wrap,
    NoWrap,
}


pub struct Cursor<'c, 'w: 'c> {
    window: &'c mut Window<'w>,
    wrapping_direction: WrappingDirection,
    wrapping_mode: WrappingMode,
    text_attribute: Option<TextAttribute>,
    x: i32,
    y: i32,
}

impl<'c, 'w> Cursor<'c, 'w> {
    pub fn new(window: &'c mut Window<'w>) -> Self {
        Cursor {
            window: window,
            wrapping_direction: WrappingDirection::Down,
            wrapping_mode: WrappingMode::NoWrap,
            text_attribute: None,
            x: 0,
            y: 0,
        }
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn position(mut self, x: i32, y: i32) -> Self {
        self.set_position(x, y);
        self
    }

    pub fn set_wrapping_direction(&mut self, wrapping_direction: WrappingDirection) {
        self.wrapping_direction = wrapping_direction;
    }

    pub fn wrapping_direction(mut self, wrapping_direction: WrappingDirection) -> Self {
        self.set_wrapping_direction(wrapping_direction);
        self
    }

    pub fn set_wrapping_mode(&mut self, wm: WrappingMode) {
        self.wrapping_mode = wm;
    }

    pub fn wrapping_mode(mut self, wm: WrappingMode) -> Self {
        self.set_wrapping_mode(wm);
        self
    }

    pub fn set_text_attribute(&mut self, ta: TextAttribute) {
        self.text_attribute = Some(ta)
    }

    /*
    pub fn text_attribute(mut self, ta: TextAttribute) -> Self {
        self.set_text_attribute(ta);
        self
    }
    */

    fn wrap_line(&mut self) {
        match self.wrapping_direction {
            WrappingDirection::Down => {
                self.y += 1;
            },
            WrappingDirection::Up => {
                self.y -= 1;
            },
        }
        self.x = 0;
    }

    fn write_grapheme_cluster_unchecked(&mut self, cluster: FormattedChar) {
        *self.window.values.get_mut((self.y as Ix, self.x as Ix)).expect("in bounds") = cluster;
    }

    fn active_text_attribute(&self) -> TextAttribute {
        if let Some(attr) = self.text_attribute {
            attr.or(&self.window.default_format)
        } else {
            self.window.default_format.clone()
        }
    }

    pub fn write(&mut self, text: &str) {

        let mut line_it = text.lines().peekable();
        while let Some(line) = line_it.next() {
            let num_auto_wraps = if self.wrapping_mode == WrappingMode::Wrap {
                let num_chars = line.chars().count(); //TODO: we do not really want chars, but the real width of the line
                max(0, (num_chars as i32 + self.x) / (self.window.get_width() as i32))
            } else {
                0
            };
            if self.wrapping_direction == WrappingDirection::Up {
                self.y -= num_auto_wraps; // reserve space for auto wraps
            }
            for grapheme_cluster in ::unicode_segmentation::UnicodeSegmentation::graphemes(line, true) {
                if self.wrapping_mode == WrappingMode::Wrap && (self.x as u32) >= self.window.get_width() {
                    self.y += 1;
                    self.x = 0;
                }
                if     0 <= self.x && (self.x as u32) < self.window.get_width()
                    && 0 <= self.y && (self.y as u32) < self.window.get_height() {

                    let text_attribute = self.active_text_attribute();
                    self.write_grapheme_cluster_unchecked(FormattedChar::new(grapheme_cluster, text_attribute));
                }
                self.x += 1;
                let cluster_width = ::unicode_width::UnicodeWidthStr::width(grapheme_cluster);
                if cluster_width > 1 {
                    let text_attribute = self.active_text_attribute();
                    for _ in 1..cluster_width {
                        if self.x >= self.window.get_width() as i32 {
                            break;
                        }
                        self.write_grapheme_cluster_unchecked(FormattedChar::new(grapheme_cluster, text_attribute.clone()));
                        self.x += 1;
                    }
                }
            }
            if self.wrapping_direction == WrappingDirection::Up {
                self.y -= num_auto_wraps; // Jump back to first line
            }
            if line_it.peek().is_some() {
                self.wrap_line();
            }
        }
    }

    pub fn writeln(&mut self, text: &str) {
        self.write(text);
        self.wrap_line();
    }

}

#[derive(Eq, PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct Demand {
    min: u32,
    max: Option<u32>,
}

/*
impl Ord for Demand {
    fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
        match (self, other) {
            (MaxPossible, MaxPossible) => Ordering::Equal,
            (Const(_), MaxPossible) => Ordering::Less,
            (MaxPossible, Const(_)) => Ordering::Greater,
            (Const(a), Const(b)) => cmp(a, b),
        }
    }
}
*/
impl ::std::ops::Add<Demand> for Demand {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Demand {
            min: self.min + rhs.min,
            max: if let (Some(l), Some(r)) = (self.max, rhs.max) {
                Some(l+r)
            } else {
                None
            }
        }
    }
}

impl Demand {
    fn exact(size: u32) -> Self {
        Demand {
            min: size,
            max: Some(size),
        }
    }
    fn at_least(size: u32) -> Self {
        Demand {
            min: size,
            max: None,
        }
    }
    fn from_to(min: u32, max: u32) -> Self {
        debug_assert!(min <= max, "Invalid min/max");
        Demand {
            min: min,
            max: Some(max),
        }
    }

    fn max(self, other: Self) -> Self {
        Demand {
            min: max(self.min, other.min),
            max: if let (Some(l), Some(r)) = (self.max, other.max) {
                Some(max(l, r))
            } else {
                None
            }
        }
    }
}

pub trait Widget {
    fn space_demand(&self) -> (Demand, Demand);
    fn draw(&mut self, window: Window);
    fn input(&mut self, Event); // -> bool?
}

#[derive(Clone, Copy)]
pub enum SeparatingStyle {
    None,
    //AlternateStyle(TextAttribute),
    Draw(char)
}
fn layout_linearly(mut available_space: u32, separator_width: u32, demands: &[Demand]) -> Box<[u32]>
{
    let mut assigned_spaces = vec![0; demands.len()].into_boxed_slice();
    let mut max_max_demand = None;
    let mut num_unbounded = 0;
    for (i, demand) in demands.iter().enumerate() {
        if let Some(max_demand) = demand.max {
            max_max_demand = Some(max(max_max_demand.unwrap_or(0), max_demand));
        } else {
            num_unbounded += 1;
        }
        let assigned_space = min(available_space, demand.min);
        available_space -= assigned_space;
        assigned_spaces[i] = assigned_space;

        let separator_width = if i == (demands.len()-1) { //Last element does not have a following separator
            0
        } else {
            separator_width
        };

        if available_space <= separator_width {
            return assigned_spaces;
        }
        available_space -= separator_width;
        //println!("Step 1: {:?}, {:?} left", assigned_spaces, available_space);
    }

    if let Some(total_max_max_demand) = max_max_demand {
        for (i, demand) in demands.iter().enumerate() {
            let max_demand = demand.max.unwrap_or(total_max_max_demand);
            if max_demand > demand.min {
                let additional_assigned_space = min(available_space, max_demand - demand.min);
                available_space -= additional_assigned_space;
                assigned_spaces[i] += additional_assigned_space;

                //println!("Step 2: {:?}, {:?} left", assigned_spaces, available_space);
                if available_space == 0 {
                    return assigned_spaces;
                }
            }
        }
    }

    if num_unbounded == 0 {
        return assigned_spaces;
    }

    let left_over_space_per_unbounded_widget = available_space / num_unbounded; //Rounded down!

    for (i, _) in demands.iter().enumerate().filter(|&(_, w)| w.max.is_none()) {
        let additional_assigned_space = min(available_space, left_over_space_per_unbounded_widget);
        available_space -= additional_assigned_space;
        assigned_spaces[i] += additional_assigned_space;

        //println!("Step 3: {:?}, {:?} left", assigned_spaces, available_space);
        if available_space == 0 {
            return assigned_spaces;
        }
    }

    assigned_spaces
}


pub struct HorizontalLayout {
    separating_style: SeparatingStyle,
}
impl HorizontalLayout {
    pub fn new(separating_style: SeparatingStyle) -> Self {
        HorizontalLayout {
            separating_style: separating_style,
        }
    }

    pub fn space_demand(&self, widgets: &[&Widget]) -> (Demand, Demand) {
        let mut total_x = Demand::exact(0);
        let mut total_y = Demand::exact(0);
        let mut n_elements = 0;
        for w in widgets {
            let (x, y) = w.space_demand();
            total_x = total_x + x;
            total_y = total_y.max(y);
            n_elements += 1;
        }
        if let SeparatingStyle::Draw(_) = self.separating_style {
            total_x = total_x + Demand::exact(n_elements);
        }
        (total_x, total_y)
    }

    pub fn draw(&self, window: Window, widgets: &mut [&mut Widget]) {

        let separator_width = if let SeparatingStyle::Draw(_) = self.separating_style { 1 } else { 0 };
        let vertical_demands: Vec<Demand> = widgets.iter().map(|w| w.space_demand().0.clone()).collect();
        let assigned_spaces = layout_linearly(window.get_width(), separator_width, vertical_demands.as_slice());

        debug_assert!(widgets.len() == assigned_spaces.len(), "widgets and spaces len mismatch");

        let mut rest_window = window;
        let mut iter = widgets.iter_mut().zip(assigned_spaces.iter()).peekable();
        while let Some((&mut ref mut w, &pos)) = iter.next() {
            let (window, r) = rest_window.split_h(pos);
            rest_window = r;
            w.draw(window);
            if let (Some(_), SeparatingStyle::Draw(c)) = (iter.peek(), self.separating_style) {
                if rest_window.get_width() > 0 {
                    let (mut window, r) = rest_window.split_h(1);
                    rest_window = r;
                    window.fill(c);
                }
            }
        }
    }
}

pub struct VerticalLayout {
    separating_style: SeparatingStyle,
}

impl VerticalLayout {
    pub fn new(separating_style: SeparatingStyle) -> Self {
        VerticalLayout {
            separating_style: separating_style,
        }
    }

    pub fn space_demand(&self, widgets: &[&Widget]) -> (Demand, Demand) {
        let mut total_x = Demand::exact(0);
        let mut total_y = Demand::exact(0);
        let mut n_elements = 0;
        for w in widgets.iter() {
            let (x, y) = w.space_demand();
            total_x = total_x.max(x);
            total_y = total_y + y;
            n_elements += 1;
        }
        if let SeparatingStyle::Draw(_) = self.separating_style {
            total_y = total_y + Demand::exact(n_elements);
        }
        (total_x, total_y)
    }

    pub fn draw(&self, window: Window, widgets: &mut [&mut Widget]) {

        let separator_width = if let SeparatingStyle::Draw(_) = self.separating_style { 1 } else { 0 };
        let vertical_demands: Vec<Demand> = widgets.iter().map(|w| w.space_demand().1.clone()).collect();
        let assigned_spaces = layout_linearly(window.get_height(), separator_width, vertical_demands.as_slice());

        debug_assert!(widgets.len() == assigned_spaces.len(), "widgets and spaces len mismatch");

        let mut rest_window = window;
        let mut iter = widgets.iter_mut().zip(assigned_spaces.iter()).peekable();
        while let Some((&mut ref mut w, &pos)) = iter.next() {
            let (window, r) = rest_window.split_v(pos);
            rest_window = r;
            w.draw(window);
            if let (Some(_), SeparatingStyle::Draw(c)) = (iter.peek(), self.separating_style) {
                if rest_window.get_height() > 0 {
                    let (mut window, r) = rest_window.split_v(1);
                    rest_window = r;
                    window.fill(c);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;


    #[derive(PartialEq)]
    struct FakeTerminal {
        values: CharMatrix,
    }
    impl FakeTerminal {
        fn with_size((w, h): (Ix, Ix)) -> Self {
            FakeTerminal {
                values: CharMatrix::default((h, w)),
            }
        }

        fn create_root_window(&mut self) -> Window {
            Window::new(self.values.view_mut(), TextAttribute::plain())
        }

        fn from_str((w, h): (Ix, Ix), description: &str) -> Result<Self, ::ndarray::ShapeError>{
            let mut tiles = Vec::<FormattedChar>::new();
            for c in description.chars() {
                if c.is_whitespace() {
                    continue;
                }
                tiles.push(FormattedChar::new(&c.to_string(), TextAttribute::plain()));
            }
            Ok(FakeTerminal {
                values: try!{::ndarray::Array2::from_shape_vec((h, w), tiles)},
            })
        }
    }


    impl ::std::fmt::Debug for FakeTerminal {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            for r in 0..self.values.dim().0 {
                try!{write!(f, "[")};
                for c in 0..self.values.dim().1 {
                    let c = self.values.get((r, c)).expect("debug: in bounds");
                    try!{write!(f, "{:?}, ", c.grapheme_cluster_as_str())};
                }
                try!{write!(f, "]\n")};
            }
            Ok(())
        }
    }


    struct FakeWidget {
        space_demand: (Demand, Demand),
        fill_char: char,
    }
    impl FakeWidget {
        fn new(space_demand: (Demand, Demand)) -> Self {
            FakeWidget {
                space_demand: space_demand,
                fill_char: '_',
            }
        }
        fn with_fill_char(space_demand: (Demand, Demand), fill_char: char) -> Self {
            FakeWidget {
                space_demand: space_demand,
                fill_char: fill_char,
            }
        }
    }
    impl Widget for FakeWidget {
        fn space_demand(&self) -> (Demand, Demand) {
            self.space_demand
        }
        fn draw(&mut self, mut window: Window) {
            window.fill(self.fill_char);
        }
        fn input(&mut self, _: Event) {
            //Noop
        }
    }


    fn assert_eq_boxed_slices<T: PartialEq+::std::fmt::Debug>(b1: Box<[T]>, b2: Box<[T]>, description: &str) {
        assert_eq!(b1, b2, "{}", description);
    }

    #[test]
    fn test_layout_linearly_exact() {
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::exact(1), Demand::exact(2)]), Box::new([1, 2]), "some left");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::exact(1), Demand::exact(3)]), Box::new([1, 3]), "exact");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::exact(2), Demand::exact(3)]), Box::new([2, 2]), "less for 2nd");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::exact(5), Demand::exact(3)]), Box::new([4, 0]), "none for 2nd");
    }

    #[test]
    fn test_layout_linearly_from_to() {
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::from_to(1, 2), Demand::from_to(1, 2)]), Box::new([2, 2]), "both hit max");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::from_to(1, 2), Demand::from_to(1, 3)]), Box::new([2, 2]), "less for 2nd");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::from_to(5, 6), Demand::from_to(1, 4)]), Box::new([4, 0]), "nothing for 2nd");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::from_to(1, 5), Demand::from_to(1, 4)]), Box::new([3, 1]), "less for 1st");
    }

    #[test]
    fn test_layout_linearly_from_at_least() {
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::at_least(1), Demand::at_least(1)]), Box::new([2, 2]), "more for both");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::at_least(1), Demand::at_least(2)]), Box::new([1, 2]), "exact for both, devisor test");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::at_least(2), Demand::at_least(2)]), Box::new([2, 2]), "exact for both");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::at_least(5), Demand::at_least(2)]), Box::new([4, 0]), "none for 2nd");
    }

    #[test]
    fn test_layout_linearly_mixed() {
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::exact(3), Demand::at_least(1)]), Box::new([3, 7]), "exact, 2nd takes rest, no separator");
        assert_eq_boxed_slices(layout_linearly(10, 1, &[Demand::exact(3), Demand::at_least(1)]), Box::new([3, 6]), "exact, 2nd takes rest, separator");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(1, 2), Demand::at_least(1)]), Box::new([2, 8]), "from_to, 2nd takes rest");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(1, 2), Demand::exact(3), Demand::at_least(1)]), Box::new([2, 3, 5]), "return paths: end");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(5, 6), Demand::exact(5), Demand::at_least(5)]), Box::new([5, 5, 0]), "return paths: first loop, 2nd it");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(4, 6), Demand::exact(4), Demand::at_least(3)]), Box::new([4, 4, 2]), "return paths: first loop, 3rd it, rest");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(3, 6), Demand::exact(4), Demand::at_least(3)]), Box::new([3, 4, 3]), "return paths: first loop, 3rd it, full");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(3, 6), Demand::exact(3), Demand::at_least(3)]), Box::new([4, 3, 3]), "return paths: second loop, 1st it");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(2, 4), Demand::exact(2), Demand::at_least(3)]), Box::new([4, 2, 4]), "return paths: second loop, 3rd it");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(2, 4), Demand::exact(2), Demand::exact(3)]), Box::new([4, 2, 3]), "return paths: after second loop, 3rd it");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(2, 4), Demand::exact(2), Demand::at_least(4)]), Box::new([4, 2, 4]), "return paths: third loop, after 1st item");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(2, 3), Demand::at_least(2), Demand::at_least(2)]), Box::new([3, 3, 3]), "return paths: third loop, finished");
    }

    fn aeq_horizontal_layout_space_demand(widgets: Vec<&Widget>, solution: (Demand, Demand)) {
        assert_eq!(HorizontalLayout::new(SeparatingStyle::None).space_demand(widgets.as_slice()), solution);
    }
    #[test]
    fn test_horizontal_layout_space_demand() {
        aeq_horizontal_layout_space_demand(vec![&FakeWidget::new((Demand::exact(1), Demand::exact(2))), &FakeWidget::new((Demand::exact(1), Demand::exact(2)))], (Demand::exact(2), Demand::exact(2)));
        aeq_horizontal_layout_space_demand(vec![&FakeWidget::new((Demand::from_to(1, 2), Demand::from_to(1, 3))), &FakeWidget::new((Demand::exact(1), Demand::exact(2)))], (Demand::from_to(2, 3), Demand::from_to(2, 3)));
        aeq_horizontal_layout_space_demand(vec![&FakeWidget::new((Demand::at_least(3), Demand::at_least(3))), &FakeWidget::new((Demand::exact(1), Demand::exact(5)))], (Demand::at_least(4), Demand::at_least(5)));
    }
    fn aeq_horizontal_layout_draw(terminal_size: (usize, usize), mut widgets: Vec<&mut Widget>, solution: &str) {
        let mut term = FakeTerminal::with_size(terminal_size);
        HorizontalLayout::new(SeparatingStyle::None).draw(term.create_root_window(), widgets.as_mut_slice());
        assert_eq!(term, FakeTerminal::from_str(terminal_size, solution).expect("term from str"));
    }
    #[test]
    fn test_horizontal_layout_draw() {
        aeq_horizontal_layout_draw((4, 1), vec![&mut FakeWidget::with_fill_char((Demand::exact(2), Demand::exact(1)), '1'), &mut FakeWidget::with_fill_char((Demand::exact(2), Demand::exact(1)), '2')], "1122");
        aeq_horizontal_layout_draw((4, 1), vec![&mut FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(1)), '1'), &mut FakeWidget::with_fill_char((Demand::at_least(2), Demand::exact(1)), '2')], "1222");
        aeq_horizontal_layout_draw((4, 2), vec![&mut FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(1)), '1'), &mut FakeWidget::with_fill_char((Demand::at_least(2), Demand::exact(2)), '2')], "1222 1222");
    }

    fn aeq_vertical_layout_space_demand(widgets: Vec<&Widget>, solution: (Demand, Demand)) {
        assert_eq!(VerticalLayout::new(SeparatingStyle::None).space_demand(widgets.as_slice()), solution);
    }
    #[test]
    fn test_vertical_layout_space_demand() {
        aeq_vertical_layout_space_demand(vec![&FakeWidget::new((Demand::exact(2), Demand::exact(1))), &FakeWidget::new((Demand::exact(2), Demand::exact(1)))], (Demand::exact(2), Demand::exact(2)));
        aeq_vertical_layout_space_demand(vec![&FakeWidget::new((Demand::from_to(1, 3), Demand::from_to(1, 2))), &FakeWidget::new((Demand::exact(2), Demand::exact(1)))], (Demand::from_to(2, 3), Demand::from_to(2, 3)));
        aeq_vertical_layout_space_demand(vec![&FakeWidget::new((Demand::at_least(3), Demand::at_least(3))), &FakeWidget::new((Demand::exact(5), Demand::exact(1)))], (Demand::at_least(5), Demand::at_least(4)));
    }
    fn aeq_vertical_layout_draw(terminal_size: (usize, usize), mut widgets: Vec<&mut Widget>, solution: &str) {
        let mut term = FakeTerminal::with_size(terminal_size);
        VerticalLayout::new(SeparatingStyle::None).draw(term.create_root_window(), widgets.as_mut_slice());
        assert_eq!(term, FakeTerminal::from_str(terminal_size, solution).expect("term from str"));
    }
    #[test]
    fn test_vertical_layout_draw() {
        aeq_vertical_layout_draw((1, 4), vec![&mut FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(2)), '1'), &mut FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(2)), '2')], "1 1 2 2");
        aeq_vertical_layout_draw((1, 4), vec![&mut FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(1)), '1'), &mut FakeWidget::with_fill_char((Demand::exact(1), Demand::at_least(2)), '2')], "1 2 2 2");
        aeq_vertical_layout_draw((2, 4), vec![&mut FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(1)), '1'), &mut FakeWidget::with_fill_char((Demand::exact(2), Demand::at_least(2)), '2')], "11 22 22 22");
    }
}
