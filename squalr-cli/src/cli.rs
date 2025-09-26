use crate::response_handlers::handle_engine_response;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::commands::engine_command::EngineCommand;
use std::io;
use std::io::Write;
use std::sync::Arc;
use structopt::StructOpt;

pub struct Cli {}

/// Implements a command line listener polls for text input commands to control the engine.
impl Cli {
    pub fn run_loop(engine_execution_context: &Arc<EngineExecutionContext>) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        loop {
            if let Err(error) = stdout.flush() {
                log::error!("Error flushing stdout {}", error);
                break;
            }

            let mut input = String::new();
            if let Err(error) = stdin.read_line(&mut input) {
                log::error!("Error reading input {}", error);
                break;
            }

            if !Self::handle_input(engine_execution_context, input.trim()) {
                break;
            }
        }
    }

    pub fn stay_alive() {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        if let Err(error) = stdout.flush() {
            log::error!("Error flushing stdout {}", error);
            return;
        }

        let mut input = String::new();
        let _ = stdin.read_line(&mut input);
        log::error!("Exiting cli.");
    }

    fn handle_input(
        engine_execution_context: &Arc<EngineExecutionContext>,
        input: &str,
    ) -> bool {
        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("close") || input.eq_ignore_ascii_case("quit") {
            return false;
        }

        let mut cli_command = match shlex::split(input) {
            Some(cli_command) => cli_command,
            None => {
                log::error!("Error parsing input");
                return true;
            }
        };

        if cli_command.is_empty() {
            return true;
        }

        // Little bit of a hack, but our command system seems to require the first command to be typed twice so just insert it.
        // We could structopt(flatten) our commands to avoid this, but then this creates even stranger command conflict issues.
        cli_command.insert(0, cli_command[0].clone());

        let engine_command = match EngineCommand::from_iter_safe(&cli_command) {
            Ok(engine_command) => engine_command,
            Err(error) => {
                log::error!("Error parsing engine command: {}", error);
                return true;
            }
        };

        engine_execution_context.dispatch_command(engine_command, |engine_command| {
            handle_engine_response(engine_command);
        });

        true
    }
}
