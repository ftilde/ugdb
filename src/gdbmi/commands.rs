use std::ffi::OsString;
use std::fmt;
use std::io::{Error, Write};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct MiCommand {
    operation: &'static str,
    options: Vec<OsString>,
    parameters: Vec<OsString>,
}

pub enum DisassembleMode {
    DisassemblyOnly = 0,
    DisassemblyWithRawOpcodes = 2,
    MixedSourceAndDisassembly = 1, // deprecated and 4 would be preferred, but might not be available in older gdb(mi) versions
    MixedSourceAndDisassemblyWithRawOpcodes = 3, // deprecated and 5 would be preferred, same as above
}

pub enum WatchMode {
    Read,
    Write,
    Access,
}

pub enum BreakPointLocation<'a> {
    Address(usize),
    Function(&'a Path, &'a str),
    Line(&'a Path, usize),
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct BreakPointNumber {
    pub major: usize,
    pub minor: Option<usize>,
}

impl std::str::FromStr for BreakPointNumber {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(dot_pos) = s.find(".") {
            let major = s[..dot_pos].parse::<usize>().map_err(|e| e.to_string())?;
            let minor = s[dot_pos + 1..]
                .parse::<usize>()
                .map_err(|e| e.to_string())?;
            Ok(BreakPointNumber {
                major: major,
                minor: Some(minor),
            })
        } else {
            match s.parse::<usize>() {
                Ok(val) => Ok(BreakPointNumber {
                    major: val,
                    minor: None,
                }),
                Err(e) => Err(e.to_string()),
            }
        }
    }
}

impl fmt::Display for BreakPointNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(minor) = self.minor {
            write!(f, "{}.{}", self.major, minor)
        } else {
            write!(f, "{}", self.major)
        }
    }
}

fn escape_command(input: &str) -> String {
    let mut output = String::new();
    output.push('\"');
    for c in input.chars() {
        match c {
            '\\' => output.push_str("\\\\"),
            '\"' => output.push_str("\\\""),
            '\r' => output.push_str("\\\r"),
            '\n' => output.push_str("\\\n"),
            other => output.push(other),
        }
    }
    output.push('\"');
    output
}

impl MiCommand {
    pub fn write_interpreter_string<S: Write>(
        &self,
        sink: &mut S,
        token: super::Token,
    ) -> Result<(), Error> {
        use std::os::unix::ffi::OsStrExt;
        write!(sink, "{}-{}", token, self.operation)?;
        for option in &self.options {
            write!(sink, " ")?;
            sink.write_all(option.as_bytes())?;
        }
        if !self.parameters.is_empty() && !self.options.is_empty() {
            write!(sink, " --")?;
        }
        for parameter in &self.parameters {
            write!(sink, " ")?;
            sink.write_all(parameter.as_bytes())?;
        }
        writeln!(sink)?;
        Ok(())
    }
    pub fn interpreter_exec<S1: Into<OsString>, S2: Into<OsString>>(
        interpreter: S1,
        command: S2,
    ) -> MiCommand {
        MiCommand {
            operation: "interpreter-exec",
            options: vec![interpreter.into(), command.into()],
            parameters: Vec::new(),
        }
    }

    pub fn cli_exec(command: &str) -> MiCommand {
        Self::interpreter_exec("console".to_owned(), escape_command(&command))
    }

    pub fn data_disassemble_file<P: AsRef<Path>>(
        file: P,
        linenum: usize,
        lines: Option<usize>,
        mode: DisassembleMode,
    ) -> MiCommand {
        MiCommand {
            operation: "data-disassemble",
            options: vec![
                OsString::from("-f"),
                OsString::from(file.as_ref()),
                OsString::from("-l"),
                OsString::from(linenum.to_string()),
                OsString::from("-n"),
                OsString::from(lines.map(|l| l as isize).unwrap_or(-1).to_string()),
            ],
            parameters: vec![OsString::from((mode as u8).to_string())],
        }
    }

