pub struct ShellParser;

use std::{
    ffi::{CString, OsString},
    os::unix::fs::PermissionsExt,
    fs::metadata,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct InputCommand {
    pub args: Vec<String>,
}

impl InputCommand {
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

fn is_executable(file: &Path) -> bool {
    let metadata = match metadata(file) {
        Ok(metadata) => metadata,
        Err(_) => return false,
    };
    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
}

impl ShellParser {

    fn find_executable(
        self,
        program_name: PathBuf) -> Option<PathBuf> {
        if is_executable(Path::new(&program_name)) {
            return Some(program_name);
        }
        None
    }

    pub fn parse_user_command(
        self,
        user_input: &str,
        exec_path: &OsString) -> InputCommand {
        let exec: PathBuf;

        for directory in std::env::split_paths(exec_path) {
            match self.find_executable(directory.join(program_name)) {
                Some(program) => exec = program,
                _ => continue,
            } 
        }

        InputCommand {
            args: [
                String::from("Bite"),
                String::from("Chatte")
            ].to_vec()
        }
    }
}
