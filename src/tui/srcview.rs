use unsegen::base::{
    Cursor,
    Color,
    GraphemeCluster,
    StyleModifier,
    Window,
};
use unsegen::input::{
    Event,
    Input,
    Key,
    ScrollBehavior,
};
use unsegen::widget::{
    Demand,
    Demand2D,
    FileLineStorage,
    HorizontalLayout,
    LineNumber,
    LineIndex,
    LineStorage,
    MemoryLineStorage,
    RenderingHints,
    SeparatingStyle,
    Widget,
};
use unsegen::widget::widgets::{
    LineDecorator,
    Pager,
    PagerContent,
    PagerError,
    PagerLine,
    SyntectHighlighter,
};
use syntect::highlighting::{
    Theme,
};
use syntect::parsing::{
    SyntaxSet,
};
use gdbmi;
use gdbmi::output::{
    BreakPointEvent,
    JsonValue,
    Object,
    ResultClass,
};
use gdbmi::input::{
    MiCommand,
    BreakPointLocation,
    BreakPointNumber,
    DisassembleMode,
};
use std::io;
use std::path::{
    Path,
    PathBuf,
};
use std::collections::{
    HashMap,
    HashSet,
};
use std::ops::{
    Add,
    Range,
};
use std::fmt;

#[derive(Debug)]
pub enum PagerShowError {
    CouldNotOpenFile(PathBuf, io::Error),
    LineDoesNotExist(LineIndex),
}

#[derive(Debug, Clone)]
struct SrcPosition {
    file: PathBuf,
    line: LineNumber,
}

impl SrcPosition {
    fn new(file: PathBuf, line: LineNumber) -> Self {
        SrcPosition {
            file: file,
            line: line,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Address(usize);
impl Address {
    fn parse(string: &str) -> Result<Self, (::std::num::ParseIntError, String)> {
        usize::from_str_radix(&string[2..],16).map(|u| Address(u)).map_err(|e| (e, string.to_owned()))
    }
}
impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, " 0x{:0$x} ", self.0)
    }
}
impl Add<usize> for Address {
    type Output = Self;
    fn add(self, rhs: usize) -> Self {
        Address(self.0 + rhs)
    }
}

#[derive(Clone)]
struct AssemblyLine {
    content: String,
    address: Address,
    src_position: Option<SrcPosition>,
}

impl AssemblyLine {
    fn new(content: String, address: Address, src_position: Option<SrcPosition>) -> Self {
        AssemblyLine {
            content: content,
            address: address,
            src_position: src_position,
        }
    }
}

impl PagerLine for AssemblyLine {
    fn get_content(&self) -> &str {
        &self.content
    }
}

struct AssemblyDecorator {
    stop_position: Option<Address>,
    breakpoint_addresses: HashSet<Address>,
}

impl AssemblyDecorator {
    fn new<'a, I: Iterator<Item=&'a BreakPoint>>(address_range: Range<Address>, stop_position: Option<Address>, breakpoints: I) -> Self {
        let addresses = breakpoints.filter_map(|bp| {
            bp.address.and_then(|addr|
                if bp.enabled && address_range.start <= addr && addr < address_range.end {
                    Some(addr)
                } else {
                    None
                })
        }).collect();
        let stop_position = if let Some(p) = stop_position {
            if address_range.start <= p && p < address_range.end {
                Some(p)
            } else {
                None
            }
        } else {
            None
        };
        AssemblyDecorator {
            stop_position: stop_position,
            breakpoint_addresses: addresses,
        }
    }
}

