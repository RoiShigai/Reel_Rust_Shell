use crate::shell_core::{
    parser::commands::{
    InputCommand,
    CommandOperator,
    CommandGroup
    },
    ShellCore,
    shell_conf::ConfigError,
};

use std::{
    ffi::{CStr, CString},
    path::{PathBuf, Path},
    fs::metadata,
    os::unix::{
        ffi::OsStrExt,
        fs::PermissionsExt,
    },
    error::Error,
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

//  fn execute(&mut self, &mut command_lst: Vec<CommandGroup>) -> Result<(), Box<dyn std::error::Error>> {
//      for command in command_lst {
//         let executable = self.find_executable(
//              command.get_exec()
//          ).ok_or_else(|| {
//                  format!("{}: Command not found", command.get_exec())
//              })?;
//          let exec_cstr = CString::new(
//              executable
//                  .as_os_str()
//                  .as_bytes()
//          )?;
//          match unsafe {fork()?} {
//              ForkResult::Parent {child} => {
//                  waitpid(child, None)?;
//              }
//              ForkResult::Child => {
//                  let env_string = self.shell_config.get_c_env()?;
//                  let env: Vec<&CStr> = env_string.iter().map(
//                      |var| var.as_c_str()
//                  ).collect();
//                  execve(&exec_cstr, &command.argv(), &env)?;
//              }
//          }
//      }
//      Ok(())
//  }
    fn resolve_path(
        &mut self,
        command_list: &mut Vec<CommandGroup>) -> Result<(), ConfigError<'_>> {
        for commandgroup in command_list {
            for input_command in &mut commandgroup.command {
                input_command.program = match self.shell_config.build_path(
                    &input_command.program) {
                    Ok(path) => path,
                    Err(e) => return Err(e),
                };
            }
        }
        Ok(())
    }

    fn execute_pipeline(&mut self, command_list: &mut Vec<CommandGroup>) -> Result<(), Box<dyn Error + '_>> {
        self.resolve_path(command_list)?;
        Ok(())
    }
}
