pub use termion::event::{Event, Key};
use termion;

pub mod widgets;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Standard,
    /*
    Bold,
    Faint,
    Italic,
    Underlined,
    Blink,
    Invert,
    */
}

impl Default for Style {
    fn default() -> Self {
        Style::Standard
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TextAttribute {
    fg_color: Option<Color>,
    bg_color: Option<Color>,
    style: Option<Style>,
    // for all members: None :<=> Don't care
}

impl Default for TextAttribute {
    fn default() -> Self {
        TextAttribute {
            fg_color: None,
            bg_color: None,
            style: None,
        }
    }
}

impl TextAttribute {

    pub fn new(fg: Option<Color>, bg: Option<Color>, style: Option<Style>) -> TextAttribute {
        TextAttribute {
            fg_color: fg,
            bg_color: bg,
            style: style,
        }
    }

    /*
    pub fn plain() -> TextAttribute {
        TextAttribute {
            fg_color: None,
            bg_color: None,
            style: None,
        }
    }
    */

    /*
    fn or(&self, other: &TextAttribute) -> TextAttribute {
        TextAttribute {
            fg_color: self.fg_color.or(other.fg_color),
            bg_color: self.bg_color.or(other.bg_color),
            style: self.style.or(other.style),
        }
    }
    */

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
        //TODO style
    }
}

#[derive(Clone, Copy)]
struct FormattedChar {
    character: char,
    format: TextAttribute,
}

impl FormattedChar {
    fn new(character: char, format: TextAttribute) -> Self {
        FormattedChar {
            character: character,
            format: format,
        }
    }
}

impl Default for FormattedChar {
    fn default() -> Self {
        Self::new(' ', TextAttribute::default())
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
        let dim = (x as Ix, y as Ix);
        //if dim != self.values.dim() {
        self.values = CharMatrix::default(dim);
        //}

        Window::new(self.values.view_mut(), default_format)
    }

