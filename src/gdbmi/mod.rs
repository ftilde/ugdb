
pub mod input;
pub mod output;

use std::process::{Command,Child,ChildStdin,Stdio};
use std::io::{Write};
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

#[derive(Debug, PartialEq)]
pub enum ExecuteError {
    Busy,
    Quit,
}

impl GDB {
    pub fn spawn(executable_path: &str, process_tty_name: &str) -> Result<(GDB, mpsc::Receiver<output::OutOfBandRecord>), ::std::io::Error> {
        let mut child = try!{Command::new("/bin/gdb")
            .arg("--interpreter=mi")
            .arg(format!("--tty={}", process_tty_name))
            .arg(executable_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()};
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let is_running = Arc::new(AtomicBool::new(false));
        let is_running_for_thread = is_running.clone();
        let (result_input, result_output) = mpsc::channel();
        let (out_of_band_input, out_of_band_output) = mpsc::channel();
        /*let outputThread = */ thread::spawn(move || {
            output::process_output(stdout, result_input, out_of_band_input, is_running_for_thread);
        });
        Ok(
            (GDB {
                process: child,
                stdin: stdin,
                is_running: is_running,
                result_output: result_output,
                //outputThread: outputThread,
            },
            out_of_band_output)
          )
    }

    pub fn interrupt_execution(&self) -> Result<(), ::nix::Error> {
        use ::nix::sys::signal;
        signal::kill(self.process.id() as i32, signal::SIGINT)
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed) /* TODO: maybe some other ordering? */
    }

    pub fn execute(&mut self, command: &input::MiCommand) -> Result<output::ResultRecord, ExecuteError> {
        if self.is_running() {
            return Err(ExecuteError::Busy)
        }

        command.write_interpreter_string(&mut self.stdin).unwrap();
        write!(&mut self.stdin, "\n").unwrap();
        match self.result_output.recv() {
            Ok(record) => Ok(record),
            Err(e) => {
                println!("Execute error: {}", e);
                Err(ExecuteError::Quit)
            },
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
