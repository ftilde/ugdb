use crate::gdb::{response::*, Address, BreakPoint, BreakpointOperationError, SrcPosition};
use crate::gdbmi::{
    commands::{BreakPointLocation, BreakPointNumber, DisassembleMode, MiCommand},
    output::{JsonValue, Object, ResultClass},
    ExecuteError,
};
use crate::Context;
use log::warn;
use std::{
    collections::HashSet,
    fs, io,
    ops::Range,
    path::{Path, PathBuf},
};
use unsegen::{
    base::{basic_types::*, Color, Cursor, GraphemeCluster, StyleModifier, Window},
    container::Container,
    input::{Input, Key, ScrollBehavior},
    widget::{
        text_width, ColDemand, Demand, Demand2D, HLayout, RenderingHints, RowDemand, VLayout,
        Widget, WidgetExt,
    },
};
use unsegen_pager::{
    LineDecorator, Pager, PagerContent, PagerError, PagerLine, SyntectHighlighter,
};
use unsegen_pager::{SyntaxSet, Theme};

#[derive(Debug)]
pub enum PagerShowError {
    CouldNotOpenFile(PathBuf, io::Error),
}

#[derive(Clone)]
struct AssemblyDebugLocation {
    func_name: String,
    offset: usize,
}

impl AssemblyDebugLocation {
    fn try_from_value(val: &JsonValue) -> Option<Self> {
        let func_name = val["func-name"].as_str()?;
        let offset = val["offset"].as_str()?.parse::<usize>().ok()?;
        Some(AssemblyDebugLocation {
            func_name: func_name.to_owned(),
            offset,
        })
    }
}

#[derive(Clone)]
struct AssemblyLine {
    content: String,
    address: Address,
    src_position: Option<SrcPosition>,
    debug_location: Option<AssemblyDebugLocation>,
}

impl AssemblyLine {
    const fn new(
        content: String,
        address: Address,
        src_position: Option<SrcPosition>,
        debug_location: Option<AssemblyDebugLocation>,
    ) -> Self {
        AssemblyLine {
            content,
            address,
            src_position,
            debug_location,
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
    fn new<'a, I: Iterator<Item = &'a BreakPoint>>(
        address_range: Range<Address>,
        stop_position: Option<Address>,
        breakpoints: I,
    ) -> Self {
        let addresses = breakpoints
            .filter_map(|bp| {
                bp.address.and_then(|addr| {
                    if bp.enabled && address_range.start <= addr && addr < address_range.end {
                        Some(addr)
                    } else {
                        None
                    }
                })
            })
            .collect();
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
            stop_position,
            breakpoint_addresses: addresses,
        }
    }
}

impl LineDecorator for AssemblyDecorator {
    type Line = AssemblyLine;
    fn horizontal_space_demand<'a, 'b: 'a>(
        &'a self,
        lines: impl DoubleEndedIterator<Item = (LineIndex, &'b Self::Line)> + 'b,
    ) -> ColDemand {
        let max_space = lines
            .last()
            .map(|(_, l)| text_width(format!(" 0x{:x} ", l.address.0).as_str()))
            .unwrap_or_else(|| Width::new(0).unwrap());
        Demand::exact(max_space)
    }
    fn decorate(
        &self,
        line: &Self::Line,
        current_line: LineIndex,
        active_line: LineIndex,
        mut window: Window,
    ) {
        let width = window.get_width();
        let mut cursor = Cursor::new(&mut window).position(ColIndex::new(0), RowIndex::new(0));

        let at_stop_position = self
            .stop_position
            .map(|p| p == line.address)
            .unwrap_or(false);
        let at_breakpoint_position = self.breakpoint_addresses.contains(&line.address);

        let (right_border, style_modifier) = match (at_stop_position, at_breakpoint_position) {
            (true, true) => ('▶', StyleModifier::new().fg_color(Color::Red).bold(true)),
            (true, false) => ('▶', StyleModifier::new().fg_color(Color::Green).bold(true)),
            (false, true) => ('●', StyleModifier::new().fg_color(Color::Red)),
            (false, false) => (' ', StyleModifier::new()),
        };

        cursor.set_style_modifier(style_modifier);

        use std::fmt::Write;
        if let (false, Some(offset)) = (
            current_line == active_line,
            line.debug_location
                .iter()
                .map(|l| l.offset)
                .find(|&offset| offset != 0),
        ) {
            let formatted_offset = format!("<+{}>", offset);
            write!(
                cursor,
                "{:>width$}{}",
                formatted_offset,
                right_border,
                width = (width - 1).positive_or_zero().into()
            )
            .unwrap();
        } else {
            write!(
                cursor,
                " 0x{:0>width$x}{}",
                line.address.0,
                right_border,
                width = (width - 4).positive_or_zero().into()
            )
            .unwrap();
        }
    }
}

pub struct AssemblyView<'a> {
    highlighting_theme: &'a Theme,
    syntax_set: SyntaxSet,
    pager: Pager<AssemblyLine, AssemblyDecorator>,
    last_stop_position: Option<Address>,
}

