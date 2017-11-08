use std::io::{Write, Error};
use std::path::{
    Path,
};
use std::fmt;

#[derive(Debug)]
pub struct MiCommand {
    operation: String,
    options: Vec<String>,
    parameters: Vec<String>,
}

pub enum DisassembleMode {
    DissassemblyOnly = 0,
    DissassemblyWithRawOpcodes = 1,
    MixedSourceAndDisassembly = 4,
    MixedSourceAndDisassemblyWithRawOpcodes = 5,
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

impl ::std::str::FromStr for BreakPointNumber {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use std::error::Error;

        if let Some(dot_pos) = s.find(".") {
            let major = try!{s[.. dot_pos].parse::<usize>().map_err(|e| e.description().to_string())};
            let minor = try!{s[dot_pos+1 ..].parse::<usize>().map_err(|e| e.description().to_string())};
            Ok(BreakPointNumber {
                major: major,
                minor: Some(minor),
            })
        } else {
            match s.parse::<usize>() {
                Ok(val) => Ok(BreakPointNumber {
                    major: val,
                    minor: None
                }),
                Err(e) => Err(e.description().to_string()),
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

impl MiCommand {
    pub fn write_interpreter_string<S: Write>(&self, sink: &mut S, token: super::Token) -> Result<(), Error> {
        try!{write!(sink, "{}-{}", token, self.operation)};
        for option in &self.options {
            try!{write!(sink, " {}", option)};
        }
        if !self.parameters.is_empty() {
            try!{write!(sink, " --")};
            for parameter in &self.parameters {
                try!{write!(sink, " {}", parameter)};
            }
        }
        try!{write!(sink, "\n")};
        Ok(())
    }
    pub fn interpreter_exec(interpreter: String, command: String) -> MiCommand {
        MiCommand {
            operation: "interpreter-exec".to_owned(),
            options: vec![interpreter, command],
            parameters: Vec::new(),
        }
    }

    pub fn cli_exec(command: String) -> MiCommand {
        //TODO need quotes everywhere?
        Self::interpreter_exec("console".to_owned(), format!("\"{}\"", command))
    }

    pub fn data_disassemble_file<P: AsRef<Path>>(file: P, linenum: usize, lines: Option<usize>, mode: DisassembleMode) -> MiCommand {
        MiCommand {
            operation: "data-disassemble".to_owned(),
            options: vec!["-f".to_owned(), file.as_ref().to_string_lossy().to_string(), "-l".to_owned(), linenum.to_string(), "-n".to_owned(), lines.map(|l| l as isize).unwrap_or(-1).to_string()],
            parameters: vec![format!("{}",(mode as u8))],
        }
    }

    pub fn data_disassemble_address(start_addr: usize, end_addr: usize, mode: DisassembleMode) -> MiCommand {
        MiCommand {
            operation: "data-disassemble".to_owned(),
            options: vec!["-s".to_owned(), start_addr.to_string(), "-e".to_owned(), end_addr.to_string()],
            parameters: vec![format!("{}",(mode as u8))],
        }
    }

    pub fn data_evaluate_expression(expression: String) -> MiCommand {
        MiCommand {
            operation: "data-evaluate-expression".to_owned(),
            options: vec![format!("\"{}\"", expression)], //TODO: maybe we need to quote existing " in expression. Is this even possible?
            parameters: vec![],
        }
    }

    pub fn insert_breakpoint(location: BreakPointLocation) -> MiCommand {
        MiCommand {
            operation: "break-insert".to_owned(),
            options: match location {
                BreakPointLocation::Address(addr) => {
                    vec![format!("*0x{:x}", addr)] //TODO: is this correct?
                },
                BreakPointLocation::Function(path, func_name) => {
                    vec!["--source".to_owned(), path.to_string_lossy().into_owned(), "--function".to_owned(), func_name.to_owned()] //TODO: is this correct?
                },
                BreakPointLocation::Line(path, line_number) => {
                    vec!["--source".to_owned(), path.to_string_lossy().into_owned(), "--line".to_owned(), format!("{}", line_number)]
                },
            },
            parameters: Vec::new(),
        }
    }

    pub fn delete_breakpoints<I: Iterator<Item=BreakPointNumber>>(breakpoint_numbers: I) -> MiCommand {
        //let options = options: breakpoint_numbers.map(|n| format!("{} ", n)).collect(),
        //GDB is broken: see http://sourceware-org.1504.n7.nabble.com/Bug-breakpoints-20133-New-unable-to-delete-a-sub-breakpoint-td396197.html
        let mut options = breakpoint_numbers.map(|n| format!("{} ", n.major)).collect::<Vec<String>>();
        options.sort();
        options.dedup();
        MiCommand {
            operation: "break-delete".to_owned(),
            options: options,
            parameters: Vec::new(),
        }
    }

    pub fn environment_pwd() -> MiCommand {
        MiCommand {
            operation: "environment-pwd".to_owned(),
            options: Vec::new(),
            parameters: Vec::new(),
        }
    }

    // Be aware: This does not seem to always interrupt execution.
    // Use gdb.interrupt_execution instead.
    pub fn exec_interrupt(/*TODO incorporate all & threadgroup? */) -> MiCommand {
        MiCommand {
            operation: "exec-interrupt".to_owned(),
            options: Vec::new(),
            parameters: Vec::new(),
        }
    }
    pub fn exit() -> MiCommand {
        MiCommand {
            operation: "gdb-exit".to_owned(),
            options: Vec::new(),
            parameters: Vec::new(),
        }
    }
}