impl LineDecorator for AssemblyDecorator {
    type Line = AssemblyLine;
    fn horizontal_space_demand<'a, 'b: 'a>(&'a self, lines: Box<DoubleEndedIterator<Item=(LineIndex, Self::Line)> + 'b>) -> Demand {
        let max_space = lines.last().map(|(_,l)| {
            ::unicode_width::UnicodeWidthStr::width(format!(" 0x{:x} ", l.address.0).as_str())
        }).unwrap_or(0);
        Demand::from_to(0, max_space as u32)
    }
    fn decorate(&self, line: &Self::Line, _: LineIndex, mut window: Window) {
        let width = window.get_width() as usize - 4;
        let mut cursor = Cursor::new(&mut window).position(0,0);

        use std::fmt::Write;
        let at_stop_position = self.stop_position.map(|p| p ==line.address).unwrap_or(false);
        let right_border = if at_stop_position {
            '▶'
        } else {
            ' '
        };
        let mut style_modifier = StyleModifier::none();
        if self.breakpoint_addresses.contains(&line.address) {
            style_modifier = StyleModifier::new().fg_color(Color::Red).on_top_of(&style_modifier);
        }
        if at_stop_position {
            style_modifier = StyleModifier::new().fg_color(Color::Green).bold(true).on_top_of(&style_modifier);
        }
        cursor.set_style_modifier(style_modifier);
        write!(cursor, " 0x{:0>width$x}{}", line.address.0, right_border, width=width).unwrap();
    }
}

pub struct AssemblyView<'a> {
    highlighting_theme: &'a Theme,
    syntax_set: SyntaxSet,
    pager: Pager<MemoryLineStorage<AssemblyLine>, SyntectHighlighter<'a>, AssemblyDecorator>,
    last_stop_position: Option<Address>,
}


impl<'a> AssemblyView<'a> {
    pub fn new(highlighting_theme: &'a Theme) -> Self {
        AssemblyView {
            highlighting_theme: highlighting_theme,
            syntax_set: SyntaxSet::load_defaults_nonewlines(),
            pager: Pager::new(),
            last_stop_position: None,
        }
    }
    fn set_last_stop_position(&mut self, pos: Address) {
        self.last_stop_position = Some(pos);
    }

    fn go_to_address(&mut self, pos: Address) -> Result<(), PagerError> {
        self.pager.go_to_line_if(|_, line| line.address == pos)
    }

    fn go_to_first_applicable_line<L: Into<LineNumber>>(&mut self, file: &Path, line: L) -> Result<(), PagerError> {
        let line: LineNumber = line.into();
        self.pager.go_to_line_if(|_, l| {
            if let Some(ref src_position) = l.src_position {
                src_position.file == file && src_position.line == line
            } else {
                false
            }
        })
    }

    fn go_to_last_stop_position(&mut self) -> Result<(), PagerError> {
        if let Some(address) = self.last_stop_position {
            self.go_to_address(address)
        } else {
            Err(PagerError::LineDoesNotExist)
        }
    }

    fn update_decoration<'b, I: Iterator<Item=&'b BreakPoint>>(&mut self, breakpoints: I) {
        if let Some(ref mut content) = self.pager.content {
            if let Some(first_line) = content.storage.view_line(LineIndex(0)) {
                let min_address = first_line.address;
                let max_address = content.storage.view(LineIndex(0)..).last().expect("we know we have at least one line").1.address;
                content.decorator = AssemblyDecorator::new(min_address..max_address, self.last_stop_position, breakpoints)
            }
        }
    }

