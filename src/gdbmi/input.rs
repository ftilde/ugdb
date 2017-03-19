use std::io::{Write, Error};
use std::path::{
    Path,
};

#[derive(Debug)]
pub struct MiCommand {
    operation: String,
    options: Vec<String>,
    parameters: Vec<String>,
}

impl MiCommand {
    pub fn write_interpreter_string<F: Write>(&self, formatter: &mut F) -> Result<(), Error> {
        try!{write!(formatter, "-{}", self.operation)};
        for option in &self.options {
            try!{write!(formatter, " {}", option)};
        }
        if !self.parameters.is_empty() {
            try!{write!(formatter, " --")};
            for parameter in &self.parameters {
                try!{write!(formatter, " {}", parameter)};
            }
        }
        try!{write!(formatter, "\n")};
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

    pub fn data_disassemble_file<P: AsRef<Path>>(file: P, linenum: usize, lines: Option<usize>) -> MiCommand {
        MiCommand {
            operation: "data-disassemble".to_owned(),
            options: vec!["-f".to_owned(), file.as_ref().to_string_lossy().to_string(), "-l".to_owned(), linenum.to_string(), "-n".to_owned(), lines.map(|l| l as isize).unwrap_or(-1).to_string()],
            parameters: vec!["0".to_owned()], //TODO: as parameter?
        }
    }

    #[allow(dead_code)]
    pub fn data_disassemble_address(start_addr: usize, end_addr: usize) -> MiCommand {
        MiCommand {
            operation: "data-disassemble".to_owned(),
            options: vec!["-s".to_owned(), start_addr.to_string(), "-e".to_owned(), end_addr.to_string()],
            parameters: vec!["0".to_owned()], //TODO: as parameter?
        }
    }

    // Be aware: This does not seem to always interrupt execution.
    // Use gdb.interrupt_execution instead.
    #[allow(dead_code)]
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

