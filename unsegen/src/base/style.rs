use termion;
use termion::raw::RawTerminal;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TextFormat {
    pub bold: bool,
    pub italic: bool,
    pub invert: bool,
    pub underline: bool,
}

impl TextFormat {
    pub fn new() -> Self {
        TextFormat {
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
    pub fn modify(&mut self, other: &TextFormat) {
        self.bold = other.bold;
        self.italic = other.italic;
        self.invert ^= other.invert;
        self.underline = other.underline;
    }
    /*
    pub fn or(&self, other: &Self) -> Self {
        TextFormat {
            bold: self.bold || other.bold,
            italic: self.italic || other.italic,
            invert: self.invert || other.invert,
            underline: self.underline || other.underline,
        }
    }
    */
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

impl Default for TextFormat {
    fn default() -> Self {
        TextFormat::new()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    Rgb {
        r: u8,
        g: u8,
        b: u8,
    },
    Black,
    Blue,
    Cyan,
    Green,
    Magenta,
    Red,
    White,
    Yellow,
    LightBlack,
    LightBlue,
    LightCyan,
    LightGreen,
    LightMagenta,
    LightRed,
    LightWhite,
    LightYellow,
}

impl Color {
    fn set_terminal_attributes_fg(&self, terminal: &mut RawTerminal<::std::io::StdoutLock>) -> ::std::io::Result<()> {
        use termion::color::Fg as Target;
        use std::io::Write;
        match self {
            &Color::Rgb { r, g, b } => write!(terminal, "{}", Target(termion::color::Rgb(r, g, b))),
            &Color::Black   => write!(terminal, "{}", Target(termion::color::Black)),
            &Color::Blue    => write!(terminal, "{}", Target(termion::color::Blue)),
            &Color::Cyan    => write!(terminal, "{}", Target(termion::color::Cyan)),
            &Color::Magenta => write!(terminal, "{}", Target(termion::color::Magenta)),
            &Color::Green   => write!(terminal, "{}", Target(termion::color::Green)),
            &Color::Red     => write!(terminal, "{}", Target(termion::color::Red)),
            &Color::White   => write!(terminal, "{}", Target(termion::color::White)),
            &Color::Yellow  => write!(terminal, "{}", Target(termion::color::Yellow)),
            &Color::LightBlack   => write!(terminal, "{}", Target(termion::color::LightBlack)),
            &Color::LightBlue    => write!(terminal, "{}", Target(termion::color::LightBlue)),
            &Color::LightCyan    => write!(terminal, "{}", Target(termion::color::LightCyan)),
            &Color::LightMagenta => write!(terminal, "{}", Target(termion::color::LightMagenta)),
            &Color::LightGreen   => write!(terminal, "{}", Target(termion::color::LightGreen)),
            &Color::LightRed     => write!(terminal, "{}", Target(termion::color::LightRed)),
            &Color::LightWhite   => write!(terminal, "{}", Target(termion::color::LightWhite)),
            &Color::LightYellow  => write!(terminal, "{}", Target(termion::color::LightYellow)),
        }
    }
    fn set_terminal_attributes_bg(&self, terminal: &mut RawTerminal<::std::io::StdoutLock>) -> ::std::io::Result<()> {
        use termion::color::Bg as Target;
        use std::io::Write;
        match self {
            &Color::Rgb { r, g, b } => write!(terminal, "{}", Target(termion::color::Rgb(r, g, b))),
            &Color::Black   => write!(terminal, "{}", Target(termion::color::Black)),
            &Color::Blue    => write!(terminal, "{}", Target(termion::color::Blue)),
            &Color::Cyan    => write!(terminal, "{}", Target(termion::color::Cyan)),
            &Color::Magenta => write!(terminal, "{}", Target(termion::color::Magenta)),
            &Color::Green   => write!(terminal, "{}", Target(termion::color::Green)),
            &Color::Red     => write!(terminal, "{}", Target(termion::color::Red)),
            &Color::White   => write!(terminal, "{}", Target(termion::color::White)),
            &Color::Yellow  => write!(terminal, "{}", Target(termion::color::Yellow)),
            &Color::LightBlack   => write!(terminal, "{}", Target(termion::color::LightBlack)),
            &Color::LightBlue    => write!(terminal, "{}", Target(termion::color::LightBlue)),
            &Color::LightCyan    => write!(terminal, "{}", Target(termion::color::LightCyan)),
            &Color::LightMagenta => write!(terminal, "{}", Target(termion::color::LightMagenta)),
            &Color::LightGreen   => write!(terminal, "{}", Target(termion::color::LightGreen)),
            &Color::LightRed     => write!(terminal, "{}", Target(termion::color::LightRed)),
            &Color::LightWhite   => write!(terminal, "{}", Target(termion::color::LightWhite)),
            &Color::LightYellow  => write!(terminal, "{}", Target(termion::color::LightYellow)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Style {
    fg_color: Color,
    bg_color: Color,
    format: TextFormat,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            fg_color: Color::White,
            bg_color: Color::LightBlack,
            format: TextFormat::default(),
        }
    }
}

impl Style {
    pub fn new(fg_color: Color, bg_color: Color, format: TextFormat) -> Self {
        Style {
            fg_color: fg_color,
            bg_color: bg_color,
            format: format
        }
    }

    pub fn plain() -> Self {
        Self::default()
    }

    pub fn set_terminal_attributes(&self, terminal: &mut RawTerminal<::std::io::StdoutLock>) {
        self.fg_color.set_terminal_attributes_fg(terminal).expect("write fg_color");
        self.bg_color.set_terminal_attributes_bg(terminal).expect("write bg_color");
        self.format.set_terminal_attributes(terminal);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StyleModifier {
    fg_color: Option<Color>,
    bg_color: Option<Color>,
    format: Option<TextFormat>,
}

impl StyleModifier {
    pub fn none() -> Self {
        StyleModifier {
            fg_color: None,
            bg_color: None,
            format: None,
        }
    }

    pub fn new() -> Self {
        Self::none()
    }

    pub fn fg_color(mut self, fg_color: Color) -> Self {
        self.fg_color = Some(fg_color);
        self
    }

    pub fn bg_color(mut self, bg_color: Color) -> Self {
        self.bg_color = Some(bg_color);
        self
    }

    pub fn format(mut self, format: TextFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn or(&self, other: &StyleModifier) -> Self {
        StyleModifier {
            fg_color: self.fg_color.or(other.fg_color),
            bg_color: self.bg_color.or(other.bg_color),
            format: self.format.or(other.format),
        }
    }

    pub fn apply_to_default(&self) -> Style {
        let mut style = Style::default();
        self.modify(&mut style);
        style
    }

    pub fn apply(&self, style: &Style) -> Style {
        let mut style = style.clone();
        self.modify(&mut style);
        style
    }

    pub fn modify(&self, style: &mut Style) {
        if let Some(fg) = self.fg_color {
            style.fg_color = fg;
        }
        if let Some(bg) = self.bg_color {
            style.bg_color = bg;
        }
        if let Some(format) = self.format {
            style.format.modify(&format);
        }
    }
}
