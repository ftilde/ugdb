
pub mod input;
pub mod output;

use std::process::{Command,Child,ChildStdin,Stdio};
use std::io::{Write};
use std::thread;
use std::sync::mpsc;


pub struct GDB {
    pub process: Child,
    stdin: ChildStdin,
    result_output: mpsc::Receiver<output::ResultRecord>,
    //outputThread: thread::Thread,
}

#[derive(Debug, PartialEq)]
pub enum ExecuteError {
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
        let (result_input, result_output) = mpsc::channel();
        let (out_of_band_input, out_of_band_output) = mpsc::channel();
        /*let outputThread = */ thread::spawn(move || {
            output::process_output(stdout, result_input, out_of_band_input);
        });
        Ok(
            (GDB {
                process: child,
                stdin: stdin,
                result_output: result_output,
                //outputThread: outputThread,
            },
            out_of_band_output)
          )
    }

    pub fn execute(&mut self, command: &input::MiCommand) -> Result<output::ResultRecord, ExecuteError> {
        command.write_interpreter_string(&mut self.stdin).unwrap();
        write!(&mut self.stdin, "\n").unwrap();
        match self.result_output.recv() {
            Ok(record) => Ok(record),
            Err(_) => Err(ExecuteError::Quit),
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
