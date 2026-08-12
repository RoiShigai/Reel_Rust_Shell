use crate::shell_core::{
    parser::commands::{
    InputCommand,
    CommandOperator,
    CommandGroup
    },
    ShellCore,
};

use std::{
    ffi::{CStr, CString},
    path::PathBuf,
    os::unix::{
        ffi::OsStrExt,
        fs::PermissionsExt
    },
};

use nix::{
    unistd::{execve, fork, ForkResult},
    sys::wait::waitpid,
};


fn is_executable(file: &Path) -> bool {
    let metadata = match metadata(file) {
        Ok(metadata) => metadata,
        Err(_) => return false,
    };
    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
}

impl  ShellCore {

    fn execute(&mut self, &mut command_lst: Vec<CommandGroup>) -> Result<(), Box<dyn std::error::Error>> {
        for command in command_lst {
           let executable = self.find_executable(
                command.get_exec()
            ).ok_or_else(|| {
                    format!("{}: Command not found", command.get_exec())
                })?;
            let exec_cstr = CString::new(
                executable
                    .as_os_str()
                    .as_bytes()
            )?;
            match unsafe {fork()?} {
                ForkResult::Parent {child} => {
                    waitpid(child, None)?;
                }
                ForkResult::Child => {
                    let env_string = self.shell_config.get_c_env()?;
                    let env: Vec<&CStr> = env_string.iter().map(
                        |var| var.as_c_str()
                    ).collect();
                    execve(&exec_cstr, &command.argv(), &env)?;
                }
            }
        }
        Ok(())
    }

    fn find_executable(&self, program: &String) -> Option<PathBuf> {
        let path = self.shell_config.return_exec_path()?;

        for directory in std::env::split_paths(path) {
            let candidate = directory.join(program);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }
}
