use termion;
use termion::raw::RawTerminal;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
    pub invert: bool,
    pub underline: bool,
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

    pub fn or(&self, other: &TextAttribute) -> TextAttribute {
        TextAttribute {
            fg_color: self.fg_color.or(other.fg_color),
            bg_color: self.bg_color.or(other.bg_color),
            style: self.style.or(&other.style),
        }
    }


    pub fn set_terminal_attributes(&self, terminal: &mut RawTerminal<::std::io::StdoutLock>) {
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