    fn show<'b, P: AsRef<Path>, L: Into<LineNumber>, I: Iterator<Item=&'b BreakPoint>>(&mut self, file: P, line: L, breakpoints: I, gdb: &mut gdbmi::GDB) -> Result<(), () /* Disassembly unsuccessful */> {
        let line_u: usize = line.into().into();
        let ref disass_object = gdb.execute(MiCommand::data_disassemble_file(file, line_u, None, DisassembleMode::MixedSourceAndDisassembly)).expect("disassembly successful").results["asm_insns"];
        if let &JsonValue::Array(ref line_objs) = disass_object {
            let mut asm_storage = MemoryLineStorage::<AssemblyLine>::new();
            for line_obj in line_objs {
                let line = LineNumber(line_obj["line"].as_str().expect("line present").parse::<usize>().expect("parse line"));
                let file = line_obj["fullname"].as_str().expect("full name present");
                let src_pos = Some(SrcPosition::new(PathBuf::from(file), line));
                for tuple in line_obj["line_asm_insn"].members() {
                    let instruction = tuple["inst"].as_str().expect("No instruction in disassembly object");
                    let address_str = tuple["address"].as_str().expect("No address in disassembly object");
                    let address = Address::parse(address_str).expect("Parse address");
                    asm_storage.lines.push(AssemblyLine::new(instruction.to_owned(), address, src_pos.clone()));
                }
            }
            asm_storage.lines.sort_by_key(|l| l.address);
            let min_address = asm_storage.lines.first().expect("At least one instruction").address;
            //TODO: use RangeInclusive when available on stable
            let max_address = asm_storage.lines.last().expect("At least one instruction").address + 1;

            let syntax = self.syntax_set.find_syntax_by_extension("s")
                .unwrap_or(self.syntax_set.find_syntax_plain_text());
            self.pager.load(
                PagerContent::create(asm_storage)
                .with_highlighter(SyntectHighlighter::new(syntax, self.highlighting_theme))
                .with_decorator(AssemblyDecorator::new(min_address..max_address, self.last_stop_position, breakpoints)));
            Ok(())
        } else {
            // Disassembly object is not an array:
            // There may be no asm correspondence for the give file and line.
            return Err(());
        }
    }
    fn toggle_breakpoint(&self, gdb: &mut gdbmi::GDB, breakpoints: &mut BreakPointSet) {
        if let Some(line) = self.pager.current_line() {
            let active_bps: Vec<BreakPointNumber> = breakpoints.values().filter_map(|bp| if let Some(ref address) = bp.address {
                if *address == line.address {
                    Some(bp.number)
                } else {
                    None
                }
            } else {
                None
            }).collect();
            if active_bps.is_empty() {
                let bp_result = gdb.execute(&MiCommand::insert_breakpoint(BreakPointLocation::Address(line.address.0))).expect("insert successful");
                match bp_result.class {
                    ResultClass::Done => {
                        for bp in BreakPoint::all_from_json(&bp_result.results["bkpt"]) {
                            breakpoints.update_breakpoint(bp)
                        }
                    },
                    ResultClass::Error => {
                        // Cannot create breakpoint
                        // TODO: display error msg somehow?
                    },
                    _ => {
                        panic!("Unexpected result class");
                    },
                }
            } else {
                for &number in active_bps.iter() {
                    breakpoints.remove_breakpoint(number);
                }
                let bp_result = gdb.execute(MiCommand::delete_breakpoints(active_bps.into_iter())).expect("path-breakpoint insert successful");
                debug_assert!(bp_result.class == ResultClass::Done, "Incorrect result class");
            }
        }
    }
    fn event(&mut self, event: Input, gdb: &mut gdbmi::GDB, breakpoints: &mut BreakPointSet) {
        event.chain(ScrollBehavior::new(&mut self.pager)
                    .forwards_on(Key::PageDown)
                    .forwards_on(Key::Char('j'))
                    .backwards_on(Key::PageUp)
                    .backwards_on(Key::Char('k'))
                   )
            .chain(|evt| match evt {
                Input { event: Event::Key(Key::Char(' ')) } => {
                    self.toggle_breakpoint(gdb, breakpoints);
                    None
                }
                e => Some(e)
            });
    }
}

impl<'a> Widget for AssemblyView<'a> {
    fn space_demand(&self) -> Demand2D {
        self.pager.space_demand()
    }
    fn draw(&mut self, window: Window, hints: RenderingHints) {
        self.pager.draw(window, hints)
    }
}

struct SourceDecorator {
    stop_position: Option<LineNumber>,
    breakpoint_lines: HashSet<LineNumber>,
}

