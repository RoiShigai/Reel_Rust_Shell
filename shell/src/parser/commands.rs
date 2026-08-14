use std::{
    ffi::{CString, OsString, NulError},
    path::{PathBuf},
};

use crate::parser::{
    lexer::Token,
    shell_parser::ParseError,
};

#[derive(Debug, PartialEq)]
pub enum CommandOperator {
    Or,
    And,
    Sequence,
}

#[derive(Debug, PartialEq)]
pub struct CommandGroup {
    pub command: Vec<InputCommand>,
    pub next: Option<CommandOperator>,
}

impl CommandGroup {
    pub fn new() -> Self {
        CommandGroup {
            command: Vec::new(),
            next: None,
        }
    }

    pub fn set_operator(&mut self, token: Token) -> Result<(), ParseError> {
        match token {
            Token::Or => self.next = Some(CommandOperator::Or),
            Token::And => self.next = Some(CommandOperator::And),
            Token::Sequence => self.next = Some(CommandOperator::Sequence),
            _ => return Err(ParseError::UnexpectedToken),
        }
        Ok(())
    }

    pub fn add_command(&mut self, command: InputCommand) {
        self.command.push(command);
    }
}

#[derive(Debug, PartialEq)]
pub enum Input {
    Inherit,
    File(PathBuf),
    Pipe,
}

#[derive(Debug, PartialEq)]
pub enum Output {
    Inherit,
    File(PathBuf),
    AppendFile(PathBuf),
    Pipe,
}

#[derive(Debug, PartialEq)]
pub enum CommandType {
    BuiltIn,
    Executable,
    Unknown
}

#[derive(Debug, PartialEq)]
pub struct InputCommand {
    pub kind: CommandType,
    pub program: OsString,
    pub args: Vec<OsString>,
    pub stdin: Input,
    pub stdout: Output,
}

impl InputCommand {
    pub fn new() -> Self {
        InputCommand {
            kind: CommandType::Unknown,
            program: OsString::new(),
            args: Vec::new(),
            stdin: Input::Inherit,
            stdout: Output::Inherit
        }
    }
    pub fn argv(&self) -> Result<Vec<CString>, NulError>{
        std::iter::once(&self.program)
            .chain(self.args.iter())
            .map(|arg| CString::new(arg.as_os_str().as_encoded_bytes()))
            .collect()
    }

    pub fn get_exec(&self) -> &OsString {
        &self.args[0]
    }
}
