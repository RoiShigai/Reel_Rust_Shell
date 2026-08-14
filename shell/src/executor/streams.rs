use crate::executor::executor::ExecError;
use crate::file::file::FileMode;
use crate::parser::commands::{CommandGroup, Input, Output};
use crate::shell_config::shell_conf::ShellConfig;
use crate::file::file::open_file;

use std::os::fd::{AsFd, OwnedFd};
use nix::{
    unistd::{pipe, dup2_stdin, dup2_stdout},
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
    streams: &Vec<CommandPipe>,
    index: usize) -> Result<(), ExecError> {
    match output {
        Output::Pipe => {
            let pipe = &streams[index];
            dup2_stdout(pipe.write.as_fd())?;
        },
        Output::Inherit => {},
        Output::File(path) => {
            let fd = open_file(path.to_path_buf(), FileMode::Write)?;
            dup2_stdout(fd)?;
        },
        Output::AppendFile(path) => {
            let fd = open_file(path.to_path_buf(), FileMode::Append)?;
            dup2_stdout(fd)?;
        },
    };
    Ok(())
}

pub fn setup_stdin(
    env: &mut ShellConfig,
    input: &Input,
    streams: &Vec<CommandPipe>,
    index: usize) -> Result<(), ExecError>{
    match input {
        Input::Pipe => {
            let pipe = &streams[index - 1];
            dup2_stdin(pipe.read.as_fd())?;
        },
        Input::Inherit => {},
        Input::File(path) => {
            println!("Streams Stdin File");
            let fd = open_file(path.to_path_buf(), FileMode::Read)?;
            dup2_stdin(fd)?;
        },
    };
    Ok(())
}
