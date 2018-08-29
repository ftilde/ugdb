extern crate json;
extern crate nix;
#[macro_use] extern crate nom;

pub mod commands;
pub mod output;

use std::process::{
    Command,
    Child,
    ChildStdin,
    Stdio,
};
use std::thread;
use std::sync::{
    mpsc,
    Arc,
};
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};
use std::ffi::{
    OsString
};
use std::path::PathBuf;

type Token = u64;

pub struct GDB {
    pub process: Child,
    stdin: ChildStdin,
    is_running: Arc<AtomicBool>,
    result_output: mpsc::Receiver<output::ResultRecord>,
    current_command_token: Token,
    //outputThread: thread::Thread,
}

pub trait OutOfBandRecordSink: std::marker::Send {
    fn send(&self, output::OutOfBandRecord);
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExecuteError {
    Busy,
    Quit,
}

pub struct GDBBuilder {
    gdb_path: PathBuf,
    opt_nh: bool,
    opt_nx: bool,
    opt_quiet: bool,
    opt_batch: bool,
    opt_cd: Option<PathBuf>,
    opt_bps: Option<u32>,
    opt_symbol_file: Option<PathBuf>,
    opt_core_file: Option<PathBuf>,
    opt_proc_id: Option<u32>,
    opt_command: Option<PathBuf>,
    opt_source_dir: Option<PathBuf>,
    opt_args: Vec<OsString>,
    opt_program: Option<PathBuf>,
    opt_tty: Option<PathBuf>,
}
#[derive(Debug)]
pub enum SpawnError {
    Io(::std::io::Error),
    Execute(ExecuteError),

}
impl GDBBuilder {
    pub fn new(gdb: PathBuf) -> Self {
        GDBBuilder {
            gdb_path: gdb,
            opt_nh: false,
            opt_nx: false,
            opt_quiet: false,
            opt_batch: false,
            opt_cd: None,
            opt_bps: None,
            opt_symbol_file: None,
            opt_core_file: None,
            opt_proc_id: None,
            opt_command: None,
            opt_source_dir: None,
            opt_args: Vec::new(),
            opt_program: None,
            opt_tty: None,
        }
    }

