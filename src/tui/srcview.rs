use gdb::{Address, BreakPoint, BreakpointOperationError, SrcPosition};
use gdbmi::commands::{BreakPointLocation, BreakPointNumber, DisassembleMode, MiCommand};
use gdbmi::output::{JsonValue, Object, ResultClass};
use gdbmi::ExecuteError;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};
use unsegen::base::basic_types::*;
use unsegen::base::{Color, Cursor, GraphemeCluster, StyleModifier, Window};
use unsegen::container::Container;
use unsegen::input::{Input, Key, ScrollBehavior};
use unsegen::widget::{
    text_width, ColDemand, Demand, Demand2D, HorizontalLayout, RenderingHints, SeparatingStyle,
    Widget,
};
use unsegen_pager::{
    LineDecorator, Pager, PagerContent, PagerError, PagerLine, SyntectHighlighter,
};
use unsegen_pager::{SyntaxSet, Theme};

#[derive(Debug)]
pub enum PagerShowError {
    CouldNotOpenFile(PathBuf, io::Error),
    LineDoesNotExist(LineIndex),
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
    fn new(
        content: String,
        address: Address,
        src_position: Option<SrcPosition>,
        debug_location: Option<AssemblyDebugLocation>,
    ) -> Self {
        AssemblyLine {
            content: content,
            address: address,
            src_position: src_position,
            debug_location: debug_location,
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
            stop_position: stop_position,
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
            .unwrap_or(Width::new(0).unwrap());
        Demand::from_to(0, max_space.into())
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
            (true, false) => (
                '▶',
                StyleModifier::new().fg_color(Color::Green).bold(true),
            ),
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
                .filter(|&offset| offset != 0)
                .next(),
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

#[derive(Debug)]
enum GotoError {
    NoLastStopPosition,
    MismatchedPagerContent,
    PagerError(PagerError),
}

impl From<PagerError> for GotoError {
    fn from(e: PagerError) -> Self {
        GotoError::PagerError(e)
    }
}

fn get_str<'a>(obj: &'a JsonValue, key: &'static str) -> Result<&'a str, GDBResponseError> {
    Ok(obj[key]
        .as_str()
        .ok_or_else(|| GDBResponseError::MissingField(key, obj.clone()))?)
}

fn get_str_obj<'a>(obj: &'a Object, key: &'static str) -> Result<&'a str, GDBResponseError> {
    Ok(obj[key]
        .as_str()
        .ok_or_else(|| GDBResponseError::MissingField(key, JsonValue::Object(obj.clone())))?)
}

fn get_addr<'a>(obj: &'a JsonValue, key: &'static str) -> Result<Address, GDBResponseError> {
    let s = get_str(obj, key)?;
    Ok(Address::parse(s)?)
}

fn get_addr_obj<'a>(obj: &'a Object, key: &'static str) -> Result<Address, GDBResponseError> {
    let s = get_str_obj(obj, key)?;
    Ok(Address::parse(s)?)
}

fn get_u64<'a>(obj: &'a JsonValue, key: &'static str) -> Result<u64, GDBResponseError> {
    let s = get_str(obj, key)?;
    Ok(s.parse::<u64>()
        .map_err(|_| GDBResponseError::Other(format!("Malformed frame description")))?)
}

