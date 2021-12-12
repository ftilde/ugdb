use crate::gdbmi::{
    commands::MiCommand,
    output::{ResultClass, ResultRecord},
    ExecuteError,
};
use crate::Context;

use log::error;

pub struct Command {
    cmd: Box<dyn FnMut(&mut Context) -> Result<(), ExecuteError>>,
}

impl Command {
    fn new(cmd: Box<dyn FnMut(&mut Context) -> Result<(), ExecuteError>>) -> Command {
        Command { cmd }
    }
    fn from_mi_with_msg(cmd: MiCommand, success_msg: &'static str) -> Command {
        Command::new(Box::new(move |p: &mut Context| {
            let res = p.gdb.mi.execute(cmd.clone()).map(|_| ());
            if res.is_ok() {
                p.log(success_msg);
            }
            res
        }))
    }
    fn from_mi(cmd: MiCommand) -> Command {
        Command::new(Box::new(move |p: &mut Context| {
            p.gdb.mi.execute(cmd.clone()).map(|_| ())
        }))
    }
}

pub enum CommandState {
    Idle,
    WaitingForConfirmation(Command),
}

impl CommandState {
    pub fn handle_input_line(&mut self, line: &str, p: &mut Context) {
        let mut tmp_state = CommandState::Idle;
        std::mem::swap(&mut tmp_state, self);
        *self = match tmp_state {
            CommandState::Idle => Self::dispatch_command(line, p),
            CommandState::WaitingForConfirmation(cmd) => Self::execute_if_confirmed(line, cmd, p),
        }
    }

    fn execute_if_confirmed(line: &str, cmd: Command, p: &mut Context) -> Self {
        match line {
            "y" | "Y" | "yes" => {
                Self::try_execute(cmd, p);
                CommandState::Idle
            }
            "n" | "N" | "no" => CommandState::Idle,
            _ => {
                p.log("Please type 'y' or 'n'.");
                CommandState::WaitingForConfirmation(cmd)
            }
        }
    }

    fn print_execute_error(e: ExecuteError, p: &mut Context) {
        match e {
            ExecuteError::Quit => p.log("quit"),
            ExecuteError::Busy => p.log("GDB is running!"),
        }
    }

    fn try_execute(mut cmd: Command, p: &mut Context) {
        match (cmd.cmd)(p) {
            Ok(_) => {}
            Err(e) => Self::print_execute_error(e, p),
        }
    }

    fn ask_if_session_active(
        cmd: Command,
        confirmation_question: &'static str,
        p: &mut Context,
    ) -> Self {
        match p.gdb.mi.is_session_active() {
            Ok(true) => {
                p.log(format!(
                    "A debugging session is active. {} (y or n)",
                    confirmation_question
                ));
                CommandState::WaitingForConfirmation(cmd)
            }
            Ok(false) => {
                Self::try_execute(cmd, p);
                CommandState::Idle
            }
            Err(e) => {
                Self::print_execute_error(e, p);
                CommandState::Idle
            }
        }
    }

    fn dispatch_command(line: &str, p: &mut Context) -> Self {
        let line = line.trim();
        let cmd_end = line.find(' ').unwrap_or(line.len());
        let cmd = &line[..cmd_end];
        let args_begin = (cmd_end + 1).min(line.len());
        let args_str = &line[args_begin..];
        match cmd {
            "!stop" => {
                p.gdb.mi.interrupt_execution().expect("interrupted gdb");
                // This does not always seem to unblock gdb, but only hang it
                //gdb.execute(&MiCommand::exec_interrupt()).expect("Interrupt");

                CommandState::Idle
            }
            "!layout" => {
                p.try_change_layout(args_str.to_owned());

                CommandState::Idle
            }
            "!show" => {
                p.show_file(args_str.to_owned(), unsegen::base::LineNumber::new(1));

                CommandState::Idle
            }
            "!reload" => match p.gdb.get_target() {
                Ok(Some(target)) => Self::ask_if_session_active(
                    Command::from_mi_with_msg(
                        MiCommand::file_exec_and_symbols(&target),
                        "Reloaded target.",
                    ),
                    "Reload anyway?",
                    p,
                ),
                Ok(None) => {
                    p.log("No target. Use the 'file' command to specify one.");
                    CommandState::Idle
                }
                Err(e) => {
                    Self::print_execute_error(e, p);
                    CommandState::Idle
                }
            },
            "shell" => {
                // This command does not work, because gdb breaks the gdbmi protocol (because it
                // likely just gives up stdout to the shell process until it terminates). This
                // cannot work in the curren architecture and actually just freezes ugdb. As a
                // workaround we just block this command.
                p.log("The `shell` command is not supported in ugdb. Consider suspending the process with Ctrl-z instead.");
                CommandState::Idle
            }
            "q" => {
                Self::ask_if_session_active(Command::from_mi(MiCommand::exit()), "Quit anyway?", p)
            }
            // Gdb commands
            _ => {
                match p.gdb.mi.execute(MiCommand::cli_exec(line)) {
                    Ok(ResultRecord {
                        class: ResultClass::Error,
                        results,
                        ..
                    }) => {
                        // Most of the time gdb seems to also write error messages to the console.
                        // We therefore (only) write the error message to debug log to avoid duplicates.
                        error!("{}", results["msg"].as_str().unwrap_or(&results.pretty(2)));
                    }
                    Ok(_) => {}
                    Err(e) => Self::print_execute_error(e, p),
                }
                CommandState::Idle
            }
        }
    }
}
