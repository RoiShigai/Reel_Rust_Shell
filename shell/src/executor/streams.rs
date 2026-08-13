use crate::executor::executor::ExecError;
use crate::parser::commands::{CommandGroup, Input, Output};
use crate::shell_config::shell_conf::ShellConfig;
use std::os::fd::{AsRawFd, OwnedFd};
use nix::libc::STDOUT_FILENO;
use nix::{
    libc::{dup2, STDIN_FILENO},
    unistd::pipe,
};

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

pub fn setup_stdout(
    env: &mut ShellConfig,
    output: &Output,
    streams: &[CommandPipe],
    index: usize) -> Result<(), ExecError> {
    match output {
        Output::Pipe => {
            let pipe = &streams[index];
            dup2(pipe.write.as_raw_fd(), STDOUT_FILENO)?;
        },
        Output::Inherit => {},
        Output::File(path) => {
            let fd = open_file(path, FileMode::Write)?;
            dup2(fd, STDOUT_FILENO)?;
        },
        Output::AppendFile(path) => {
            let fd = open_file(path, FileMode::Append)?;
            dup2(fd, STDOUT_FILENO)?;
        },
    };
    Ok(())
}

pub fn setup_stdin(
    env: &mut ShellConfig,
    input: &Input,
    streams: &[CommandPipe],
    index: usize) -> Result<(), ExecError>{
    match input {
        Input::Pipe => {
            let pipe = &streams[index - 1];
            dup2(pipe.read.as_raw_fd(), STDIN_FILENO);
        },
        Input::Inherit => {},
        Input::File(path) => {
            let fd = open_file(path, FileMode::Read)?;
            dup2(fd, STDIN_FILENO);
        },
    };
    Ok(())
}