    pub fn data_disassemble_address(
        start_addr: usize,
        end_addr: usize,
        mode: DisassembleMode,
    ) -> MiCommand {
        MiCommand {
            operation: "data-disassemble",
            options: vec![
                OsString::from("-s"),
                OsString::from(start_addr.to_string()),
                OsString::from("-e"),
                OsString::from(end_addr.to_string()),
            ],
            parameters: vec![OsString::from((mode as u8).to_string())],
        }
    }

    pub fn data_evaluate_expression(expression: String) -> MiCommand {
        MiCommand {
            operation: "data-evaluate-expression",
            options: vec![OsString::from(format!("\"{}\"", expression))], //TODO: maybe we need to quote existing " in expression. Is this even possible?
            parameters: vec![],
        }
    }

    pub fn insert_breakpoint(location: BreakPointLocation) -> MiCommand {
        MiCommand {
            operation: "break-insert",
            options: match location {
                BreakPointLocation::Address(addr) => vec![OsString::from(format!("*0x{:x}", addr))],
                BreakPointLocation::Function(path, func_name) => {
                    let mut ret = OsString::from(path);
                    ret.push(":");
                    ret.push(func_name);
                    vec![ret]

                    // Not available in old gdb(mi) versions
                    //vec![
                    //    OsString::from("--source"),
                    //    OsString::from(path),
                    //    OsString::from("--function"),
                    //    OsString::from(func_name),
                    //]
                }
                BreakPointLocation::Line(path, line_number) => {
                    let mut ret = OsString::from(path);
                    ret.push(":");
                    ret.push(line_number.to_string());
                    vec![ret]

                    // Not available in old gdb(mi) versions
                    //vec![
                    //OsString::from("--source"),
                    //OsString::from(path),
                    //OsString::from("--line"),
                    //OsString::from(format!("{}", line_number)),
                    //],
                }
            },
            parameters: Vec::new(),
        }
    }

    pub fn delete_breakpoints<I: Iterator<Item = BreakPointNumber>>(
        breakpoint_numbers: I,
    ) -> MiCommand {
        //let options = options: breakpoint_numbers.map(|n| format!("{} ", n)).collect(),
        //GDB is broken: see http://sourceware-org.1504.n7.nabble.com/Bug-breakpoints-20133-New-unable-to-delete-a-sub-breakpoint-td396197.html
        let mut options = breakpoint_numbers
            .map(|n| format!("{} ", n.major).into())
            .collect::<Vec<OsString>>();
        options.sort();
        options.dedup();
        MiCommand {
            operation: "break-delete",
            options: options,
            parameters: Vec::new(),
        }
    }

    pub fn insert_watchpoing(expression: &str, mode: WatchMode) -> MiCommand {
        let options = match mode {
            WatchMode::Write => Vec::new(),
            WatchMode::Read => vec!["-r".into()],
            WatchMode::Access => vec!["-a".into()],
        };
        MiCommand {
            operation: "break-watch",
            options,
            parameters: vec![expression.into()],
        }
    }

    pub fn environment_pwd() -> MiCommand {
        MiCommand {
            operation: "environment-pwd",
            options: Vec::new(),
            parameters: Vec::new(),
        }
    }

    // Be aware: This does not seem to always interrupt execution.
    // Use gdb.interrupt_execution instead.
    pub fn exec_interrupt() -> MiCommand {
        MiCommand {
            operation: "exec-interrupt",
            options: Vec::new(),
            parameters: Vec::new(),
        }
    }

    // Warning: This cannot be used to pass special characters like \n to gdb because
    // (unlike it is said in the spec) there is apparently no way to pass \n unescaped
    // to gdb, and for "exec-arguments" gdb somehow does not unescape these chars...
    pub fn exec_arguments(args: Vec<OsString>) -> MiCommand {
        MiCommand {
            operation: "exec-arguments",
            options: args,
            parameters: Vec::new(),
        }
    }

    pub fn exit() -> MiCommand {
        MiCommand {
            operation: "gdb-exit",
            options: Vec::new(),
            parameters: Vec::new(),
        }
    }