#[derive(Debug, derive_more::From)]
enum GotoError {
    NoLastStopPosition,
    MismatchedPagerContent,
    PagerError(PagerError),
}

impl<'a> AssemblyView<'a> {
    pub fn new(highlighting_theme: &'a Theme) -> Self {
        AssemblyView {
            highlighting_theme,
            syntax_set: SyntaxSet::load_defaults_nonewlines(),
            pager: Pager::new(),
            last_stop_position: None,
        }
    }
    fn set_last_stop_position(&mut self, pos: Address) {
        self.last_stop_position = Some(pos);
    }

    fn go_to_address(&mut self, pos: Address) -> Result<(), GotoError> {
        Ok(self.pager.go_to_line_if(|_, line| line.address == pos)?)
    }

    fn go_to_first_applicable_line<L: Into<LineNumber>>(
        &mut self,
        file: &Path,
        line: L,
    ) -> Result<(), GotoError> {
        let line: LineNumber = line.into();
        Ok(self.pager.go_to_line_if(|_, l| {
            if let Some(ref src_position) = l.src_position {
                src_position.file == file && src_position.line == line
            } else {
                false
            }
        })?)
    }

    fn go_to_last_stop_position(&mut self) -> Result<(), GotoError> {
        if let Some(address) = self.last_stop_position {
            self.go_to_address(address)
        } else {
            Err(GotoError::NoLastStopPosition)
        }
    }

    fn update_decoration(&mut self, p: &mut Context) {
        if let Some(ref mut content) = self.pager.content_mut() {
            let first_line_address = content.view_line(LineIndex::new(0)).map(|l| l.address);
            if let Some(min_address) = first_line_address {
                let max_address = {
                    content
                        .view(LineIndex::new(0)..)
                        .last()
                        .expect("We know we have at least one line")
                        .1
                        .address
                };
                content.set_decorator(AssemblyDecorator::new(
                    min_address..max_address,
                    self.last_stop_position,
                    p.gdb.breakpoints.values(),
                ));
            }
        }
    }

    fn show_lines(&mut self, lines: Vec<AssemblyLine>, p: &mut Context) {
        if lines.is_empty() {
            return; //Nothing to show
        }
        let min_address = lines.first().expect("We know lines is not empty").address;
        //TODO: use RangeInclusive when available on stable
        let max_address = lines.last().expect("We know lines is not empty").address + 1;

        let syntax = self
            .syntax_set
            .find_syntax_by_extension("s")
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let highlighter = SyntectHighlighter::new(syntax, self.highlighting_theme);
        self.pager.load(
            PagerContent::from_lines(lines)
                .with_highlighter(&highlighter)
                .with_decorator(AssemblyDecorator::new(
                    min_address..max_address,
                    self.last_stop_position,
                    p.gdb.breakpoints.values(),
                )),
        );
    }

    fn get_instructions(disass_results: &Object) -> Result<Vec<AssemblyLine>, GDBResponseError> {
        if let JsonValue::Array(line_objs) = &disass_results["asm_insns"] {
            let mut lines = Vec::<AssemblyLine>::new();
            for line_obj in line_objs {
                let line = LineNumber::new(
                    get_str(line_obj, "line")?
                        .parse::<usize>()
                        .map_err(|_| GDBResponseError::Other("Malformed line".into()))?,
                );

                let file = get_str(line_obj, "fullname")?;
                let src_pos = Some(SrcPosition::new(PathBuf::from(file), line));
                for tuple in line_obj["line_asm_insn"].members() {
                    let instruction = get_str(tuple, "inst")?;
                    let address = get_addr(tuple, "address")?;
                    lines.push(AssemblyLine::new(
                        instruction.to_owned(),
                        address,
                        src_pos.clone(),
                        AssemblyDebugLocation::try_from_value(tuple),
                    ));
                }
            }
            lines.sort_by_key(|l| l.address);
            Ok(lines)
        } else {
            Err(GDBResponseError::MissingField(
                "asm_insns",
                JsonValue::Object(disass_results.clone()),
            ))
        }
    }

    fn show_file<P: AsRef<Path>, L: Into<LineNumber>>(
        &mut self,
        file: P,
        line: L,
        p: &mut Context,
    ) -> Result<(), DisassembleError> {
        let line_u: usize = line.into().into();
        let disass_results = p
            .gdb
            .mi
            .execute(MiCommand::data_disassemble_file(
                file.as_ref(),
                line_u,
                None,
                DisassembleMode::MixedSourceAndDisassembly,
            ))?
            .results;

        let lines = Self::get_instructions(&disass_results)?;
        self.show_lines(lines, p);
        Ok(())
    }

    fn show_address(
        &mut self,
        address_start: Address,
        address_end: Address,
        p: &mut Context,
    ) -> Result<(), DisassembleError> {
        let line_objs = disassemble_address(address_start, address_end, p)?;

        let mut lines = Vec::<AssemblyLine>::new();
        for line_tuple in line_objs {
            let instruction = get_str(&line_tuple, "inst")?;
            let address = get_addr(&line_tuple, "address")?;
            lines.push(AssemblyLine::new(
                instruction.to_owned(),
                address,
                None,
                AssemblyDebugLocation::try_from_value(&line_tuple),
            ));
        }
        self.show_lines(lines, p);
        Ok(())
    }

