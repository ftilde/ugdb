extern crate json;
extern crate nix;
#[macro_use]
extern crate nom;

pub mod input;
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
    OsStr,
    OsString
};

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

impl GDB {
    pub fn spawn_with_executable<S>(executable_path: &str, process_tty_name: &str, oob_sink: S) -> Result<GDB, ::std::io::Error> where S: OutOfBandRecordSink + 'static {
        Self::spawn(&[executable_path], process_tty_name, oob_sink)
    }
    pub fn spawn<S, A: AsRef<OsStr>, B: AsRef<OsStr>>(arguments: &[A], process_tty_name: B, oob_sink: S) -> Result<GDB, ::std::io::Error> where S: OutOfBandRecordSink + 'static {
        let mut tty_arg = OsString::from("--tty=");
        tty_arg.push(process_tty_name.as_ref());
        let mut child = try!{Command::new("/bin/gdb")
            .arg("--interpreter=mi")
            .arg(tty_arg)
            .args(arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()};
        let stdin = child.stdin.take().expect("take stdin");
        let stdout = child.stdout.take().expect("take stdout");
        let is_running = Arc::new(AtomicBool::new(false));
        let is_running_for_thread = is_running.clone();
        let (result_input, result_output) = mpsc::channel();
        /*let outputThread = */ thread::Builder::new().name("gdbmi parser".to_owned()).spawn(move || {
            output::process_output(stdout, result_input, oob_sink, is_running_for_thread);
        }).expect("Spawn gdbmi parser thread");
        Ok(
            GDB {
                process: child,
                stdin: stdin,
                is_running: is_running,
                result_output: result_output,
                current_command_token: 0,
                //outputThread: outputThread,
            }
          )
    }

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

    pub fn execute<C: std::borrow::Borrow<input::MiCommand>>(&mut self, command: C) -> Result<output::ResultRecord, ExecuteError> {
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

    pub fn execute_later(&mut self, command: &input::MiCommand) {
        let command_token = self.get_usable_token();
        command.write_interpreter_string(&mut self.stdin, command_token).expect("write interpreter command");
        let _ = self.result_output.recv();
    }
}