    pub fn nh(mut self) -> Self {
        self.opt_nh = true;
        self
    }
    pub fn nx(mut self) -> Self {
        self.opt_nx = true;
        self
    }
    pub fn quiet(mut self) -> Self {
        self.opt_quiet = true;
        self
    }
    pub fn batch(mut self) -> Self {
        self.opt_batch = true;
        self
    }
    pub fn working_dir(mut self, dir: PathBuf) -> Self {
        self.opt_cd = Some(dir);
        self
    }
    pub fn bps(mut self, bps: u32) -> Self {
        self.opt_bps = Some(bps);
        self
    }
    pub fn symbol_file(mut self, file: PathBuf) -> Self {
        self.opt_symbol_file = Some(file);
        self
    }
    pub fn core_file(mut self, file: PathBuf) -> Self {
        self.opt_core_file = Some(file);
        self
    }
    pub fn proc_id(mut self, pid: u32) -> Self {
        self.opt_proc_id = Some(pid);
        self
    }
    pub fn command_file(mut self, command_file: PathBuf) -> Self {
        self.opt_command = Some(command_file);
        self
    }
    pub fn source_dir(mut self, dir: PathBuf) -> Self {
        self.opt_source_dir = Some(dir);
        self
    }
    pub fn args(mut self, args: &[OsString]) -> Self {
        self.opt_args.extend_from_slice(args);
        self
    }
    pub fn program(mut self, program: PathBuf) -> Self {
        self.opt_program = Some(program);
        self
    }
    pub fn tty(mut self, tty: PathBuf) -> Self {
        self.opt_tty = Some(tty);
        self
    }
    pub fn try_spawn<S>(self, oob_sink: S) -> Result<GDB, SpawnError> where S: OutOfBandRecordSink + 'static
    {
        let mut args = Vec::<OsString>::new();
        if self.opt_nh {
            args.push("--nh".into());
        }
        if self.opt_nx {
            args.push("--nx".into());
        }
        if self.opt_quiet {
            args.push("--quiet".into());
        }
        if self.opt_batch {
            args.push("--batch".into());
        }
        if let Some(cd) = self.opt_cd {
            args.push("--cd=".into());
            args.last_mut().unwrap().push(&cd);
        }
        if let Some(bps) = self.opt_bps {
            args.push("-b".into());
            args.push(bps.to_string().into());
        }
        if let Some(symbol_file) = self.opt_symbol_file {
            args.push("--symbols=".into());
            args.last_mut().unwrap().push(&symbol_file);
        }
        if let Some(core_file) = self.opt_core_file {
            args.push("--core=".into());
            args.last_mut().unwrap().push(&core_file);
        }
        if let Some(proc_id) = self.opt_proc_id {
            args.push("--pid=".into());
            args.last_mut().unwrap().push(proc_id.to_string());
        }
        if let Some(command) = self.opt_command {
            args.push("--command=".into());
            args.last_mut().unwrap().push(&command);
        }
        if let Some(source_dir) = self.opt_source_dir {
            args.push("--directory=".into());
            args.last_mut().unwrap().push(&source_dir);
        }
        if let Some(tty) = self.opt_tty {
            args.push("--tty=".into());
            args.last_mut().unwrap().push(&tty);
        }
        if let Some(program) = self.opt_program {
            args.push(program.into());
        }

        let mut child = Command::new(self.gdb_path)
            .arg("--interpreter=mi")
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn().map_err(SpawnError::Io)?;
        let stdin = child.stdin.take().expect("take stdin");
        let stdout = child.stdout.take().expect("take stdout");
        let is_running = Arc::new(AtomicBool::new(false));
        let is_running_for_thread = is_running.clone();
        let (result_input, result_output) = mpsc::channel();
        /*let outputThread = */ thread::Builder::new().name("gdbmi parser".to_owned()).spawn(move || {
            output::process_output(stdout, result_input, oob_sink, is_running_for_thread);
        }).map_err(SpawnError::Io)?;
        let mut gdb = GDB {
            process: child,
            stdin: stdin,
            is_running: is_running,
            result_output: result_output,
            current_command_token: 0,
            //outputThread: outputThread,
        };
        if !self.opt_args.is_empty() {
            gdb.execute(commands::MiCommand::exec_arguments(self.opt_args)).map_err(SpawnError::Execute)?;
        }
        Ok(gdb)
    }
}

impl GDB {
    pub fn interrupt_execution(&self) -> Result<(), ::nix::Error> {
        use ::nix::sys::signal;
        signal::kill(self.process.id() as i32, signal::SIGINT)
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
    pub fn get_usable_token(&mut self) -> Token {
        self.current_command_token = self.current_command_token.wrapping_add(1);
        self.current_command_token
    }

    pub fn execute<C: std::borrow::Borrow<commands::MiCommand>>(&mut self, command: C) -> Result<output::ResultRecord, ExecuteError> {
        if self.is_running() {
            return Err(ExecuteError::Busy)
        }
        let command_token = self.get_usable_token();

        command.borrow().write_interpreter_string(&mut self.stdin, command_token).expect("write interpreter command");
        match self.result_output.recv() {
            Ok(record) => {
                let token = record.token;
                if token.is_none() || token.unwrap() != command_token {
                    panic!("Input token ({}) does not match output token ({:?})", command_token, token);
                }
                Ok(record)
            },
            Err(_) => {
                Err(ExecuteError::Quit)
            },
        }
    }

    pub fn execute_later(&mut self, command: &commands::MiCommand) {
        let command_token = self.get_usable_token();
        command.write_interpreter_string(&mut self.stdin, command_token).expect("write interpreter command");
        let _ = self.result_output.recv();
    }

    pub fn is_session_active(&mut self) -> Result<bool, ExecuteError> {
        let res = self.execute(commands::MiCommand::thread_info(None))?;
        Ok(!res.results["threads"].is_empty())
    }
}