fn get_u64_obj<'a>(obj: &'a Object, key: &'static str) -> Result<u64, GDBResponseError> {
    let s = get_str_obj(obj, key)?;
    Ok(s.parse::<u64>()
        .map_err(|_| GDBResponseError::Other(format!("Malformed frame description")))?)
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

    fn update_decoration(&mut self, p: ::UpdateParameters) {
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

    fn show_lines(&mut self, lines: Vec<AssemblyLine>, p: ::UpdateParameters) {
        if lines.is_empty() {
            return; //Nothing to show
        }
        let min_address = lines.first().expect("We know lines is not empty").address;
        //TODO: use RangeInclusive when available on stable
        let max_address = lines.last().expect("We know lines is not empty").address + 1;

        let syntax = self
            .syntax_set
            .find_syntax_by_extension("s")
            .unwrap_or(self.syntax_set.find_syntax_plain_text());
        self.pager.load(
            PagerContent::from_lines(lines)
                .with_highlighter(&SyntectHighlighter::new(syntax, self.highlighting_theme))
                .with_decorator(AssemblyDecorator::new(
                    min_address..max_address,
                    self.last_stop_position,
                    p.gdb.breakpoints.values(),
                )),
        );
    }

    fn get_instructions(disass_results: &Object) -> Result<Vec<AssemblyLine>, GDBResponseError> {
        if let &JsonValue::Array(ref line_objs) = &disass_results["asm_insns"] {
            let mut lines = Vec::<AssemblyLine>::new();
            for line_obj in line_objs {
                let line = LineNumber::new(
                    get_str(&line_obj, "line")?
                        .parse::<usize>()
                        .map_err(|_| GDBResponseError::Other(format!("Malformed line")))?,
                );

                let file = get_str(&line_obj, "fullname")?;
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
        p: ::UpdateParameters,
    ) -> Result<(), () /* Disassembly unsuccessful */> {
        let line_u: usize = line.into().into();
        let disass_results = match {
            p.gdb.mi.execute(MiCommand::data_disassemble_file(
                file.as_ref(),
                line_u,
                None,
                DisassembleMode::MixedSourceAndDisassembly,
            ))
        } {
            Ok(o) => o.results,
            Err(ExecuteError::Busy) => {
                // that's okay, we will try again next time. This may occur if the user is
                // hammering "n", the disassembly has not yet finished, but gdb is already
                // executing the next step.
                return Err(());
            }
            Err(ExecuteError::Quit) => {
                // If GDB has quit the ugdb will shut down soon as well
                return Err(());
            }
        };

        match Self::get_instructions(&disass_results) {
            Ok(lines) => {
                self.show_lines(lines, p);
                Ok(())
            }
            Err(e) => {
                p.logger.log_message(format!(
                    "Disassembly failed for {:?}:{}: {:?}",
                    file.as_ref(),
                    line_u,
                    e
                ));
                Err(())
            }
        }
    }

    fn show_address(
        &mut self,
        address_start: Address,
        address_end: Address,
        p: ::UpdateParameters,
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

    fn toggle_breakpoint(&self, p: ::UpdateParameters) {
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
                        p.logger
                            .log_message("Cannot insert breakpoint: Gdb is busy.");
                    }
                    Err(BreakpointOperationError::ExecutionError(msg)) => {
                        p.logger
                            .log_message(format!("Cannot insert breakpoint: {}", msg));
                    }
                }
            } else {
                match p.gdb.delete_breakpoints(active_bps.into_iter()) {
                    Ok(()) => {}
                    Err(BreakpointOperationError::Busy) => {
                        p.logger
                            .log_message("Cannot remove breakpoint: Gdb is busy.");
                    }
                    Err(BreakpointOperationError::ExecutionError(msg)) => {
                        p.logger
                            .log_message(format!("Cannot remove breakpoint: {}", msg));
                    }
                }
            }
        }
    }
    fn event(&mut self, event: Input, p: ::UpdateParameters) -> Option<Input> {
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

impl<'a> Widget for AssemblyView<'a> {
    fn space_demand(&self) -> Demand2D {
        self.pager.space_demand()
    }
    fn draw(&self, window: Window, hints: RenderingHints) {
        self.pager.draw(window, hints)
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
            stop_position: stop_position,
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
            .unwrap_or(Width::new(0).unwrap());
        Demand::from_to(0, max_space.into())
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
            (true, false) => (
                '▶',
                StyleModifier::new().fg_color(Color::Green).bold(true),
            ),
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

struct FileInfo {
    path: PathBuf,
    modified: ::std::time::SystemTime,
}

pub struct SourceView<'a> {
    highlighting_theme: &'a Theme,
    syntax_set: SyntaxSet,
    pager: Pager<String, SourceDecorator>,
    file_info: Option<FileInfo>,
    last_stop_position: Option<SrcPosition>,
    stack_level: Option<u64>,
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
            highlighting_theme: highlighting_theme,
            syntax_set: SyntaxSet::load_defaults_nonewlines(),
            pager: Pager::new(),
            file_info: None,
            last_stop_position: None,
            stack_level: None,
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
                if &src_pos.file == &file_info.path {
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

    fn update_decoration(&mut self, p: ::UpdateParameters) {
        if let Some((ref file_path, ref mut content)) = current_file_and_content_mut!(self) {
            // This sucks: we basically want to call get_last_line_number_for, but can't because we
            // borrowed content mutably...
            let last_line_number = self.last_stop_position.clone().and_then(|last_src_pos| {
                if last_src_pos.file == **file_path {
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

    fn show<'b, P: AsRef<Path>, L: Into<LineIndex>>(
        &mut self,
        path: P,
        line: L,
        p: ::UpdateParameters,
    ) -> Result<(), PagerShowError> {
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
        let line = line.into();
        self.pager
            .go_to_line(line)
            .map_err(|_| PagerShowError::LineDoesNotExist(line))
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
            .unwrap_or(self.syntax_set.find_syntax_plain_text());
        let last_line_number = self.get_last_line_number_for(path.as_ref());
        self.pager.load(
            pager_content
                .with_highlighter(&SyntectHighlighter::new(syntax, self.highlighting_theme))
                .with_decorator(SourceDecorator::new(
                    path.as_ref(),
                    last_line_number,
                    breakpoints,
                )),
        );
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

    fn toggle_breakpoint(&self, p: ::UpdateParameters) {
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
                    p.logger
                        .log_message("Cannot insert breakpoint: Gdb is busy.");
                }
            } else {
                if p.gdb.delete_breakpoints(active_bps.into_iter()).is_err() {
                    p.logger
                        .log_message("Cannot remove breakpoint: Gdb is busy.");
                }
            }
        }
    }

    fn event(&mut self, event: Input, p: ::UpdateParameters) -> Option<Input> {
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

impl<'a> Widget for SourceView<'a> {
    fn space_demand(&self) -> Demand2D {
        self.pager.space_demand()
    }
    fn draw(&self, window: Window, hints: RenderingHints) {
        if let Some(file) = self.current_file() {
            match window.split(RowIndex::new(1)) {
                Ok((mut up, down)) => {
                    let mut cursor = Cursor::new(&mut up);
                    cursor.set_style_modifier(StyleModifier::new().bold(true));
                    if let Some(level) = self.stack_level {
                        cursor.write(&format!("[{}]", level));
                    } else {
                        cursor.write("[?]");
                    }
                    cursor.write(&format!(" ▶ {}", file.display()));
                    self.pager.draw(down, hints);
                }
                Err(window) => {
                    self.pager.draw(window, hints);
                }
            }
        } else {
            self.pager.draw(window, hints);
        }
    }
}

#[derive(Clone, PartialEq)]
enum CodeWindowMode {
    Source,
    Assembly,
    SideBySide,
    Message(String),
}

pub struct CodeWindow<'a> {
    src_view: SourceView<'a>,
    asm_view: AssemblyView<'a>,
    layout: HorizontalLayout,
    mode: CodeWindowMode,
    last_bp_update: ::std::time::Instant,
}

#[derive(Clone, Debug, PartialEq)]
enum GDBResponseError {
    MissingField(&'static str, JsonValue),
    MalformedAddress(String),
    Other(String),
}

impl From<(::std::num::ParseIntError, String)> for GDBResponseError {
    fn from((_, s): (::std::num::ParseIntError, String)) -> Self {
        GDBResponseError::MalformedAddress(s)
    }
}

#[derive(Clone, Debug, PartialEq)]
enum DisassembleError {
    Execution(ExecuteError),
    GDB(GDBResponseError),
    Other(String),
}

impl From<GDBResponseError> for DisassembleError {
    fn from(error: GDBResponseError) -> Self {
        DisassembleError::GDB(error)
    }
}

impl From<(::std::num::ParseIntError, String)> for DisassembleError {
    fn from((_, s): (::std::num::ParseIntError, String)) -> Self {
        GDBResponseError::MalformedAddress(s).into()
    }
}

#[derive(Debug)]
enum ShowError {
    GDB(GDBResponseError),
    CouldNotOpenFile(PathBuf),
    NoLineInfo,
}

impl From<GDBResponseError> for ShowError {
    fn from(error: GDBResponseError) -> Self {
        ShowError::GDB(error)
    }
}

fn disassemble_address(
    address_start: Address,
    address_end: Address,
    p: ::UpdateParameters,
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
            return Err(DisassembleError::Execution(e));
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
        Err(GDBResponseError::MissingField(
            "asm_insns",
            JsonValue::Object(disass_results.clone()),
        ))?
    }
}

impl<'a> CodeWindow<'a> {
    pub fn new(highlighting_theme: &'a Theme, welcome_msg: &'static str) -> Self {
        CodeWindow {
            src_view: SourceView::new(highlighting_theme),
            asm_view: AssemblyView::new(highlighting_theme),
            layout: HorizontalLayout::new(SeparatingStyle::Draw(
                GraphemeCluster::try_from('|').unwrap(),
            )),
            mode: CodeWindowMode::Message(welcome_msg.to_owned()),
            last_bp_update: ::std::time::Instant::now(),
        }
    }

    fn show_from_file(&mut self, frame: &Object, p: ::UpdateParameters) -> Result<(), ShowError> {
        self.src_view.stack_level = match p.gdb.mi.execute(MiCommand::stack_info_frame(None)) {
            Ok(o) => get_u64(&o.results["frame"], "level").ok(),
            Err(_) => None,
        };

        let address = get_addr_obj(frame, "addr")?;

        if let Some(path) = frame["fullname"].as_str() {
            // File information may not be present
            let line = LineNumber::new(
                get_str_obj(frame, "line")?
                    .parse::<usize>()
                    .map_err(|_| GDBResponseError::Other(format!("Malformed line")))?,
            );
            self.src_view.set_last_stop_position(path, line);

            match self.src_view.show(path, line, p) {
                Ok(()) => {
                    self.src_view
                        .go_to_last_stop_position()
                        .expect("We just set a last stop pos!");
                }
                Err(PagerShowError::CouldNotOpenFile(b, _)) => {
                    return Err(ShowError::CouldNotOpenFile(b));
                }
                Err(PagerShowError::LineDoesNotExist(_)) => {
                    //Ignore
                }
            }

            if self.mode == CodeWindowMode::Assembly || self.mode == CodeWindowMode::SideBySide {
                self.asm_view.set_last_stop_position(address);
                if self.asm_view.show_file(path, line, p).is_ok() {
                    if self.asm_view.go_to_last_stop_position().is_err() {
                        p.logger
                            .log_message(format!("Failed to go to address: {}", address));
                    }
                } else {
                    p.logger
                        .log_debug(format!("Disassembly failed, switching to source mode"));
                    self.mode = CodeWindowMode::Source;
                }
            }

            self.mode = match &self.mode {
                &CodeWindowMode::Message(_) | &CodeWindowMode::Assembly => CodeWindowMode::Source,
                &ref other => other.clone(),
            };
            Ok(())
        } else {
            Err(ShowError::NoLineInfo)
        }
    }
    fn find_function_range(at: Address, p: ::UpdateParameters) -> Result<(Address, Address), ()> {
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
        p: ::UpdateParameters,
    ) -> Result<(Address, Address), DisassembleError> {
        let block_lines = disassemble_address(at, at + approx_byte_size, p)?;

        let penultimate_index = block_lines
            .len()
            .checked_sub(2)
            .ok_or(DisassembleError::Other("Not enough lines".to_owned()))?;
        let penultimate = block_lines
            .get(penultimate_index)
            .ok_or(DisassembleError::Other("Not enough lines".to_owned()))?;
        let end_address = get_addr(penultimate, "address")?;
        Ok((at, end_address))
    }

    fn show_from_address(
        &mut self,
        frame: &Object,
        p: ::UpdateParameters,
    ) -> Result<(), DisassembleError> {
        let address = get_addr_obj(frame, "addr")?;

        let (begin, end) = {
            Self::find_function_range(address, p)
                .or_else(|_| Self::find_valid_address_range(address, 128, p))?
        };

        self.asm_view.set_last_stop_position(address);

        if self.asm_view.show_address(begin, end, p).is_ok() {
            if let Err(e) = self.asm_view.go_to_last_stop_position() {
                p.logger.log_debug(format!("We just set a last stop pos {}, but it does not seem to be valid must be valid: {:?}", address, e));
            }
            self.mode = CodeWindowMode::Assembly;
        } else {
            self.mode = CodeWindowMode::Message("Disassembly failed!".to_owned());
        }

        Ok(())
    }
    pub fn show_frame(&mut self, frame: &Object, p: ::UpdateParameters) {
        match self.show_from_file(frame, p) {
            Ok(_) => return, /*Done!*/
            Err(ShowError::NoLineInfo) => {
                // That's fine, just try disassemble instead
            }
            Err(other) => {
                p.logger
                    .log_debug(format!("Error showing file: {:?}", other));
            }
        }
        match self.show_from_address(frame, p) {
            Ok(_) => return, /*Done!*/
            Err(other) => {
                p.logger
                    .log_debug(format!("Error showing asm: {:?}", other));
                self.mode = CodeWindowMode::Message("Disassembly failed!".to_owned());
            }
        }
    }

    fn toggle_mode(&mut self, p: ::UpdateParameters) {
        self.mode = match self.mode {
            CodeWindowMode::Assembly => {
                if self.src_view.current_file().is_some() {
                    CodeWindowMode::Source
                } else {
                    CodeWindowMode::Assembly
                }
            }
            CodeWindowMode::SideBySide => CodeWindowMode::Assembly,
            CodeWindowMode::Source => {
                if let Some(path) = self.src_view.current_file() {
                    if self
                        .asm_view
                        .show_file(path, self.src_view.current_line_number(), p)
                        .is_ok()
                    {
                        // The current line may not have associated assembly!
                        // TODO: Maybe we want to try the next line or something...
                        let _ = self
                            .asm_view
                            .go_to_first_applicable_line(path, self.src_view.current_line_number());
                    }
                }
                CodeWindowMode::SideBySide
            }
            CodeWindowMode::Message(ref m) => CodeWindowMode::Message(m.clone()),
        }
    }

    fn try_switch_stackframe(
        &mut self,
        p: ::UpdateParameters,
        up: bool,
    ) -> Result<(), GDBResponseError> {
        let stack_result = match p.gdb.mi.execute(MiCommand::stack_info_frame(None)) {
            Ok(o) => o.results,
            Err(_) => return Ok(()), //Ignore
        };
        let level = get_u64(&stack_result["frame"], "level")?;

        let new_level = if up {
            let depth_result = match p.gdb.mi.execute(MiCommand::stack_info_depth()) {
                Ok(o) => o.results,
                Err(_) => return Ok(()), //Ignore
            };
            let depth = get_u64_obj(&depth_result, "depth")?;
            (level + 1).min(depth.checked_sub(1).unwrap_or(0))
        } else {
            level.checked_sub(1).unwrap_or(0)
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
    fn switch_stackframe(&mut self, p: ::UpdateParameters, up: bool) {
        match self.try_switch_stackframe(p, up) {
            Ok(_) => {}
            Err(e) => {
                p.logger
                    .log_debug(format!("Failed to switch stackframe: {:?}", e));
            }
        }
    }

    pub fn update_after_event(&mut self, p: ::UpdateParameters) {
        if p.gdb.breakpoints.last_change > self.last_bp_update {
            self.asm_view.update_decoration(p);
            self.src_view.update_decoration(p);
            self.last_bp_update = p.gdb.breakpoints.last_change;
        }
    }
}

impl<'a> Widget for CodeWindow<'a> {
    fn space_demand(&self) -> Demand2D {
        match &self.mode {
            &CodeWindowMode::Assembly => self.asm_view.space_demand(),
            &CodeWindowMode::SideBySide => {
                self.layout.space_demand(&[&self.asm_view, &self.src_view])
            }
            &CodeWindowMode::Source => self.src_view.space_demand(),
            &CodeWindowMode::Message(ref m) => MsgWindow::new(&m).space_demand(),
        }
    }
    fn draw(&self, window: Window, hints: RenderingHints) {
        match &self.mode {
            &CodeWindowMode::Assembly => self.asm_view.draw(window, hints),
            &CodeWindowMode::SideBySide => self.layout.draw(
                window,
                &[
                    (&self.asm_view, hints),
                    (&self.src_view, hints.active(false)),
                ],
            ),
            &CodeWindowMode::Source => self.src_view.draw(window, hints),
            &CodeWindowMode::Message(ref m) => MsgWindow::new(&m).draw(window, hints),
        }
    }
}

impl<'a> Container<::UpdateParametersStruct> for CodeWindow<'a> {
    fn input(&mut self, input: Input, p: ::UpdateParameters) -> Option<Input> {
        input
            .chain((Key::Char('d'), || self.toggle_mode(p)))
            .chain((Key::PageUp, || self.switch_stackframe(p, true)))
            .chain((Key::PageDown, || self.switch_stackframe(p, false)))
            .chain(|i: Input| match self.mode {
                CodeWindowMode::Assembly | CodeWindowMode::SideBySide => {
                    let ret = self.asm_view.event(i, p);
                    if let Some(src_pos) = self
                        .asm_view
                        .pager
                        .current_line()
                        .and_then(|ref line| line.src_position.clone())
                    {
                        let _ = self.src_view.show(src_pos.file, src_pos.line, p);
                    }
                    ret
                }
                CodeWindowMode::Source => self.src_view.event(i, p),
                CodeWindowMode::Message(_) => Some(i),
            })
            .finish()
    }
}

struct MsgWindow<'a> {
    msg: &'a str,
}

impl<'a> MsgWindow<'a> {
    fn new(msg: &'a str) -> Self {
        MsgWindow { msg: msg }
    }
}

impl<'a> Widget for MsgWindow<'a> {
    fn space_demand(&self) -> Demand2D {
        Demand2D {
            width: Demand::at_least(1),
            height: Demand::at_least(1),
        }
    }
    fn draw(&self, mut window: Window, _: RenderingHints) {
        let lines: Vec<_> = self.msg.lines().collect();
        let num_lines = lines.len();

        let start_line = ((window.get_height() - num_lines as i32) / 2).from_origin();
        let window_width = window.get_width();

        let mut c = Cursor::new(&mut window);
        c.move_to_y(start_line);
        for line in lines {
            let start_x = ((window_width - text_width(line)) / 2).from_origin();
            c.move_to_x(start_x);
            c.write(line);
            c.wrap_line();
        }
    }
}
