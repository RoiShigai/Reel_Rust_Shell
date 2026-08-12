use crate::shell_core::{
    parser::commands::{
    InputCommand,
    CommandOperator,
    CommandGroup,
    CommandType,
    },
    ShellCore,
    shell_conf::{ConfigError, ShellConfig},
};

use std::{
    ffi::{OsStr, CStr, CString},
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
        command_list: &mut Vec<CommandGroup>) -> Result<(), ConfigError<'_>> {
        for commandgroup in command_list {
            for input_command in &mut commandgroup.command {
                input_command.kind = Self::check_builtin(&input_command.program);
                if input_command.kind == CommandType::Unknown {
                            input_command.program = match shell_config.build_path(
                            &input_command.program) {
                            Ok(path) => path,
                            Err(e) => return Err(e),
                        };
                    }
                }
            }
        Ok(())
    }

    fn exec_group(&mut self, group: &CommandGroup) -> Result<u8, Box<dyn Error>> {
        for command in &group.command {
            todo!();
        }
        Ok(0)
    }

    fn execute_pipeline(
        &mut self,
        command_list: &mut Vec<CommandGroup>) -> Result<(), Box<dyn Error + '_>> {
        let mut last_status: u8 = 0;

        Self::resolve_path(&self.shell_config ,command_list)?;
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
                last_status = self.exec_group(group)?;
            }
        }
        Ok(())
    }
}