    pub fn select_frame(frame_number: u64) -> MiCommand {
        MiCommand {
            operation: "stack-select-frame",
            options: vec![frame_number.to_string().into()],
            parameters: Vec::new(),
        }
    }

    pub fn stack_info_frame(frame_number: Option<u64>) -> MiCommand {
        MiCommand {
            operation: "stack-info-frame",
            options: if let Some(frame_number) = frame_number {
                vec![frame_number.to_string().into()]
            } else {
                vec![]
            },
            parameters: Vec::new(),
        }
    }

    pub fn stack_info_depth() -> MiCommand {
        MiCommand {
            operation: "stack-info-depth",
            options: Vec::new(),
            parameters: Vec::new(),
        }
    }

    pub fn stack_list_variables(
        thread_number: Option<u64>,
        frame_number: Option<u64>,
    ) -> MiCommand {
        let mut parameters = vec![];
        if let Some(thread_number) = thread_number {
            parameters.push("--thread".into());
            parameters.push(thread_number.to_string().into());
        }
        if let Some(frame_number) = frame_number {
            parameters.push("--frame".into());
            parameters.push(frame_number.to_string().into());
        }
        parameters.push("--simple-values".into()); //TODO: make configurable if required.
        MiCommand {
            operation: "stack-list-variables",
            options: Vec::new(),
            parameters,
        }
    }

    pub fn thread_info(thread_id: Option<u64>) -> MiCommand {
        MiCommand {
            operation: "thread-info",
            options: if let Some(id) = thread_id {
                vec![id.to_string().into()]
            } else {
                vec![]
            },
            parameters: Vec::new(),
        }
    }

    pub fn file_exec_and_symbols(file: &Path) -> MiCommand {
        MiCommand {
            operation: "file-exec-and-symbols",
            options: vec![file.into()],
            parameters: Vec::new(),
        }
    }

    pub fn file_symbol_file(file: Option<&Path>) -> MiCommand {
        MiCommand {
            operation: "file-symbol-file",
            options: if let Some(file) = file {
                vec![file.into()]
            } else {
                vec![]
            },
            parameters: Vec::new(),
        }
    }

    pub fn list_thread_groups(list_all_available: bool, thread_group_ids: &[u32]) -> MiCommand {
        MiCommand {
            operation: "list-thread-groups",
            options: if list_all_available {
                vec![OsString::from("--available")]
            } else {
                vec![]
            },
            parameters: thread_group_ids
                .iter()
                .map(|id| id.to_string().into())
                .collect(),
        }
    }

    pub fn var_create(
        name: Option<OsString>, /*none: generate name*/
        expression: &str,
        frame_addr: Option<u64>, /*none: current frame*/
    ) -> MiCommand {
        MiCommand {
            operation: "var-create",
            options: vec![],
            parameters: vec![
                name.map(|v| v.into()).unwrap_or(OsString::from("\"-\"")),
                OsString::from(
                    frame_addr
                        .map(|s| s.to_string())
                        .unwrap_or("\"*\"".to_string()),
                ),
                escape_command(expression).into(),
            ],
        }
    }
    pub fn var_delete(name: impl Into<OsString>, delete_children: bool) -> MiCommand {
        let mut parameters = vec![];
        if delete_children {
            parameters.push("-c".into());
        }
        parameters.push(name.into());
        MiCommand {
            operation: "var-delete",
            options: Vec::new(),
            parameters,
        }
    }
    pub fn var_list_children(
        name: impl Into<OsString>,
        print_values: bool,
        from_to: Option<std::ops::Range<u64>>,
    ) -> MiCommand {
        let mut com = MiCommand {
            operation: "var-list-children",
            options: vec![],
            parameters: vec![
                if print_values {
                    "--all-values"
                } else {
                    "--no-values"
                }
                .into(),
                name.into(),
            ],
        };
        if let Some(from_to) = from_to {
            com.parameters
                .push(OsString::from(from_to.start.to_string()));
            com.parameters.push(OsString::from(from_to.end.to_string()));
        }
        com
    }
}
