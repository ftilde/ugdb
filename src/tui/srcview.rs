use unsegen::{
    Cursor,
    Demand,
    Event,
    FileLineStorage,
    HorizontalLayout,
    Key,
    LineNumber,
    LineIndex,
    MemoryLineStorage,
    ScrollBehavior,
    SeparatingStyle,
    Widget,
    Window,
};
use unsegen::widgets::{
    LineDecorator,
    LineNumberDecorator,
    Pager,
    PagerContent,
    PagerLine,
    SyntectHighLighter,
};
use input::{
    Input,
};
use syntect::highlighting::{
    Theme,
};
use syntect::parsing::{
    SyntaxSet,
};
use gdbmi::output::{
    NamedValues,
};
use std::io;
use std::path::{
    Path,
    PathBuf,
};
use gdbmi;
use gdbmi::input::{
    MiCommand,
};

#[derive(Debug)]
pub enum PagerShowError {
    CouldNotOpenFile(PathBuf, io::Error),
    LineDoesNotExist(LineIndex),
}

#[derive(Clone)]
struct AssemblyLine {
    content: String,
    address: usize,
}

impl AssemblyLine {
    fn new(content: String, address: usize) -> Self {
        AssemblyLine {
            content: content,
            address: address,
        }
    }
}

impl PagerLine for AssemblyLine {
    fn get_content(&self) -> &str {
        &self.content
    }
}

struct AssemblyDecorator;

impl LineDecorator for AssemblyDecorator {
    type Line = AssemblyLine;
    fn horizontal_space_demand<'a, 'b: 'a>(&'a self, lines: Box<DoubleEndedIterator<Item=(LineIndex, Self::Line)> + 'b>) -> Demand {
        let max_space = lines.last().map(|(_,l)| {
            ::unicode_width::UnicodeWidthStr::width(format!(" 0x{:x} ", l.address).as_str())
        }).unwrap_or(0);
        Demand::from_to(0, max_space as u32)
    }
    fn decorate(&self, line: &Self::Line, _: LineIndex, mut window: Window) {
        let width = window.get_width() as usize - 4;
        let mut cursor = Cursor::new(&mut window).position(0,0);

        use std::fmt::Write;
        let _ = write!(cursor, " 0x{:0>width$x} ", line.address, width=width);
    }
}

pub struct SourceView<'a> {
    highlighting_theme: &'a Theme,
    syntax_set: SyntaxSet,
    pager: Pager<FileLineStorage, SyntectHighLighter<'a>, LineNumberDecorator<String>>,
}

impl<'a> SourceView<'a> {
    pub fn new(highlighting_theme: &'a Theme) -> Self {
        SourceView {
            highlighting_theme: highlighting_theme,
            syntax_set: SyntaxSet::load_defaults_nonewlines(),
            pager: Pager::new(),
        }
    }

    pub fn show<P: AsRef<Path>, L: Into<LineIndex>>(&mut self, path: P, line: L) -> Result<(), PagerShowError> {
        let need_to_reload = if let Some(ref content) = self.pager.content {
            content.storage.get_file_path() != path.as_ref()
        } else {
            true
        };
        if need_to_reload {
            let path_ref = path.as_ref();
            try!{self.load(path_ref).map_err(|e| PagerShowError::CouldNotOpenFile(path_ref.to_path_buf(), e))};
        }
        let line = line.into();
        self.pager.go_to_line(line).map_err(|_| PagerShowError::LineDoesNotExist(line))
    }

    fn load<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file_storage = try!{FileLineStorage::new(path.as_ref())};
        let syntax = self.syntax_set.find_syntax_for_file(path.as_ref())
            .expect("file IS openable, see file storage")
            .unwrap_or(self.syntax_set.find_syntax_plain_text());
        self.pager.load(
            PagerContent::create(file_storage)
            .with_highlighter(SyntectHighLighter::new(syntax, self.highlighting_theme))
            .with_decorator(LineNumberDecorator::default())
            );
        Ok(())
    }

    fn current_line(&self) -> LineNumber {
        self.pager.current_line().into()
    }

    fn current_file(&self) -> Option<&Path> {
        if let Some(ref content) = self.pager.content {
            Some(content.storage.get_file_path())
        } else {
            None
        }
    }

    pub fn event(&mut self, event: Input, _ /*gdb*/: &mut gdbmi::GDB) {
        event.chain(ScrollBehavior::new(&mut self.pager)
                    .forwards_on(Key::PageDown)
                    .forwards_on(Key::Char('j'))
                    .backwards_on(Key::PageUp)
                    .backwards_on(Key::Char('k'))
                   );
    }
}

impl<'a> Widget for SourceView<'a> {
    fn space_demand(&self) -> (Demand, Demand) {
        self.pager.space_demand()
    }
    fn draw(&mut self, window: Window) {
        self.pager.draw(window)
    }
}

pub struct AssemblyView<'a> {
    highlighting_theme: &'a Theme,
    syntax_set: SyntaxSet,
    pager: Pager<MemoryLineStorage<AssemblyLine>, SyntectHighLighter<'a>, AssemblyDecorator>,
}

