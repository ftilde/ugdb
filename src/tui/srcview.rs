use unsegen::{
    Cursor,
    Demand,
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

pub struct SrcView<'a> {
    highlighting_theme: &'a Theme,
    syntax_set: SyntaxSet,
    file_viewer: Pager<FileLineStorage, SyntectHighLighter<'a>, LineNumberDecorator<String>>,
    asm_viewer: Pager<MemoryLineStorage<AssemblyLine>, SyntectHighLighter<'a>, AssemblyDecorator>,
    layout: HorizontalLayout,
}

impl<'a> SrcView<'a> {
    pub fn new(highlighting_theme: &'a Theme) -> Self {
        SrcView {
            highlighting_theme: highlighting_theme,
            syntax_set: SyntaxSet::load_defaults_nonewlines(),
            file_viewer: Pager::new(),
            asm_viewer: Pager::new(),
            layout: HorizontalLayout::new(SeparatingStyle::Draw('|')),
        }
    }
    pub fn show_frame(&mut self, mut frame: NamedValues, gdb: &mut gdbmi::GDB) {
        if let Some(path_object) = frame.remove("fullname") { // File information may not be present
            let path = path_object.unwrap_const();
            let line: LineNumber = frame.remove("line").expect("line present").unwrap_const().parse::<usize>().expect("Parse usize").into();
            let _ = self.show_in_file_viewer(&path, line); // GDB may give out invalid paths, so we just ignore them (at least for now)
            self.show_in_asm_viewer(&path, line, gdb); // GDB may give out invalid paths, so we just ignore them (at least for now)
        }
    }

    pub fn show_in_asm_viewer<P: AsRef<Path>, L: Into<LineNumber>>(&mut self, file: P, line: L, gdb: &mut gdbmi::GDB) {
        let line_u: usize = line.into().into();
        let disass_obj = gdb.execute(&MiCommand::data_disassemble_file(file, line_u, None)).expect("disassembly successful").results.remove("asm_insns").expect("asm_insns present");
        let mut asm_storage = MemoryLineStorage::<AssemblyLine>::new();
        for tuple in disass_obj.unwrap_valuelist() {
            let mut tuple = tuple.unwrap_tuple_or_named_value_list();
            let instruction = tuple.remove("inst").expect("inst present").unwrap_const();
            let address = usize::from_str_radix(&tuple.remove("address").expect("address present").unwrap_const()[2..],16).expect("Parse address");
            asm_storage.lines.push(AssemblyLine::new(instruction, address));
        }
        let syntax = self.syntax_set.find_syntax_by_extension("s")
            .unwrap_or(self.syntax_set.find_syntax_plain_text());
        self.asm_viewer.load(
            PagerContent::create(asm_storage)
            .with_highlighter(SyntectHighLighter::new(syntax, self.highlighting_theme))
            .with_decorator(AssemblyDecorator));
    }

    pub fn show_in_file_viewer<P: AsRef<Path>, L: Into<LineIndex>>(&mut self, path: P, line: L) -> Result<(), PagerShowError> {
        let need_to_reload = if let Some(ref content) = self.file_viewer.content {
            content.storage.get_file_path() != path.as_ref()
        } else {
            true
        };
        if need_to_reload {
            let path_ref = path.as_ref();
            try!{self.load_in_file_viewer(path_ref).map_err(|e| PagerShowError::CouldNotOpenFile(path_ref.to_path_buf(), e))};
        }
        let line = line.into();
        self.file_viewer.go_to_line(line).map_err(|_| PagerShowError::LineDoesNotExist(line))
    }

    pub fn load_in_file_viewer<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file_storage = try!{FileLineStorage::new(path.as_ref())};
        let syntax = self.syntax_set.find_syntax_for_file(path.as_ref())
            .expect("file IS openable, see file storage")
            .unwrap_or(self.syntax_set.find_syntax_plain_text());
        self.file_viewer.load(
            PagerContent::create(file_storage)
            .with_highlighter(SyntectHighLighter::new(syntax, self.highlighting_theme))
            .with_decorator(LineNumberDecorator::default())
            );
        Ok(())
    }
    pub fn event(&mut self, event: Input, _ /*gdb*/: &mut gdbmi::GDB) {
        event.chain(ScrollBehavior::new(&mut self.file_viewer)
                    .forwards_on(Key::PageDown)
                    .backwards_on(Key::PageUp)
                   );
    }
}

impl<'a> Widget for SrcView<'a> {
    fn space_demand(&self) -> (Demand, Demand) {
        let widgets: Vec<&Widget> = vec![&self.asm_viewer, &self.file_viewer];
        self.layout.space_demand(widgets.as_slice())
    }
    fn draw(&mut self, window: Window) {
        let mut widgets: Vec<&mut Widget> = vec![&mut self.asm_viewer, &mut self.file_viewer];
        self.layout.draw(window, &mut widgets)
    }
}

