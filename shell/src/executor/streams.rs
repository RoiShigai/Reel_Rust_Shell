use crate::executor::executor::ExecError;
use crate::parser::commands::{CommandGroup, Input, Output};
use crate::shell_config::shell_conf::ShellConfig;
use std::os::fd::OwnedFd;
use nix::unistd::pipe;

pub struct CommandPipe {
    read: OwnedFd,
    write: OwnedFd,
}

pub fn create_streams(groups: &CommandGroup) -> Result<Vec<CommandPipe>, ExecError>{
    let mut streams = Vec::new();

    for command in groups.command.windows(2) {
        let current_cmd = &command[0];
        let next_cmd = &command[1];

        match (&current_cmd.stdout, &next_cmd.stdin) {
            (Output::Pipe, Input::Pipe) => {
                let (read, write) = pipe()?;

                streams.push(
                    CommandPipe {
                        read, write
                    }
                );
            },
            (Output::Pipe, _) | (_, Input::Pipe) => {
                return Err(ExecError::InvalidPipeline);
            },
            _ => {}
        }
    }
    Ok(streams)
}

fn setup_stdin(env: &mut ShellConfig, input: Input) {
    match input {
        Input::Pipe => {
            
        }
    }
}