impl<'a> AssemblyView<'a> {
    pub fn new(highlighting_theme: &'a Theme) -> Self {
        AssemblyView {
            highlighting_theme: highlighting_theme,
            syntax_set: SyntaxSet::load_defaults_nonewlines(),
            pager: Pager::new(),
        }
    }

    pub fn show<P: AsRef<Path>, L: Into<LineNumber>>(&mut self, file: P, line: L, gdb: &mut gdbmi::GDB) -> Result<(), () /* Disassembly unsuccessful */> {
        let line_u: usize = line.into().into();
        let disass_obj = try!{gdb.execute(&MiCommand::data_disassemble_file(file, line_u, None)).expect("disassembly successful").results.remove("asm_insns").ok_or(())};
        let mut asm_storage = MemoryLineStorage::<AssemblyLine>::new();
        for tuple in disass_obj.unwrap_valuelist() {
            let mut tuple = tuple.unwrap_tuple_or_named_value_list();
            let instruction = tuple.remove("inst").expect("inst present").unwrap_const();
            let address = usize::from_str_radix(&tuple.remove("address").expect("address present").unwrap_const()[2..],16).expect("Parse address");
            asm_storage.lines.push(AssemblyLine::new(instruction, address));
        }
        let syntax = self.syntax_set.find_syntax_by_extension("s")
            .unwrap_or(self.syntax_set.find_syntax_plain_text());
        self.pager.load(
            PagerContent::create(asm_storage)
            .with_highlighter(SyntectHighLighter::new(syntax, self.highlighting_theme))
            .with_decorator(AssemblyDecorator));
        Ok(())
    }
    pub fn event(&mut self, event: Input, _ /*gdb*/: &mut gdbmi::GDB) {
        event.chain(ScrollBehavior::new(&mut self.pager)
                    .forwards_on(Key::PageDown)
                    .forwards_on(Key::Char('j'))
                    .backwards_on(Key::PageUp)
                    .backwards_on(Key::Char('k'))
                   );
    }
}

impl<'a> Widget for AssemblyView<'a> {
    fn space_demand(&self) -> (Demand, Demand) {
        self.pager.space_demand()
    }
    fn draw(&mut self, window: Window) {
        self.pager.draw(window)
    }
}

enum CodeWindowMode {
    Source,
    Assembly,
}

pub struct CodeWindow<'a> {
    src_view: SourceView<'a>,
    asm_view: AssemblyView<'a>,
    layout: HorizontalLayout,
    mode: CodeWindowMode,
}

impl<'a> CodeWindow<'a> {
    pub fn new(highlighting_theme: &'a Theme) -> Self {
        CodeWindow {
            src_view: SourceView::new(highlighting_theme),
            asm_view: AssemblyView::new(highlighting_theme),
            layout: HorizontalLayout::new(SeparatingStyle::Draw('|')),
            mode: CodeWindowMode::Source,
        }
    }
    pub fn show_frame(&mut self, mut frame: NamedValues, gdb: &mut gdbmi::GDB) {
        if let Some(path_object) = frame.remove("fullname") { // File information may not be present
            let path = path_object.unwrap_const();
            let line: LineNumber = frame.remove("line").expect("line present").unwrap_const().parse::<usize>().expect("Parse usize").into();
            let _ = self.src_view.show(&path, line); // GDB may give out invalid paths, so we just ignore them (at least for now)
            if self.asm_view.show(&path, line, gdb).is_err() {
                self.mode = CodeWindowMode::Source;
            };
        }
    }

    fn toggle_mode(&mut self, gdb: &mut gdbmi::GDB) {
        self.mode = match self.mode {
            CodeWindowMode::Assembly => {
                CodeWindowMode::Source
            },
            CodeWindowMode::Source => {
                if let Some(path) = self.src_view.current_file() {
                    if self.asm_view.show(path, self.src_view.current_line(), gdb).is_ok() {
                        CodeWindowMode::Assembly
                    } else {
                        CodeWindowMode::Source
                    }
                } else {
                    CodeWindowMode::Source
                }
            },
        }
    }

    pub fn event(&mut self, event: Input, gdb: &mut gdbmi::GDB) {
        event.chain(|i: Input| match i.event {
            Event::Key(Key::Char('d')) => {
                self.toggle_mode(gdb);
                None
            },
            _ => Some(i),
        }).chain(|i: Input| {
            match self.mode {
                CodeWindowMode::Assembly => self.asm_view.event(i, gdb),
                CodeWindowMode::Source => self.src_view.event(i, gdb),
            }
            None
        });
    }
}

impl<'a> Widget for CodeWindow<'a> {
    fn space_demand(&self) -> (Demand, Demand) {
        let widgets: Vec<&Widget> = match self.mode {
            CodeWindowMode::Assembly => vec![&self.asm_view, &self.src_view],
            CodeWindowMode::Source => vec![&self.src_view],
        };
        self.layout.space_demand(widgets.as_slice())
    }
    fn draw(&mut self, window: Window) {
        let mut widgets: Vec<&mut Widget> = match self.mode {
            CodeWindowMode::Assembly => vec![&mut self.asm_view, &mut self.src_view],
            CodeWindowMode::Source => vec![&mut self.src_view],
        };
        self.layout.draw(window, &mut widgets)
    }
}

