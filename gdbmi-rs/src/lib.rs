extern crate json;
extern crate nix;
#[macro_use]
extern crate nom;

pub mod input;
pub mod output;

use std::process::{Command,Child,ChildStdin,Stdio};
use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct GDB {
    pub process: Child,
    stdin: ChildStdin,
    is_running: Arc<AtomicBool>,
    result_output: mpsc::Receiver<output::ResultRecord>,
    //outputThread: thread::Thread,
}

pub trait OutOfBandRecordSink: std::marker::Send {
    fn send(&self, output::OutOfBandRecord);
}

#[derive(Debug, PartialEq)]
pub enum ExecuteError {
    Busy,
    Quit,
}

impl GDB {
    pub fn spawn<S>(executable_path: &str, process_tty_name: &str, oob_sink: S) -> Result<GDB, ::std::io::Error> where S: OutOfBandRecordSink + 'static{
        let mut child = try!{Command::new("/bin/gdb")
            .arg("--interpreter=mi")
            .arg(format!("--tty={}", process_tty_name))
            .arg(executable_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()};
        let stdin = child.stdin.take().expect("take stdin");
        let stdout = child.stdout.take().expect("take stdout");
        let is_running = Arc::new(AtomicBool::new(false));
        let is_running_for_thread = is_running.clone();
        let (result_input, result_output) = mpsc::channel();
        /*let outputThread = */ thread::spawn(move || {
            output::process_output(stdout, result_input, oob_sink, is_running_for_thread);
        });
        Ok(
            GDB {
                process: child,
                stdin: stdin,
                is_running: is_running,
                result_output: result_output,
                //outputThread: outputThread,
            }
          )
    }

    pub fn interrupt_execution(&self) -> Result<(), ::nix::Error> {
        use ::nix::sys::signal;
        signal::kill(self.process.id() as i32, signal::SIGINT)
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed) /* TODO: maybe some other ordering? */
    }

    pub fn execute<C: std::borrow::Borrow<input::MiCommand>>(&mut self, command: C) -> Result<output::ResultRecord, ExecuteError> {
        if self.is_running() {
            return Err(ExecuteError::Busy)
        }

        command.borrow().write_interpreter_string(&mut self.stdin).expect("write interpreter command");
        match self.result_output.recv() {
            Ok(record) => Ok(record),
            Err(e) => {
                println!("Execute error: {}", e);
                Err(ExecuteError::Quit)
            },
        }
    }

    pub fn execute_later(&mut self, command: &input::MiCommand) {
        command.write_interpreter_string(&mut self.stdin).expect("write interpreter command");
        let _ = self.result_output.recv();
    }
}
