// This module encapsulates some functionality of gdb. Depending on how general this turns out, we
// may want to move it to a separate crate or merge it with gdbmi-rs
use gdbmi;
use gdbmi::commands::{BreakPointLocation, BreakPointNumber, MiCommand};
use gdbmi::output::{BreakPointEvent, JsonValue, Object, ResultClass};
use gdbmi::ExecuteError;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::{Add, Sub};
use std::path::PathBuf;
use unsegen::base::LineNumber;

#[derive(Debug, Clone)]
pub struct SrcPosition {
    pub file: PathBuf,
    pub line: LineNumber,
}

impl SrcPosition {
    pub fn new(file: PathBuf, line: LineNumber) -> Self {
        SrcPosition {
            file: file,
            line: line,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address(pub usize);
impl Address {
    pub fn parse(string: &str) -> Result<Self, (::std::num::ParseIntError, String)> {
        usize::from_str_radix(&string[2..], 16)
            .map(|u| Address(u))
            .map_err(|e| (e, string.to_owned()))
    }
}
impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}
impl Add<usize> for Address {
    type Output = Self;
    fn add(self, rhs: usize) -> Self {
        Address(self.0 + rhs)
    }
}

impl Sub<usize> for Address {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self {
        Address(self.0 - rhs)
    }
}

pub struct BreakPoint {
    pub number: BreakPointNumber,
    pub address: Option<Address>,
    pub enabled: bool,
    pub src_pos: Option<SrcPosition>, // May not be present if debug information is missing!
}

impl BreakPoint {
    pub fn from_json(bkpt: &Object) -> Self {
        let number = bkpt["number"]
            .as_str()
            .expect("find bp number")
            .parse::<BreakPointNumber>()
            .expect("Parse usize");
        let enabled = bkpt["enabled"].as_str().expect("find enabled") == "y";
        let address = bkpt["addr"]
            .as_str()
            .and_then(|addr| Address::parse(addr).ok()); //addr may not be present or contain
        let src_pos = {
            let maybe_file = bkpt["fullname"].as_str();
            let maybe_line = bkpt["line"]
                .as_str()
                .map(|l_nr| LineNumber::new(l_nr.parse::<usize>().expect("Parse usize")));
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

    pub fn all_from_json(bkpt_obj: &JsonValue) -> Box<Iterator<Item = BreakPoint>> {
        match bkpt_obj {
            &JsonValue::Object(ref bp) => Box::new(Some(Self::from_json(&bp)).into_iter()),
            &JsonValue::Array(ref bp_array) => Box::new(
                bp_array
                    .iter()
                    .map(|bp| {
                        if let &JsonValue::Object(ref bp) = bp {
                            Self::from_json(&bp)
                        } else {
                            panic!("Invalid breakpoint object in array");
                        }
                    })
                    .collect::<Vec<BreakPoint>>()
                    .into_iter(),
            ),
            _ => panic!("Invalid breakpoint object"),
        }
    }
}

pub struct BreakPointSet {
    map: HashMap<BreakPointNumber, BreakPoint>,
    pub last_change: ::std::time::Instant,
}

impl BreakPointSet {
    pub fn new() -> Self {
        BreakPointSet {
            map: HashMap::new(),
            last_change: ::std::time::Instant::now(),
        }
    }

    fn notify_change(&mut self) {
        self.last_change = ::std::time::Instant::now();
    }

    pub fn update_breakpoint(&mut self, new_bp: BreakPoint) {
        let _ = self.map.insert(new_bp.number, new_bp);
        //debug_assert!(res.is_some(), "Modified non-existent breakpoint");
        self.notify_change();
    }

    pub fn remove_breakpoint(&mut self, bp_num: BreakPointNumber) {
        self.map.remove(&bp_num);
        if bp_num.minor.is_none() {
            //TODO: ensure removal of child breakpoints
        }
        self.notify_change();
    }
}

impl ::std::ops::Deref for BreakPointSet {
    type Target = HashMap<BreakPointNumber, BreakPoint>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

pub struct GDB {
    pub mi: gdbmi::GDB,
    pub breakpoints: BreakPointSet,
}

pub enum BreakpointOperationError {
    Busy,
    ExecutionError(String),
}

impl GDB {
    pub fn new(mi: gdbmi::GDB) -> Self {
        GDB {
            mi: mi,
            breakpoints: BreakPointSet::new(),
        }
    }

    pub fn kill(&mut self) {
        self.mi.interrupt_execution().expect("interrupt worked");
        self.mi.execute_later(&gdbmi::commands::MiCommand::exit());
    }

    pub fn insert_breakpoint(
        &mut self,
        location: BreakPointLocation,
    ) -> Result<(), BreakpointOperationError> {
        let bp_result = self
            .mi
            .execute(&MiCommand::insert_breakpoint(location))
            .map_err(|e| match e {
                ExecuteError::Busy => BreakpointOperationError::Busy,
                ExecuteError::Quit => panic!("Could not insert breakpoint: GDB quit"),
            })?;
        match bp_result.class {
            ResultClass::Done => {
                self.handle_breakpoint_event(BreakPointEvent::Created, &bp_result.results);
                Ok(())
            }
            ResultClass::Error => Err(BreakpointOperationError::ExecutionError(
                bp_result
                    .results
                    .get("msg")
                    .and_then(|msg_obj| msg_obj.as_str())
                    .map(|s| s.to_owned())
                    .unwrap_or(bp_result.results.dump()),
            )),
            _ => {
                panic!("Unexpected resultclass: {:?}", bp_result.class);
            }
        }
    }

    pub fn delete_breakpoints<I: Clone + Iterator<Item = BreakPointNumber>>(
        &mut self,
        bp_numbers: I,
    ) -> Result<(), BreakpointOperationError> {
        let bp_result = self
            .mi
            .execute(MiCommand::delete_breakpoints(bp_numbers.clone()))
            .map_err(|e| match e {
                ExecuteError::Busy => BreakpointOperationError::Busy,
                ExecuteError::Quit => panic!("Could not insert breakpoint: GDB quit"),
            })?;
        match bp_result.class {
            ResultClass::Done => {
                let major_to_delete = bp_numbers.map(|n| n.major).collect::<HashSet<usize>>();
                let bkpts_to_delete = self
                    .breakpoints
                    .map
                    .keys()
                    .filter_map(|&k| {
                        if major_to_delete.contains(&k.major) {
                            Some(k)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<BreakPointNumber>>();
                for bkpt in bkpts_to_delete {
                    self.breakpoints.remove_breakpoint(bkpt);
                }
                Ok(())
            }
            ResultClass::Error => Err(BreakpointOperationError::ExecutionError(
                bp_result
                    .results
                    .get("msg")
                    .and_then(|msg_obj| msg_obj.as_str())
                    .map(|s| s.to_owned())
                    .unwrap_or(bp_result.results.dump()),
            )),
            _ => {
                panic!("Unexpected resultclass: {:?}", bp_result.class);
            }
        }
    }

    pub fn handle_breakpoint_event(&mut self, bp_type: BreakPointEvent, info: &Object) {
        match bp_type {
            BreakPointEvent::Created | BreakPointEvent::Modified => {
                match &info["bkpt"] {
                    &JsonValue::Object(ref bkpt) => {
                        let bp = BreakPoint::from_json(&bkpt);
                        self.breakpoints.update_breakpoint(bp);
                        //debug_assert!(bp_type != BreakPointEvent::Modified || res.is_some(), "Modified non-existent id");
                    }
                    &JsonValue::Array(ref bkpts) => {
                        for bkpt in bkpts {
                            if let &JsonValue::Object(ref bkpt) = bkpt {
                                let bp = BreakPoint::from_json(&bkpt);
                                self.breakpoints.update_breakpoint(bp);
                            } else {
                                panic!("Malformed breakpoint list");
                            }
                        }
                    }
                    _ => {
                        panic!("Invalid bkpt structure");
                    }
                }
            }
            BreakPointEvent::Deleted => {
                let id = info["id"]
                    .as_str()
                    .expect("find id")
                    .parse::<BreakPointNumber>()
                    .expect("Parse usize");
                self.breakpoints.remove_breakpoint(id);
            }
        }
    }

    // Warning: This is a hack, as gdbmi does not currently offer a command to query the current target
    // May not work and can break at any time.
    pub fn get_target(&mut self) -> Result<Option<PathBuf>, ExecuteError> {
        let result = self.mi.execute(MiCommand::list_thread_groups(false, &[]))?;
        if result.class == ResultClass::Done {
            Ok(result.results["groups"]
                .members()
                .filter_map(|thread| thread["executable"].as_str())
                .next()
                .map(|exec| exec.into()))
        } else {
            Ok(None)
        }
    }

    pub fn get_stack_level(&mut self) -> Result<u64, response::GDBResponseError> {
        let frame = self.mi.execute(MiCommand::stack_info_frame(None))?;
        response::get_u64(&frame.results["frame"], "level")
    }
}

// Various helper for getting stuff out of gdb response values
pub mod response {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    pub enum GDBResponseError {
        MissingField(&'static str, JsonValue),
        MalformedAddress(String),
        Other(String),
        Execution(ExecuteError),
    }

    impl From<(::std::num::ParseIntError, String)> for GDBResponseError {
        fn from((_, s): (::std::num::ParseIntError, String)) -> Self {
            GDBResponseError::MalformedAddress(s)
        }
    }
    impl From<ExecuteError> for GDBResponseError {
        fn from(e: ExecuteError) -> Self {
            GDBResponseError::Execution(e)
        }
    }

    pub fn get_str<'a>(obj: &'a JsonValue, key: &'static str) -> Result<&'a str, GDBResponseError> {
        Ok(obj[key]
            .as_str()
            .ok_or_else(|| GDBResponseError::MissingField(key, obj.clone()))?)
    }

    pub fn get_str_obj<'a>(
        obj: &'a Object,
        key: &'static str,
    ) -> Result<&'a str, GDBResponseError> {
        Ok(obj[key]
            .as_str()
            .ok_or_else(|| GDBResponseError::MissingField(key, JsonValue::Object(obj.clone())))?)
    }

    pub fn get_addr<'a>(
        obj: &'a JsonValue,
        key: &'static str,
    ) -> Result<Address, GDBResponseError> {
        let s = get_str(obj, key)?;
        Ok(Address::parse(s)?)
    }

    pub fn get_addr_obj<'a>(
        obj: &'a Object,
        key: &'static str,
    ) -> Result<Address, GDBResponseError> {
        let s = get_str_obj(obj, key)?;
        Ok(Address::parse(s)?)
    }

    pub fn get_u64<'a>(obj: &'a JsonValue, key: &'static str) -> Result<u64, GDBResponseError> {
        let s = get_str(obj, key)?;
        Ok(s.parse::<u64>()
            .map_err(|_| GDBResponseError::Other(format!("Malformed frame description")))?)
    }

    pub fn get_u64_obj<'a>(obj: &'a Object, key: &'static str) -> Result<u64, GDBResponseError> {
        let s = get_str_obj(obj, key)?;
        Ok(s.parse::<u64>()
            .map_err(|_| GDBResponseError::Other(format!("Malformed frame description")))?)
    }
}