impl SourceDecorator {
    fn new<'a, I: Iterator<Item=&'a BreakPoint>>(file: &Path, stop_position: Option<LineNumber>, breakpoints: I) -> Self {
        let addresses = breakpoints.filter_map(|bp| {
            bp.src_pos.clone().and_then(|pos| {
                if bp.enabled && pos.file == file {
                    Some(pos.line)
                } else {
                    None
                }
            })
        }).collect();
        SourceDecorator {
            stop_position: stop_position,
            breakpoint_lines: addresses,
        }
    }
}

impl LineDecorator for SourceDecorator {
    type Line = String;
    fn horizontal_space_demand<'a, 'b: 'a>(&'a self, lines: Box<DoubleEndedIterator<Item=(LineIndex, Self::Line)> + 'b>) -> Demand {
        let max_space = lines.last().map(|(i,_)| {
            ::unicode_width::UnicodeWidthStr::width(format!(" {} ", i).as_str())
        }).unwrap_or(0);
        Demand::from_to(0, max_space as u32)
    }
    fn decorate(&self, _: &Self::Line, index: LineIndex, mut window: Window) {
        let width = window.get_width() as usize - 2;
        let line_number = LineNumber::from(index);
        let mut cursor = Cursor::new(&mut window).position(0,0);

        let at_stop_position = self.stop_position.map(|p| p == index.into()).unwrap_or(false);
        let right_border = if at_stop_position {
            '▶'
        } else {
            ' '
        };

        let mut style_modifier = StyleModifier::none();
        if self.breakpoint_lines.contains(&index.into()) {
            style_modifier = StyleModifier::new().fg_color(Color::Red).on_top_of(&style_modifier);
        }
        if at_stop_position {
            style_modifier = StyleModifier::new().fg_color(Color::Green).bold(true).on_top_of(&style_modifier);
        }
        cursor.set_style_modifier(style_modifier);

        use std::fmt::Write;
        write!(cursor, " {:width$}{}", line_number, right_border, width = width).unwrap();
    }
}

pub struct SourceView<'a> {
    highlighting_theme: &'a Theme,
    syntax_set: SyntaxSet,
    pager: Pager<FileLineStorage, SyntectHighlighter<'a>, SourceDecorator>,
    last_stop_position: Option<SrcPosition>,
}

impl<'a> SourceView<'a> {
    pub fn new(highlighting_theme: &'a Theme) -> Self {
        SourceView {
            highlighting_theme: highlighting_theme,
            syntax_set: SyntaxSet::load_defaults_nonewlines(),
            pager: Pager::new(),
            last_stop_position: None,
        }
    }
    fn set_last_stop_position<P: AsRef<Path>>(&mut self, file: P, pos: LineNumber) {
        self.last_stop_position = Some(SrcPosition::new(file.as_ref().to_path_buf(), pos));
    }

    fn go_to_line<L: Into<LineNumber>>(&mut self, line: L) -> Result<(), PagerError> {
        self.pager.go_to_line(line.into())
    }

    fn go_to_last_stop_position(&mut self) -> Result<(), PagerError> {
        let (same_file, line) = if let Some(ref content) = self.pager.content {
            if let Some(ref src_pos) = self.last_stop_position {
                (src_pos.file == content.storage.get_file_path(), src_pos.line)
            } else {
                return Err(PagerError::LineDoesNotExist)
            }
        } else {
            return Err(PagerError::LineDoesNotExist)
        };
        if same_file {
            self.go_to_line(line)
        } else {
            Err(PagerError::LineDoesNotExist)
        }
    }

    fn get_last_line_number_for<P: AsRef<Path>>(&self, file: P) -> Option<LineNumber>{
        self.last_stop_position.clone().and_then(|last_src_pos| {
            if file.as_ref() == last_src_pos.file {
                Some(last_src_pos.line)
            } else {
                None
            }
        })
    }

    fn update_decoration<'b, I: Iterator<Item=&'b BreakPoint>>(&mut self, breakpoints: I) {
        if let Some(ref mut content) = self.pager.content {
            let path = content.storage.get_file_path();

            // This sucks: we basically want to call get_last_line_number_for, but can't because we
            // borrowed content mutably...
            let last_line_number = self.last_stop_position.clone().and_then(|last_src_pos| {
                if path == last_src_pos.file {
                    Some(last_src_pos.line)
                } else {
                    None
                }
            });
            content.decorator = SourceDecorator::new(path, last_line_number, breakpoints)
        }
    }

