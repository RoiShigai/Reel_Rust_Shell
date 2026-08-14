use crate::{
    executor::streams::{create_streams, setup_stdin, setup_stdout, CommandPipe},
    file::file::FileError,
    parser::commands::{
        CommandGroup,
        CommandOperator,
        CommandType,
        InputCommand
    },
    shell_config::shell_conf::{
        ConfigError,
        ShellConfig
    },
};

use std::{
    ffi::{CString, OsStr},
    fmt,
    os::unix::{
        ffi::OsStrExt,
    },
};

use nix::{
    errno::Errno,
    sys::wait::{waitpid, WaitStatus},
    unistd::{execve, fork, ForkResult, Pid},
};

#[derive(Debug)]
pub enum ExecError {
    FailedPipeCreation,
    InvalidPipeline,
    UnknownCommand,
    IOError(std::io::Error),
    FileError(FileError),
    ConfigError(ConfigError),
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FailedPipeCreation => write!(f, "Failed To Create Pipe"),
            Self::InvalidPipeline => write!(f, "Invalid Pipeline"),
            Self::UnknownCommand => write!(f, "Unknown Command Type"),
            Self::IOError(error) => write!(f, "StdIO Error during Execution: '{}'", error),
            Self::FileError(error) => write!(f, "FileError during execution: {}", error),
            Self::ConfigError(k) => write!(f, "ConfigError: '{}'", k),
        }
    }
}

impl From<Errno> for ExecError {
    fn from(_: Errno) -> Self {
        ExecError::FailedPipeCreation
    }
}

impl From<ConfigError> for ExecError {
    fn from(error: ConfigError) -> Self {
            ExecError::ConfigError(error)
    }
}

impl From<FileError> for ExecError {
    fn from(_: FileError) -> Self {
        ExecError::FileError(FileError::PathError)
    }
}

impl std::error::Error for ExecError {}

impl From<std::io::Error> for ExecError {
    fn from(error: std::io::Error) -> Self {
        ExecError::IOError(error)
    }
}

fn check_builtin(command: &OsStr) -> CommandType {
    match command.to_str() {
        Some("cd") => CommandType::BuiltIn,
        Some("pwd") => CommandType::BuiltIn,
        Some("export") => CommandType::BuiltIn,
        Some("unset") => CommandType::BuiltIn,
        Some("exit") => CommandType::BuiltIn,
        _ => CommandType::Unknown,
    }
}

fn resolve_path(
    shell_config: &ShellConfig,
    command_list: &mut Vec<CommandGroup>) -> Result<(), ConfigError> {
    for commandgroup in command_list {
        for input_command in &mut commandgroup.command {
            input_command.kind = check_builtin(&input_command.program);
            if input_command.kind == CommandType::Unknown {
                        input_command.program = match shell_config.build_path(
                        &input_command.program) {
                        Ok(path) => {
                            input_command.kind = CommandType::Executable;
                            path
                        }
                        Err(e) => return Err(e),
                    };
                }
            }
        }
    Ok(())
}

fn exec_child(
    env: &mut ShellConfig,
    command: &InputCommand,
    streams: &Vec<CommandPipe>,
    index: usize,
) -> ! {

    if let Err(error) =
        setup_stdin(env, &command.stdin, streams, index)
    {
        eprintln!("stdin setup failed: {error}");
        std::process::exit(1);
    }

    if let Err(error) =
        setup_stdout(env, &command.stdout, streams, index)
    {
        eprintln!("stdout setup failed: {error}");
        std::process::exit(1);
    }

    let c_env = match env.get_c_env() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("environment error: {error}");
            std::process::exit(1);
        }
    };

    let c_exec = match CString::new(
        command.program.as_os_str().as_bytes()
    ) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid executable: {error}");
            std::process::exit(1);
        }
    };
    println!("Child - Exec");
    println!("command: {:?}", command.program);
    println!("argv: {:?}", command.argv());
    let argv = match command.argv() {
            Ok(argv) => argv,
            Err(error) => {
                eprintln!("shell: invalid arguments: {error}");
                std::process::exit(1);
            }
        };
    match execve(&c_exec, &argv, &c_env) {
            Ok(never) => match never {},
            Err(error) => {
                eprintln!(
                    "shell: failed to execute {}: {}",
                    command.program.to_string_lossy(),
                    error
                );
                std::process::exit(127);
            }
        };
    unreachable!();
}

fn exec_command(
    env: &mut ShellConfig,
    command: &InputCommand,
    streams: &Vec<CommandPipe>,
    index: usize) -> Result<Pid, ExecError>{
    
    match command.kind {
        CommandType::BuiltIn => {
            todo!();
        }
        CommandType::Executable => {
            match unsafe {fork()? } {
                ForkResult::Parent {child} => {
                    println!("OK - Pipe parent");
                    Ok(child)
                },
                ForkResult::Child => exec_child(env, command, streams, index),
            }
        }
        _ => {
            Err(ExecError::UnknownCommand)
        }
    }
}

fn exec_group(env:&mut ShellConfig, group: &CommandGroup) -> Result<i32, ExecError> {
    let streams = create_streams(group)?;
    let mut children = Vec::new();
    let mut last_status: i32 = 0;

    for (index, command) in group.command.iter().enumerate() {
        let child = exec_command(env, command, &streams, index)?;
        children.push(child);
    }
    drop(streams);
    for child in children {
        match waitpid(child, None)? {
            WaitStatus::Exited(_, status) => {
                last_status = status;
            }
            WaitStatus::Signaled(_, signal, _) => {
                last_status = 128 + signal as i32;
            }
            _ => {}
        }
    }
    Ok(last_status)
}

pub fn execute_pipeline(
    env:&mut ShellConfig,
    command_list: &mut Vec<CommandGroup>) -> Result<i32, ExecError> {
    let mut last_status: i32 = 0;

    resolve_path(env ,command_list)?;
    for group in command_list {
        let should_exec = match group.next {
            None => true,
            Some(CommandOperator::Sequence) => true,
            Some(CommandOperator::Or) => {
                last_status != 0 
            },
            Some(CommandOperator::And) => {
                last_status == 0 
            }
        };
        if should_exec {
            last_status = exec_group(env, group)?;
        }
    }
    Ok(last_status)
}
