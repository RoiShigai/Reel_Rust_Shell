use std::{
    ffi::{CString},
    path::{PathBuf},
};
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
pub struct InputCommand {
   pub program: String,
   pub args: Vec<String>,
   pub stdin: Input,
   pub stdout: Output,
}

impl InputCommand {
    pub fn new() -> Self {
        InputCommand {
            program: String::new(),
            args: Vec::new(),
            stdin: Input::Inherit,
            stdout: Output::Inherit
        }
    }
    pub fn argv(&self) -> Vec<CString>{
        self.args
            .iter()
            .map(
            |arg| CString::new(
                arg.as_str()
            )).collect::<Result<_, _>>()
            .expect("Failed to convert args into C string")
    }

    pub fn get_exec(&self) -> &String {
        &self.args[0]
    }
}