    fn show<'b, P: AsRef<Path>, L: Into<LineIndex>, I: Iterator<Item=&'b BreakPoint>>(&mut self, path: P, line: L, breakpoints: I) -> Result<(), PagerShowError> {
        let need_to_reload = if let Some(ref content) = self.pager.content {
            content.storage.get_file_path() != path.as_ref()
        } else {
            true
        };
        if need_to_reload {
            let path_ref = path.as_ref();
            try!{self.load(path_ref, breakpoints).map_err(|e| PagerShowError::CouldNotOpenFile(path_ref.to_path_buf(), e))};
        } else {
            let last_line_number = self.get_last_line_number_for(path.as_ref());
            if let Some(ref mut content) = self.pager.content {
                content.decorator = SourceDecorator::new(path.as_ref(), last_line_number, breakpoints);
            }
        }
        let line = line.into();
        self.pager.go_to_line(line).map_err(|_| PagerShowError::LineDoesNotExist(line))
    }

    fn load<'b, P: AsRef<Path>, I: Iterator<Item=&'b BreakPoint>>(&mut self, path: P, breakpoints: I) -> io::Result<()> {
        let file_storage = try!{FileLineStorage::new(path.as_ref())};
        let syntax = self.syntax_set.find_syntax_for_file(path.as_ref())
            .expect("file IS openable, see file storage")
            .unwrap_or(self.syntax_set.find_syntax_plain_text());
        let last_line_number = self.get_last_line_number_for(path.as_ref());
        self.pager.load(
            PagerContent::create(file_storage)
            .with_highlighter(SyntectHighlighter::new(syntax, self.highlighting_theme))
            .with_decorator(SourceDecorator::new(path.as_ref(), last_line_number, breakpoints))
            );
        Ok(())
    }

    fn current_line_number(&self) -> LineNumber {
        self.pager.current_line_index().into()
    }

    fn current_file(&self) -> Option<&Path> {
        if let Some(ref content) = self.pager.content {
            Some(content.storage.get_file_path())
        } else {
            None
        }
    }

    fn toggle_breakpoint(&self, gdb: &mut gdbmi::GDB, breakpoints: &mut BreakPointSet) {
        let line = self.current_line_number();
        if let Some(path) = self.current_file() {
            let active_bps: Vec<BreakPointNumber> = breakpoints.values().filter_map(|bp| if let Some(ref src_pos) = bp.src_pos {
                if src_pos.file == path && src_pos.line == line {
                    Some(bp.number)
                } else {
                    None
                }
            } else {
                None
            }).collect();
            if active_bps.is_empty() {
                let bp_result = gdb.execute(MiCommand::insert_breakpoint(BreakPointLocation::Line(path, line.into()))).expect("path-breakpoint insert successful");
                match bp_result.class {
                    ResultClass::Done => {
                        for bp in BreakPoint::all_from_json(&bp_result.results["bkpt"]) {
                            breakpoints.update_breakpoint(bp)
                        }
                    },
                    ResultClass::Error => {
                        // Cannot create breakpoint
                        // TODO: display error msg somehow?
                    },
                    _ => {
                        panic!("Unexpected result class");
                    },
                }
            } else {
                for &number in active_bps.iter() {
                    breakpoints.remove_breakpoint(number);
                }
                let bp_result = gdb.execute(MiCommand::delete_breakpoints(active_bps.into_iter())).expect("path-breakpoint insert successful");
                debug_assert!(bp_result.class == ResultClass::Done, "Incorrect result class");
            }
        }
    }

