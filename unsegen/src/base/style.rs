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
        TextFormat {
            bold: false,
            italic: false,
            invert: false,
            underline: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TextFormatModifier {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub invert: bool, //not optional, but false by default, as it will toggle
    pub underline: Option<bool>,
}

impl TextFormatModifier {
    pub fn new() -> Self {
        TextFormatModifier {
            bold: None,
            italic: None,
            invert: false,
            underline: None,
        }
    }
    pub fn bold(mut self, val: bool) -> Self {
        self.bold = Some(val);
        self
    }
    pub fn italic(mut self, val: bool) -> Self {
        self.italic = Some(val);
        self
    }
    pub fn invert(mut self) -> Self {
        self.invert ^= true;
        self
    }
    pub fn underline(mut self, val: bool) -> Self {
        self.underline = Some(val);
        self
    }
    fn or(&self, other: &TextFormatModifier) -> Self {
        TextFormatModifier {
            bold: self.bold.or(other.bold),
            italic: self.italic.or(other.italic),
            invert: self.invert ^ other.invert,
            underline: self.underline.or(other.underline),
        }
    }

    fn modify(&self, format: &mut TextFormat) {
        if let Some(bold) = self.bold {
            format.bold = bold;
        }
        if let Some(italic) = self.italic {
            format.italic = italic;
        }
        format.invert ^= self.invert;
        if let Some(underline) = self.underline {
            format.underline = underline;
        }
    }
}

impl Default for TextFormatModifier {
    fn default() -> Self {
        TextFormatModifier::new()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    Rgb {
        r: u8,
        g: u8,
        b: u8,
    },
    Ansi(u8),
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
    pub fn ansi_rgb(r: u8, g: u8, b: u8) -> Self {
        Color::Ansi(termion::color::AnsiValue::rgb(r,g,b).0)
    }
    pub fn ansi_grayscale(v: u8 /* < 24 */) -> Self {
        Color::Ansi(termion::color::AnsiValue::grayscale(v).0)
    }

    fn set_terminal_attributes_fg(&self, terminal: &mut RawTerminal<::std::io::StdoutLock>) -> ::std::io::Result<()> {
        use termion::color::Fg as Target;
        use std::io::Write;
        match self {
            &Color::Rgb { r, g, b } => write!(terminal, "{}", Target(termion::color::Rgb(r, g, b))),
            &Color::Ansi(v) => write!(terminal, "{}", Target(termion::color::AnsiValue(v))),
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
            &Color::Ansi(v) => write!(terminal, "{}", Target(termion::color::AnsiValue(v))),
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
    format: TextFormatModifier,
}

impl StyleModifier {
    pub fn none() -> Self {
        StyleModifier {
            fg_color: None,
            bg_color: None,
            format: TextFormatModifier::new(),
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

    pub fn format(mut self, format: TextFormatModifier) -> Self {
        self.format = format;
        self
    }

    // Convenience functions to access text format
    pub fn bold(mut self, val: bool) -> Self {
        self.format.bold = Some(val);
        self
    }
    pub fn italic(mut self, val: bool) -> Self {
        self.format.italic = Some(val);
        self
    }
    pub fn invert(mut self) -> Self {
        self.format.invert ^= true;
        self
    }
    pub fn underline(mut self, val: bool) -> Self {
        self.format.underline = Some(val);
        self
    }

    pub fn or(&self, other: &StyleModifier) -> Self {
        StyleModifier {
            fg_color: self.fg_color.or(other.fg_color),
            bg_color: self.bg_color.or(other.bg_color),
            format: self.format.or(&other.format),
        }
    }

    pub fn if_not(&self, other: StyleModifier) -> Self {
        other.or(&self)
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
        self.format.modify(&mut style.format);
    }
}