    pub fn present(&mut self) {
        //self.values = CharMatrix::default((5, 10));
        //self.values[(1,1)] = FormattedChar::new('a', TextAttribute::default());
        use std::io::Write;
        //write!(self.terminal, "{}", termion::clear::All).expect("clear screen"); //Causes flickering and is unnecessary

        let mut current_format = TextAttribute::default();

        for (y, line) in self.values.axis_iter(Axis(1)).enumerate() {
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
                let character = match c.character {
                    '\n' => ' ',
                    '\r' => ' ',
                    '\0' => ' ',
                    '\t' => ' ', //TODO?
                    x => x,
                };
                buffer.push(character);
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
        self.values.dim().0 as u32
    }

    pub fn get_height(&self) -> u32 {
        self.values.dim().1 as u32
    }

    pub fn split_v(self, split_pos: u32) -> (Self, Self) {
        assert!(split_pos < self.get_height(), "Invalid split_pos");
        //let split_pos = ::std::cmp::min(split_pos, self.get_height());
        let (first_mat, second_mat) = self.values.split_at(Axis(1), split_pos as Ix);
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
        assert!(split_pos < self.get_width(), "Invalid split_pos");
        //let split_pos = ::std::cmp::min(split_pos, self.get_height());
        let (first_mat, second_mat) = self.values.split_at(Axis(0), split_pos as Ix);
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
        let mut cursor = self.create_cursor();
        for _ in 0..height {
            cursor.writeln(&line);
        }
    }

    pub fn set_default_format(&mut self, format: TextAttribute) {
        self.default_format = format;
    }

    pub fn create_cursor<'c>(&'c mut self) -> Cursor<'c, 'w>  {
        Cursor::new(self)
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
    fn new(window: &'c mut Window<'w>) -> Self {
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

    /*
    pub fn set_text_attribute(&mut self, ta: TextAttribute) {
        self.text_attribute = Some(ta)
    }

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

    pub fn write(&mut self, text: &str) {

        for character in text.chars() {
            if self.wrapping_mode == WrappingMode::Wrap && (self.x as u32) >= self.window.get_width() {
                self.wrap_line(); //TODO: properly handle for WrapDirection == Up
            }
            if character == '\n' {
                self.wrap_line();
            } else {
                if     0 <= self.x && (self.x as u32) < self.window.get_width()
                    && 0 <= self.y && (self.y as u32) < self.window.get_height() {

                    let pos = (self.x as Ix, self.y as Ix);
                    let text_attribute = if let Some(attr) = self.text_attribute {
                        attr.clone()
                    } else {
                        self.window.default_format.clone()
                    };
                    *self.window.values.get_mut(pos).expect("in bounds") = FormattedChar::new(character, text_attribute);
                }
                self.x += 1;
            }
        }
    }

    pub fn writeln(&mut self, text: &str) {
        self.write(text);
        self.wrap_line();
    }

}

#[derive(Eq, PartialEq, PartialOrd)]
pub enum Demand {
    MaxPossible,
    Const(u32),
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
        match (self, rhs) {
            (_, Demand::MaxPossible) => Demand::MaxPossible,
            (Demand::MaxPossible, _) => Demand::MaxPossible,
            (Demand::Const(a), Demand::Const(b)) => Demand::Const(a + b),
        }
    }
}

impl Demand {
    fn max(self, other: Self) -> Self {
        match (self, other) {
            (_, Demand::MaxPossible) => Demand::MaxPossible,
            (Demand::MaxPossible, _) => Demand::MaxPossible,
            (Demand::Const(a), Demand::Const(b)) => Demand::Const(::std::cmp::max(a, b)),
        }
    }
}

pub trait Widget {
    fn space_demand(&self) -> (Demand, Demand);
    fn draw(&self, window: Window);
    fn input(&mut self, Event); // -> bool?
}

#[derive(Clone, Copy)]
pub enum SeparatingStyle {
    None,
    //AlternateStyle(TextAttribute),
    Draw(char)
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

    pub fn space_demand<'a, T: Iterator<Item=&'a Widget> + 'a>(&'a self, widgets: T) -> (Demand, Demand) {
        let mut total_x = Demand::Const(0);
        let mut total_y = Demand::Const(0);
        let mut n_elements = 0;
        for w in widgets {
            let (x, y) = w.space_demand();
            total_x = total_x + x;
            total_y = total_y.max(y);
            n_elements += 1;
        }
        if let SeparatingStyle::Draw(_) = self.separating_style {
            total_x = total_x + Demand::Const(n_elements);
        }
        (total_x, total_y)
    }

    pub fn draw<'a, T: Iterator<Item=&'a Widget> + 'a>(&'a self, window: Window, widgets: T) {
        let mut widgets = widgets.peekable();
        let mut rest_window = window;
        let mut pos;
        while let Some(w) = widgets.next() {
            let (x, _) = w.space_demand();
            pos = match x {
                Demand::Const(i) => i,
                Demand::MaxPossible => rest_window.get_width(),
            };
            let (window, r) = rest_window.split_h(pos);
            rest_window = r;
            w.draw(window);
            if let (Some(_), SeparatingStyle::Draw(c)) = (widgets.peek(), self.separating_style) {
                let (mut window, r) = rest_window.split_h(1);
                rest_window = r;
                window.fill(c);
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

    pub fn space_demand<'a, T: Iterator<Item=&'a Widget> + 'a>(&'a self, widgets: T) -> (Demand, Demand) {
        let mut total_x = Demand::Const(0);
        let mut total_y = Demand::Const(0);
        let mut n_elements = 0;
        for w in widgets {
            let (x, y) = w.space_demand();
            total_x = total_x.max(x);
            total_y = total_y + y;
            n_elements += 1;
        }
        if let SeparatingStyle::Draw(_) = self.separating_style {
            total_y = total_y + Demand::Const(n_elements);
        }
        (total_x, total_y)
    }

    pub fn draw(&self, window: Window, widgets: &[&Widget]) {
        //TODO fix horizontal layout

        let mut space_claimed = 0;
        let mut num_max_possible = 0;
        for w in widgets {
            let (_, y) = w.space_demand();
            if let Demand::Const(claimed) = y {
                space_claimed += claimed;
            } else {
                num_max_possible += 1;
            }
        }
        if let SeparatingStyle::Draw(_) = self.separating_style {
            space_claimed += widgets.len() as u32;
        }

        let free_space = ::std::cmp::max(0, window.get_height()-space_claimed);
        let space_for_max_possible = free_space / ::std::cmp::max(1, num_max_possible);

        let mut widgets = widgets.into_iter().peekable();
        let mut rest_window = window;
        let mut pos;

        while let Some(w) = widgets.next() {
            let (_, y) = w.space_demand();
            pos = match y {
                Demand::Const(i) => i,
                Demand::MaxPossible => space_for_max_possible,
            };
            let (window, r) = rest_window.split_v(pos);
            rest_window = r;
            w.draw(window);
            if let (Some(_), SeparatingStyle::Draw(c)) = (widgets.peek(), self.separating_style) {
                let (mut window, r) = rest_window.split_v(1);
                rest_window = r;
                window.fill(c);
            }
        }
    }
}