    fn event(&mut self, event: Input, gdb: &mut gdbmi::GDB, breakpoints: &mut BreakPointSet) {
        event.chain(ScrollBehavior::new(&mut self.pager)
                    .forwards_on(Key::PageDown)
                    .forwards_on(Key::Char('j'))
                    .backwards_on(Key::PageUp)
                    .backwards_on(Key::Char('k'))
                   )
            .chain(|evt| match evt {
                Input { event: Event::Key(Key::Char(' ')) } => {
                    self.toggle_breakpoint(gdb, breakpoints);
                    None
                }
                e => Some(e)
            });
    }
}

impl<'a> Widget for SourceView<'a> {
    fn space_demand(&self) -> Demand2D {
        self.pager.space_demand()
    }
    fn draw(&mut self, window: Window, hints: RenderingHints) {
        self.pager.draw(window, hints)
    }
}

struct BreakPoint {
    number: BreakPointNumber,
    address: Option<Address>,
    enabled: bool,
    src_pos: Option<SrcPosition>, // May not be present if debug information is missing!
}


impl BreakPoint {
    fn from_json(bkpt: &Object) -> Self {
        let number = bkpt["number"].as_str().expect("find bp number").parse::<BreakPointNumber>().expect("Parse usize");
        let enabled = bkpt["enabled"].as_str().expect("find enabled") == "y";
        let address = bkpt["addr"].as_str().and_then(|addr| Address::parse(addr).ok()); //addr may not be present or contain
        let src_pos = {
            let maybe_file = bkpt["fullname"].as_str();
            let maybe_line = bkpt["line"].as_str().map(|l_nr| LineNumber(l_nr.parse::<usize>().expect("Parse usize")));
            if let (Some(file), Some(line)) = (maybe_file, maybe_line) {
                Some(SrcPosition::new(PathBuf::from(file), line))
            } else {
                None
            }
        };
        BreakPoint {
            number: number,
            address: address,
            enabled: enabled,
            src_pos: src_pos,
        }
    }

    fn all_from_json(bkpt_obj: &JsonValue) -> Box<Iterator<Item=BreakPoint>> {
        match bkpt_obj {
            &JsonValue::Object(ref bp) => {
                Box::new(Some(Self::from_json(&bp)).into_iter())
            },
            &JsonValue::Array(ref bp_array) => {
                Box::new(bp_array.iter().map(|bp| {
                    if let &JsonValue::Object(ref bp) = bp {
                        Self::from_json(&bp)
                    } else {
                        panic!("Invalid breakpoint object in array");
                    }
                }).collect::<Vec<BreakPoint>>().into_iter())
            },
            _ => {
                panic!("Invalid breakpoint object")
            },
        }
    }
}

struct BreakPointSet {
    breakpoints: HashMap<BreakPointNumber, BreakPoint>,
    changed: bool,
}

impl BreakPointSet {
    fn new() -> Self {
        BreakPointSet {
            breakpoints: HashMap::new(),
            changed: false,
        }
    }

    fn update_breakpoint(&mut self, new_bp: BreakPoint) {
        let _ = self.breakpoints.insert(new_bp.number, new_bp);
        //debug_assert!(res.is_some(), "Modified non-existent breakpoint");
        self.changed = true;
    }

    fn remove_breakpoint(&mut self, bp_num: BreakPointNumber) {
        self.breakpoints.remove(&bp_num);
        self.changed = true;
    }
}

impl ::std::ops::Deref for BreakPointSet {
    type Target = HashMap<BreakPointNumber, BreakPoint>;

