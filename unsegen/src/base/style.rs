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

    pub fn black() -> Self {
        Color::new(0,0,0)
    }
    pub fn white() -> Self {
        Color::new(255,255,255)
    }
    pub fn red() -> Self {
        Color::new(255,0,0)
    }
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
pub struct Style {
    fg_color: Color,
    bg_color: Color,
    format: TextFormat,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            fg_color: Color::white(),
            bg_color: Color::black(),
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
        use std::io::Write;

        write!(terminal, "{}", termion::color::Fg(self.fg_color.to_termion_color())).expect("write fgcolor");
        write!(terminal, "{}", termion::color::Bg(self.bg_color.to_termion_color())).expect("write bgcolor");

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
            style.format = format;
        }
    }
}
