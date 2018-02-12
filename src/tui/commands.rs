use gdbmi::input::MiCommand;
use gdbmi::ExecuteError;

pub enum CommandState {
    Idle,
    WaitingForConfirmation(MiCommand),
}

impl CommandState {
    pub fn handle_input_line(&mut self, line: &str, p: ::UpdateParameters) {
        let mut tmp_state = CommandState::Idle;
        ::std::mem::swap(&mut tmp_state, self);
        *self = match tmp_state {
            CommandState::Idle => Self::dispatch_command(line, p),
            CommandState::WaitingForConfirmation(cmd) => Self::execute_if_confirmed(line, cmd, p),
        }
    }

    fn execute_if_confirmed(line: &str, cmd: MiCommand, p: ::UpdateParameters) -> Self {
        match line {
            "y" | "Y" | "yes" => {
                Self::execute_command(cmd, p);
                CommandState::Idle
            },
            "n" | "N" | "no" => {
                CommandState::Idle
            },
            _ => {
                p.logger.log_message("Please type 'y' or 'n'.");
                CommandState::WaitingForConfirmation(cmd)
            },
        }
    }

    fn execute_command(cmd: MiCommand, p: ::UpdateParameters) {
        match p.gdb.mi.execute(&cmd) {
            Ok(result) => {
                p.logger.log_debug(format!("Result: {:?}", result));
            },
            Err(ExecuteError::Quit) => {
                p.logger.log_message("quit");
            },
            Err(ExecuteError::Busy) => {
                p.logger.log_message("GDB is running!");
            },
        }
    }

    fn dispatch_command(line: &str, p: ::UpdateParameters) -> Self {
        let mut cmd_split = line.split(' ');
        let cmd = if let Some(cmd) = cmd_split.next() {
            cmd
        } else {
            return CommandState::Idle;
        };
        let _arguments = cmd_split.collect::<Vec<_>>();
        match cmd {
            "!stop" => {
                p.gdb.mi.interrupt_execution().expect("interrupted gdb");
                // This does not always seem to unblock gdb, but only hang it
                //use gdbmi::input::MiCommand;
                //gdb.execute(&MiCommand::exec_interrupt()).expect("Interrupt ");
                //
                CommandState::Idle
            },
            "q" => {
                let cmd = MiCommand::exit();
                match p.gdb.mi.is_session_active() {
                    Ok(true) => {
                        p.logger.log_message("A debugging session is active. Quit anyway? (y or n)");
                        CommandState::WaitingForConfirmation(cmd)
                    },
                    Ok(false) => {
                        Self::execute_command(cmd, p);
                        CommandState::Idle
                    }
                    Err(ExecuteError::Quit) => {
                        p.logger.log_message("quit");
                        CommandState::Idle
                    },
                    Err(ExecuteError::Busy) => {
                        p.logger.log_message("GDB is running!");
                        CommandState::Idle
                    },
                }
            }
            // Gdb commands
            _ => {
                Self::execute_command(MiCommand::cli_exec(line.to_owned()), p);
                CommandState::Idle
            },
        }
    }

}