    fn toggle_breakpoint(&self, p: &mut Context) {
        if let Some(line) = self.pager.current_line() {
            let active_bps: Vec<BreakPointNumber> = p
                .gdb
                .breakpoints
                .values()
                .filter_map(|bp| {
                    if let Some(ref address) = bp.address {
                        if *address == line.address {
                            Some(bp.number)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            if active_bps.is_empty() {
                match p
                    .gdb
                    .insert_breakpoint(BreakPointLocation::Address(line.address.0))
                {
                    Ok(()) => {}
                    Err(BreakpointOperationError::Busy) => {
                        p.log("Cannot insert breakpoint: Gdb is busy.");
                    }
                    Err(BreakpointOperationError::ExecutionError(msg)) => {
                        p.log(format!("Cannot insert breakpoint: {}", msg));
                    }
                }
            } else {
                match p.gdb.delete_breakpoints(active_bps.into_iter()) {
                    Ok(()) => {}
                    Err(BreakpointOperationError::Busy) => {
                        p.log("Cannot remove breakpoint: Gdb is busy.");
                    }
                    Err(BreakpointOperationError::ExecutionError(msg)) => {
                        p.log(format!("Cannot remove breakpoint: {}", msg));
                    }
                }
            }
        }
    }
    fn event(&mut self, event: Input, p: &mut Context) -> Option<Input> {
        event
            .chain(
                ScrollBehavior::new(&mut self.pager)
                    .forwards_on(Key::Down)
                    .forwards_on(Key::Char('j'))
                    .backwards_on(Key::Up)
                    .backwards_on(Key::Char('k'))
                    .to_beginning_on(Key::Home)
                    .to_end_on(Key::End),
            )
            .chain((Key::Char(' '), || self.toggle_breakpoint(p)))
            .finish()
    }
}

struct SourceDecorator {
    stop_position: Option<LineNumber>,
    breakpoint_lines: HashSet<LineNumber>,
}

impl SourceDecorator {
    fn new<'a, I: Iterator<Item = &'a BreakPoint>>(
        file: &Path,
        stop_position: Option<LineNumber>,
        breakpoints: I,
    ) -> Self {
        let addresses = breakpoints
            .filter_map(|bp| {
                bp.src_pos.clone().and_then(|pos| {
                    if bp.enabled && pos.file == file {
                        Some(pos.line)
                    } else {
                        None
                    }
                })
            })
            .collect();
        SourceDecorator {
            stop_position,
            breakpoint_lines: addresses,
        }
    }
}

impl LineDecorator for SourceDecorator {
    type Line = String;
    fn horizontal_space_demand<'a, 'b: 'a>(
        &'a self,
        lines: impl DoubleEndedIterator<Item = (LineIndex, &'b Self::Line)> + 'b,
    ) -> ColDemand {
        let max_space = lines
            .last()
            .map(|(i, _)| text_width(format!(" {} ", i).as_str()))
            .unwrap_or_else(|| Width::new(0).unwrap());
        Demand::exact(max_space)
    }
    fn decorate(
        &self,
        _: &Self::Line,
        current_index: LineIndex,
        _active_index: LineIndex,
        mut window: Window,
    ) {
        let width = (window.get_width() - 2).positive_or_zero();
        let line_number = LineNumber::from(current_index);
        let mut cursor = Cursor::new(&mut window).position(ColIndex::new(0), RowIndex::new(0));

        let at_stop_position = self
            .stop_position
            .map(|p| p == current_index.into())
            .unwrap_or(false);
        let at_breakpoint_position = self.breakpoint_lines.contains(&current_index.into());

        let (right_border, style_modifier) = match (at_stop_position, at_breakpoint_position) {
            (true, true) => ('▶', StyleModifier::new().fg_color(Color::Red).bold(true)),
            (true, false) => ('▶', StyleModifier::new().fg_color(Color::Green).bold(true)),
            (false, true) => ('●', StyleModifier::new().fg_color(Color::Red)),
            (false, false) => (' ', StyleModifier::new()),
        };

        cursor.set_style_modifier(style_modifier);

        use std::fmt::Write;
        write!(
            cursor,
            " {:width$}{}",
            line_number,
            right_border,
            width = width.into()
        )
        .unwrap();
    }
}

#[derive(Clone)]
struct FileInfo {
    path: PathBuf,
    modified: std::time::SystemTime,
}

pub struct SourceView<'a> {
    highlighting_theme: &'a Theme,
    syntax_set: SyntaxSet,
    pager: Pager<String, SourceDecorator>,
    file_info: Option<FileInfo>,
    last_stop_position: Option<SrcPosition>,
}

macro_rules! current_file_and_content_mut {
    ($x:expr) => {
        match (&$x.file_info, &mut $x.pager.content_mut()) {
            (&Some(ref file_info), &mut Some(ref mut content)) => Some((&file_info.path, content)),
            (&None, &mut None) => None,
            (&Some(_), &mut None) => panic!("Pager has file path, but no content"),
            (&None, &mut Some(_)) => panic!("Pager has content, but no file path"),
        }
    };
}

impl<'a> SourceView<'a> {
    pub fn new(highlighting_theme: &'a Theme) -> Self {
        SourceView {
            highlighting_theme,
            syntax_set: SyntaxSet::load_defaults_nonewlines(),
            pager: Pager::new(),
            file_info: None,
            last_stop_position: None,
        }
    }
    fn set_last_stop_position<P: AsRef<Path>>(&mut self, file: P, pos: LineNumber) {
        self.last_stop_position = Some(SrcPosition::new(file.as_ref().to_path_buf(), pos));
    }

    fn go_to_line<L: Into<LineNumber>>(&mut self, line: L) -> Result<(), GotoError> {
        Ok(self.pager.go_to_line(line.into())?)
    }

    fn go_to_last_stop_position(&mut self) -> Result<(), GotoError> {
        let line = if let Some(ref file_info) = self.file_info {
            if let Some(ref src_pos) = self.last_stop_position {
                if src_pos.file == file_info.path {
                    src_pos.line
                } else {
                    return Err(GotoError::MismatchedPagerContent);
                }
            } else {
                return Err(GotoError::NoLastStopPosition);
            }
        } else {
            return Err(GotoError::from(PagerError::NoContent));
        };

        self.go_to_line(line)
    }

    fn get_last_line_number_for<P: AsRef<Path>>(&self, file: P) -> Option<LineNumber> {
        self.last_stop_position.clone().and_then(|last_src_pos| {
            if file.as_ref() == last_src_pos.file {
                Some(last_src_pos.line)
            } else {
                None
            }
        })
    }

    fn update_decoration(&mut self, p: &mut Context) {
        if let Some((file_path, content)) = current_file_and_content_mut!(self) {
            // This sucks: we basically want to call get_last_line_number_for, but can't because we
            // borrowed content mutably...
            let last_line_number = self.last_stop_position.clone().and_then(|last_src_pos| {
                if last_src_pos.file == *file_path {
                    Some(last_src_pos.line)
                } else {
                    None
                }
            });
            content.set_decorator(SourceDecorator::new(
                file_path,
                last_line_number,
                p.gdb.breakpoints.values(),
            ));
        }
    }

    fn need_to_load_file(&self, path: &Path) -> bool {
        if let Some(ref loaded_file_info) = self.file_info {
            if loaded_file_info.path != path {
                return true;
            }
            if let Ok(modified_new) = fs::metadata(path).and_then(|m| m.modified()) {
                modified_new > loaded_file_info.modified
            } else {
                true
            }
        } else {
            true
        }
    }

    fn show<P: AsRef<Path>>(&mut self, path: P, p: &mut Context) -> Result<(), PagerShowError> {
        if self.need_to_load_file(path.as_ref()) {
            let path_ref = path.as_ref();
            self.load(path_ref, p.gdb.breakpoints.values())
                .map_err(|e| PagerShowError::CouldNotOpenFile(path_ref.to_path_buf(), e))?;
        } else {
            let last_line_number = self.get_last_line_number_for(path.as_ref());
            if let Some(ref mut content) = self.pager.content_mut() {
                content.set_decorator(SourceDecorator::new(
                    path.as_ref(),
                    last_line_number,
                    p.gdb.breakpoints.values(),
                ));
            }
        }
        Ok(())
    }

    fn reload(&mut self, p: &mut Context) -> Result<(), PagerShowError> {
        if let Some(i) = self.file_info.clone() {
            self.show(i.path, p)?;
        }
        Ok(())
    }

    fn content_is_stale(&self) -> bool {
        if let Some(i) = &self.file_info {
            self.need_to_load_file(&i.path)
        } else {
            true
        }
    }

    fn load<'b, P: AsRef<Path>, I: Iterator<Item = &'b BreakPoint>>(
        &mut self,
        path: P,
        breakpoints: I,
    ) -> io::Result<()> {
        let pager_content = PagerContent::from_file(path.as_ref())?;
        let syntax = self
            .syntax_set
            .find_syntax_for_file(path.as_ref())
            .expect("file IS openable, see pager content")
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let last_line_number = self.get_last_line_number_for(path.as_ref());
        let highlighter = SyntectHighlighter::new(syntax, self.highlighting_theme);
        self.pager
            .load(pager_content.with_highlighter(&highlighter).with_decorator(
                SourceDecorator::new(path.as_ref(), last_line_number, breakpoints),
            ));
        self.file_info = Some(FileInfo {
            path: path.as_ref().to_owned(),
            modified: fs::metadata(path)?.modified()?,
        });
        Ok(())
    }

    fn current_line_number(&self) -> LineNumber {
        self.pager.current_line_index().into()
    }

    fn current_file(&self) -> Option<&Path> {
        if let Some(ref file_info) = self.file_info {
            Some(&file_info.path)
        } else {
            None
        }
    }

    fn toggle_breakpoint(&self, p: &mut Context) {
        let line = self.current_line_number();
        if let Some(path) = self.current_file() {
            let active_bps: Vec<BreakPointNumber> = p
                .gdb
                .breakpoints
                .values()
                .filter_map(|bp| {
                    if let Some(ref src_pos) = bp.src_pos {
                        if src_pos.file == path && src_pos.line == line {
                            Some(bp.number)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            if active_bps.is_empty() {
                if p.gdb
                    .insert_breakpoint(BreakPointLocation::Line(path, line.into()))
                    .is_err()
                {
                    p.log("Cannot insert breakpoint: Gdb is busy.");
                }
            } else if p.gdb.delete_breakpoints(active_bps.into_iter()).is_err() {
                p.log("Cannot remove breakpoint: Gdb is busy.");
            }
        }
    }

    fn event(&mut self, event: Input, p: &mut Context) -> Option<Input> {
        event
            .chain(
                ScrollBehavior::new(&mut self.pager)
                    .forwards_on(Key::Down)
                    .forwards_on(Key::Char('j'))
                    .backwards_on(Key::Up)
                    .backwards_on(Key::Char('k'))
                    .to_beginning_on(Key::Home)
                    .to_end_on(Key::End),
            )
            .chain((Key::Char(' '), || self.toggle_breakpoint(p)))
            .finish()
    }
}

#[derive(Clone, PartialEq)]
enum DisplayMode {
    Source,
    Assembly,
    SideBySide,
    Message(String),
}
#[derive(Clone, PartialEq)]
enum SrcContentState {
    Available,
    Unavailable,
    NotYetLoaded(PathBuf),
}

#[derive(Clone, PartialEq)]
enum AsmContentState {
    Available,
    Unavailable,
    NotYetLoadedFile(PathBuf, LineIndex),
    NotYetLoadedAddr(Address, Address),
}

#[derive(Default)]
struct StackInfo {
    stack_level: Option<u64>,
    stack_depth: Option<u64>,
    file_path: Option<PathBuf>,
    function: Option<String>,
}

impl<'a> Widget for &'a StackInfo {
    fn space_demand(&self) -> Demand2D {
        Demand2D {
            // TODO: reenable this once configurable layouts are a thing.
            /*
            width: Demand::at_least(
                Width::new(
                    (self
                        .file_path
                        .as_ref()
                        .map(|p| p.to_string_lossy().len())
                        .unwrap_or(0)
                        + self.function.as_ref().map(|p| p.len()).unwrap_or(0))
                        as i32,
                )
                .unwrap(),
            ),
            */
            width: Demand::at_least(Width::new(1).unwrap()),
            height: Demand::exact(Height::new(1).unwrap()),
        }
    }
    fn draw(&self, mut window: Window, _hints: RenderingHints) {
        use std::fmt::Write;
        let width = window.get_width();
        let mut cursor = Cursor::new(&mut window).style_modifier(StyleModifier::new().bold(true));
        let _ = write!(cursor, "[");
        if let Some(l) = self.stack_level {
            let _ = write!(cursor, "{}", l);
        } else {
            let _ = write!(cursor, "?");
        }
        let _ = write!(cursor, "/");
        if let Some(l) = self.stack_depth {
            let _ = write!(cursor, "{}", l);
        } else {
            let _ = write!(cursor, "?");
        }
        let _ = write!(cursor, "] ");

        if let Some(f) = &self.function {
            let _ = write!(cursor, "{}", f);
        } else {
            let _ = write!(cursor, "?");
        }
        {
            let mut cursor = cursor.save().style_modifier();
            cursor.set_style_modifier(StyleModifier::new().bold(false));
            let _ = write!(cursor, " @ ");
        }

        if let Some(f) = &self.file_path {
            let path_str = f.to_string_lossy();
            let remaining_space =
                (width.raw_value() as usize).saturating_sub(cursor.get_col().raw_value() as _);
            if remaining_space >= text_width(path_str.as_ref()).raw_value() as _ {
                let _ = write!(cursor, "{}", path_str);
            } else {
                // Not enough space, only show file itself
                if let Some(n) = f.file_name() {
                    let _ = write!(cursor, "{}", n.to_string_lossy());
                } else {
                    let _ = write!(cursor, "?");
                }
            }
        } else {
            let _ = write!(cursor, "?");
        }
    }
}

#[derive(Clone, Debug, PartialEq, derive_more::From)]
#[allow(clippy::upper_case_acronyms)]
enum DisassembleError {
    GDB(GDBResponseError),
    Other(String),
}
impl From<ExecuteError> for DisassembleError {
    fn from(e: ExecuteError) -> Self {
        GDBResponseError::Execution(e).into()
    }
}

fn disassemble_address(
    address_start: Address,
    address_end: Address,
    p: &mut Context,
) -> Result<Vec<JsonValue>, DisassembleError> {
    let mut disass_results = match p.gdb.mi.execute(MiCommand::data_disassemble_address(
        address_start.0,
        address_end.0,
        DisassembleMode::DisassemblyOnly,
    )) {
        Ok(o) => {
            if o.class == ResultClass::Error {
                return Err(DisassembleError::Other(
                    o.results["msg"].as_str().unwrap_or("unknown").to_owned(),
                ));
            }
            o.results
        }
        Err(e) => {
            return Err(GDBResponseError::Execution(e).into());
        }
    };
    if let JsonValue::Array(line_objs) = disass_results["asm_insns"].take() {
        let mut line_objs = line_objs
            .into_iter()
            .map(|l| {
                let addr = get_addr(&l, "address")?;
                Ok((addr, l))
            })
            .collect::<Result<Vec<(Address, JsonValue)>, DisassembleError>>()?;
        //I'm not sure if GDB does this already, but we better not rely on it...
        line_objs.sort_by_key(|(a, _)| *a);

        Ok(line_objs.into_iter().map(|(_, o)| o).collect::<Vec<_>>())
    } else {
        Err(
            GDBResponseError::MissingField("asm_insns", JsonValue::Object(disass_results.clone()))
                .into(),
        )
    }
}

pub struct CodeWindow<'a> {
    src_view: SourceView<'a>,
    asm_view: AssemblyView<'a>,
    preferred_mode: DisplayMode,
    src_state: SrcContentState,
    asm_state: AsmContentState,
    last_bp_update: std::time::Instant,
    stack_info: StackInfo,
}

impl<'a> CodeWindow<'a> {
    pub fn new(highlighting_theme: &'a Theme, welcome_msg: &'static str) -> Self {
        CodeWindow {
            src_view: SourceView::new(highlighting_theme),
            asm_view: AssemblyView::new(highlighting_theme),
            preferred_mode: DisplayMode::Message(welcome_msg.to_owned()),
            src_state: SrcContentState::Unavailable,
            asm_state: AsmContentState::Unavailable,
            last_bp_update: std::time::Instant::now(),
            stack_info: Default::default(),
        }
    }

    fn available_display_mode(&self) -> DisplayMode {
        match (&self.preferred_mode, &self.src_state, &self.asm_state) {
            (DisplayMode::Message(msg), _, _) => DisplayMode::Message(msg.clone()),
            (DisplayMode::Source, SrcContentState::Available, _) => DisplayMode::Source,
            (DisplayMode::Source, _, AsmContentState::Available) => DisplayMode::Assembly,
            (DisplayMode::Assembly, _, AsmContentState::Available) => DisplayMode::Assembly,
            (DisplayMode::Assembly, SrcContentState::Available, _) => DisplayMode::Source,
            (DisplayMode::SideBySide, SrcContentState::Available, AsmContentState::Available) => {
                DisplayMode::SideBySide
            }
            (DisplayMode::SideBySide, SrcContentState::Available, _) => DisplayMode::Source,
            (DisplayMode::SideBySide, _, AsmContentState::Available) => DisplayMode::Assembly,
            (_, _, _) => DisplayMode::Message("Neither source nor assembly available!".to_owned()),
        }
    }

    fn try_load_source_content(&mut self, p: &mut Context) -> Result<(), PagerShowError> {
        match self.src_state.clone() {
            SrcContentState::NotYetLoaded(path) => {
                let ret = self.src_view.show(path, p);
                if ret.is_ok() {
                    self.src_state = SrcContentState::Available;
                } else {
                    self.src_state = SrcContentState::Unavailable;
                }
                ret
            }
            _ => {
                if self.src_view.content_is_stale() {
                    self.src_view.reload(p)?;
                }
                Ok(())
            }
        }
    }

    fn try_load_asm_content(&mut self, p: &mut Context) -> Result<(), DisassembleError> {
        match self.asm_state.clone() {
            AsmContentState::NotYetLoadedFile(path, line) => {
                let ret = self.asm_view.show_file(path, line, p);
                if ret.is_ok() {
                    self.asm_state = AsmContentState::Available;
                } else {
                    self.asm_state = AsmContentState::Unavailable;
                }
                ret
            }
            AsmContentState::NotYetLoadedAddr(begin, end) => {
                let ret = self.asm_view.show_address(begin, end, p);
                if ret.is_ok() {
                    self.asm_state = AsmContentState::Available;
                } else {
                    self.asm_state = AsmContentState::Unavailable;
                }
                ret
            }
            _ => Ok(()),
        }
    }

    fn try_load_active_content(&mut self, p: &mut Context) {
        let try_load_src = |s: &mut Self, p: &mut Context| {
            if let Err(e) = s.try_load_source_content(p) {
                warn!("Failed to load file: {:?}", e);
            }
        };
        let try_load_asm = |s: &mut Self, p: &mut Context| match s.try_load_asm_content(p) {
            Err(DisassembleError::GDB(GDBResponseError::Execution(ExecuteError::Busy))) => {
                p.log("Cannot disassemble: Gdb is busy.");
            }
            Err(e) => warn!("Failed to load assembly: {:?}", e),
            Ok(_) => {}
        };
        match self.preferred_mode {
            DisplayMode::SideBySide => {
                try_load_src(self, p);
                try_load_asm(self, p);
            }
            DisplayMode::Assembly => {
                try_load_asm(self, p);
                if self.asm_state == AsmContentState::Unavailable {
                    try_load_src(self, p);
                }
            }
            DisplayMode::Source => {
                try_load_src(self, p);
                if self.src_state == SrcContentState::Unavailable {
                    try_load_asm(self, p);
                }
            }
            DisplayMode::Message(_) => {}
        }
    }

    fn find_function_range(at: Address, p: &mut Context) -> Result<(Address, Address), ()> {
        let first_lines = disassemble_address(at, at + 16, p).map_err(|_| ())?;
        let current = first_lines.first().ok_or(())?;
        let asm_debug_location = AssemblyDebugLocation::try_from_value(current).ok_or(())?;
        let begin = at - asm_debug_location.offset;

        let block_size = 128;
        let mut current = at;
        let func_change_block = loop {
            let current_block_lines =
                disassemble_address(current, current + block_size, p).map_err(|_| ())?;
            {
                let penultimate_index = current_block_lines.len().checked_sub(2).ok_or(())?;
                let penultimate = current_block_lines
                    .get(penultimate_index)
                    .expect("We know penulatimate_index is valid");
                if let Some(penultimate_func_name) = penultimate["func-name"].as_str() {
                    if penultimate_func_name == asm_debug_location.func_name {
                        current = get_addr(penultimate, "address").map_err(|_| ())?;
                        continue;
                    }
                }
            }
            //func-name is None or different => we found our block
            break current_block_lines;
        };
        for line in func_change_block {
            if line["func-name"] != asm_debug_location.func_name {
                let end = get_addr(&line, "address").map_err(|_| ())?;
                return Ok((begin, end));
            }
        }
        unreachable!("func_change_block has to contain changing line");
    }
    fn find_valid_address_range(
        at: Address,
        approx_byte_size: usize,
        p: &mut Context,
    ) -> Result<(Address, Address), DisassembleError> {
        let block_lines = disassemble_address(at, at + approx_byte_size, p)?;

        let penultimate_index = block_lines
            .len()
            .checked_sub(2)
            .ok_or_else(|| DisassembleError::Other("Not enough lines".to_owned()))?;
        let penultimate = block_lines
            .get(penultimate_index)
            .ok_or_else(|| DisassembleError::Other("Not enough lines".to_owned()))?;
        let end_address = get_addr(penultimate, "address")?;
        Ok((at, end_address))
    }

    pub fn show_file(&mut self, file: String, line: LineNumber, p: &mut Context) {
        let mut object = Object::new();
        object.insert("fullname", JsonValue::String(file));
        object.insert("line", JsonValue::String(line.to_string()));
        self.show_frame(&object, p);
    }

    pub fn show_frame(&mut self, frame: &Object, p: &mut Context) {
        // Always try to switch away from (relatively unhelpful) message to srcview:
        if let DisplayMode::Message(_) = self.preferred_mode {
            self.preferred_mode = DisplayMode::Source;
        }

        self.src_state = SrcContentState::Unavailable;
        self.asm_state = AsmContentState::Unavailable;

        self.stack_info.stack_level = p.gdb.get_stack_level().ok();
        self.stack_info.stack_depth = p.gdb.get_stack_depth().ok();
        self.stack_info.file_path = frame["fullname"].as_str().map(PathBuf::from);
        self.stack_info.function = frame["func"].as_str().map(|s| s.to_owned());

        if let Some(path) = frame["fullname"].as_str() {
            let path = PathBuf::from(path);

            self.src_state = match self.src_view.current_file() {
                Some(f) if f == path => SrcContentState::Available,
                _ => SrcContentState::NotYetLoaded(path.clone()),
            };

            match get_u64_obj(frame, "line") {
                Ok(line) => {
                    let line = LineNumber::new(line as usize);

                    self.src_view.set_last_stop_position(path.clone(), line);

                    self.asm_state = if self
                        .asm_view
                        .go_to_first_applicable_line(&path, line)
                        .is_ok()
                    {
                        AsmContentState::Available
                    } else {
                        AsmContentState::NotYetLoadedFile(path, line.into())
                    };
                    match get_addr_obj(frame, "addr") {
                        Ok(address) => self.asm_view.set_last_stop_position(address),
                        Err(e) => warn!("Failed get address from frame: {:?}", e),
                    }
                }
                Err(e) => warn!("Failed get line from frame: {:?}", e),
            }
        };

        // If we were not able to load asm via file information, try loading from the address.
        // This may be the case for jit compiled code or PLT entries or something like that.
        if self.asm_state == AsmContentState::Unavailable {
            match get_addr_obj(frame, "addr") {
                Ok(address) => {
                    if self.asm_view.go_to_address(address).is_ok() {
                        self.asm_state = AsmContentState::Available;
                    } else {
                        match Self::find_function_range(address, p)
                            .or_else(|_| Self::find_valid_address_range(address, 128, p))
                        {
                            Ok((begin, end)) => {
                                self.asm_state = AsmContentState::NotYetLoadedAddr(begin, end)
                            }
                            Err(e) => {
                                warn!("Failed to disassemble from address {}: {:?}", address, e)
                            }
                        };
                    }
                    self.asm_view.set_last_stop_position(address);
                }
                Err(e) => warn!("Failed get address from frame: {:?}", e),
            }
        }

        self.try_load_active_content(p);
        let _ = self.asm_view.go_to_last_stop_position();
        let _ = self.src_view.go_to_last_stop_position();
        self.asm_view.update_decoration(p);
        self.src_view.update_decoration(p);
    }

    fn toggle_mode(&mut self, p: &mut Context) {
        let mut sync_asm_to_src = false;
        let prev_mode = self.preferred_mode.clone();
        self.preferred_mode = match prev_mode {
            DisplayMode::Assembly => DisplayMode::Source,
            DisplayMode::SideBySide => DisplayMode::Assembly,
            DisplayMode::Source => {
                sync_asm_to_src = true;
                DisplayMode::SideBySide
            }
            DisplayMode::Message(ref m) => DisplayMode::Message(m.clone()),
        };
        self.try_load_active_content(p);
        if self.available_display_mode() == prev_mode {
            // Disallow "blindly" changing the preferred mode if source/asm is not available.
            self.preferred_mode = prev_mode;
        } else if sync_asm_to_src {
            if let Some(path) = self.src_view.current_file() {
                if self
                    .asm_view
                    .go_to_first_applicable_line(path, self.src_view.current_line_number())
                    .is_err()
                    && self
                        .asm_view
                        .show_file(path, self.src_view.current_line_number(), p)
                        .is_ok()
                {
                    // The current line may not have associated assembly!
                    let _ = self
                        .asm_view
                        .go_to_first_applicable_line(path, self.src_view.current_line_number());
                }
            }
        }
    }

    fn try_switch_stackframe(&mut self, p: &mut Context, up: bool) -> Result<(), GDBResponseError> {
        let level = p.gdb.get_stack_level()?;

        let new_level = if up {
            let depth = p.gdb.get_stack_depth()?;
            (level + 1).min(depth.saturating_sub(1))
        } else {
            level.saturating_sub(1)
        };

        if level != new_level {
            p.gdb.mi.execute_later(MiCommand::select_frame(new_level));

            match p.gdb.mi.execute(MiCommand::stack_info_frame(None)) {
                Ok(o) => {
                    if o.class == ResultClass::Done {
                        if let JsonValue::Object(ref frame) = o.results["frame"] {
                            self.show_frame(frame, p);
                        } else {
                            return Err(GDBResponseError::MissingField(
                                "frame",
                                JsonValue::Object(o.results.clone()),
                            ));
                        }
                    } else {
                        return Err(GDBResponseError::Other(format!(
                            "Unexpected result class: {:?}",
                            o.class
                        )));
                    }
                }
                Err(_) => return Ok(()), //Ignore
            };
        }
        Ok(())
    }
    fn switch_stackframe(&mut self, p: &mut Context, up: bool) {
        match self.try_switch_stackframe(p, up) {
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to switch stackframe: {:?}", e);
            }
        }
    }

    pub fn update_after_event(&mut self, p: &mut Context) {
        if p.gdb.breakpoints.last_change > self.last_bp_update {
            self.asm_view.update_decoration(p);
            self.src_view.update_decoration(p);
            self.last_bp_update = p.gdb.breakpoints.last_change;
        }
    }
}

impl<'a> Container<Context> for CodeWindow<'a> {
    fn input(&mut self, input: Input, p: &mut Context) -> Option<Input> {
        input
            .chain((Key::Char('d'), || self.toggle_mode(p)))
            .chain((Key::PageUp, || self.switch_stackframe(p, true)))
            .chain((Key::PageDown, || self.switch_stackframe(p, false)))
            .chain(|i: Input| match self.available_display_mode() {
                DisplayMode::Assembly | DisplayMode::SideBySide => {
                    let ret = self.asm_view.event(i, p);
                    if let Some(src_pos) = self
                        .asm_view
                        .pager
                        .current_line()
                        .and_then(|line| line.src_position.clone())
                    {
                        self.src_state = SrcContentState::NotYetLoaded(src_pos.file);
                        self.try_load_active_content(p);
                        let _ = self.src_view.go_to_line(src_pos.line);
                    }
                    ret
                }
                DisplayMode::Source => self.src_view.event(i, p),
                DisplayMode::Message(_) => Some(i),
            })
            .finish()
    }
    fn as_widget<'e>(&'e self) -> Box<dyn Widget + 'e> {
        let mode = self.available_display_mode();

        let mut r = VLayout::new();
        if let DisplayMode::Assembly | DisplayMode::Source | DisplayMode::SideBySide = mode {
            r = r.widget(&self.stack_info)
        }
        r = match mode {
            DisplayMode::Assembly => r.widget(self.asm_view.pager.as_widget()),
            DisplayMode::SideBySide => r.widget(
                HLayout::new()
                    .separator(GraphemeCluster::try_from('|').unwrap())
                    .widget(self.asm_view.pager.as_widget())
                    .widget(self.src_view.pager.as_widget()),
            ),
            DisplayMode::Source => r.widget(self.src_view.pager.as_widget()),
            DisplayMode::Message(m) => r.widget(m.centered().with_demand(|d| Demand2D {
                width: ColDemand::at_least(d.width.min),
                height: RowDemand::at_least(d.height.min),
            })),
        };
        Box::new(r)
    }
}