    fn deref(&self) -> &Self::Target {
        &self.breakpoints
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
    breakpoints: BreakPointSet,
}

impl<'a> CodeWindow<'a> {
    pub fn new(highlighting_theme: &'a Theme) -> Self {
        CodeWindow {
            src_view: SourceView::new(highlighting_theme),
            asm_view: AssemblyView::new(highlighting_theme),
            layout: HorizontalLayout::new(SeparatingStyle::Draw(GraphemeCluster::try_from('|').unwrap())),
            mode: CodeWindowMode::Source,
            breakpoints: BreakPointSet::new(),
        }
    }
    pub fn show_frame(&mut self, frame: &Object, gdb: &mut gdbmi::GDB) {
        if let Some(path) = frame["fullname"].as_str() { // File information may not be present
            let line = LineNumber(frame["line"].as_str().expect("line present").parse::<usize>().expect("Parse usize"));
            let address = Address::parse(frame["addr"].as_str().expect("address present")).expect("Parse address");
            self.src_view.set_last_stop_position(path, line);
            // GDB may give out invalid paths, so we just ignore them (at least for now)
            if self.src_view.show(path, line, self.breakpoints.values()).is_ok() {
                self.src_view.go_to_last_stop_position().expect("We just set a last stop pos!");
            }

            self.asm_view.set_last_stop_position(address);
            if self.asm_view.show(path, line, self.breakpoints.values(), gdb).is_err() {
                self.mode = CodeWindowMode::Source;
            } else {
                self.asm_view.go_to_last_stop_position().expect("We just set a last stop pos and it must be valid!");
            }
        }
    }

    pub fn handle_breakpoint_event(&mut self, bp_type: BreakPointEvent, info: &Object) {
        match bp_type {
            BreakPointEvent::Created | BreakPointEvent::Modified => {
                if let JsonValue::Object(ref bkpt) = info["bkpt"] {
                    let bp = BreakPoint::from_json(bkpt);
                    self.breakpoints.update_breakpoint(bp);
                    //debug_assert!(bp_type != BreakPointEvent::Modified || res.is_some(), "Modified non-existent id");
                } else {
                    panic!("Invalid bkpt");
                }
            },
            BreakPointEvent::Deleted => {
                let id = info["id"].as_str().expect("find id").parse::<BreakPointNumber>().expect("Parse usize");
                self.breakpoints.remove_breakpoint(id);
            },
        }
        self.synchronize_breakpoints();
    }

    fn toggle_mode(&mut self, gdb: &mut gdbmi::GDB) {
        self.mode = match self.mode {
            CodeWindowMode::Assembly => {
                CodeWindowMode::Source
            },
            CodeWindowMode::Source => {
                if let Some(path) = self.src_view.current_file() {
                    if self.asm_view.show(path, self.src_view.current_line_number(), self.breakpoints.values(), gdb).is_ok() {

                        // The current line may not have associated assembly!
                        // TODO: Maybe we want to try the next line or something...
                        let _ = self.asm_view.go_to_first_applicable_line(path, self.src_view.current_line_number());

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

    fn synchronize_breakpoints(&mut self) {
        if self.breakpoints.changed {
            self.asm_view.update_decoration(self.breakpoints.values());
            self.src_view.update_decoration(self.breakpoints.values());
            self.breakpoints.changed = false;
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
                CodeWindowMode::Assembly => {
                    self.asm_view.event(i, gdb, &mut self.breakpoints);
                    if let Some(src_pos) = self.asm_view.pager.current_line().and_then(|line| line.src_position) {
                        let _  = self.src_view.show(src_pos.file, src_pos.line, self.breakpoints.values());
                    }
                },
                CodeWindowMode::Source => {
                    self.src_view.event(i, gdb, &mut self.breakpoints);
                },
            }
            None
        });
        self.synchronize_breakpoints();
    }
}

impl<'a> Widget for CodeWindow<'a> {
    fn space_demand(&self) -> Demand2D {
        let widgets: Vec<&Widget> = match self.mode {
            CodeWindowMode::Assembly => vec![&self.asm_view, &self.src_view],
            CodeWindowMode::Source => vec![&self.src_view],
        };
        self.layout.space_demand(widgets.as_slice())
    }
    fn draw(&mut self, window: Window, hints: RenderingHints) {
        let mut widgets: Vec<(&mut Widget, RenderingHints)> = match self.mode {
            CodeWindowMode::Assembly => vec![(&mut self.asm_view, hints), (&mut self.src_view, hints)],
            CodeWindowMode::Source => vec![(&mut self.src_view, hints)],
        };
        self.layout.draw(window, &mut widgets)
    }
}

