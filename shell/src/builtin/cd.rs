use crate::{
    executor::executor::ExecError,
    shell_config::shell_conf::{ShellConfig, ConfigError},
};

use std::{
    ffi::OsString,
};

fn set_home(config: &mut ShellConfig) -> Result<(), ExecError> {
    let env = config.get_env();
    let home = env
        .get(&OsString::from("HOME"))
        .cloned()
        .ok_or_else(|| {
            ExecError::ConfigError(
                ConfigError::EnvKeyNotFound("HOME".into())
            )
        })?;
    let old_pwd = env
        .get(&OsString::from("PWD"))
        .cloned()
        .ok_or_else(|| {
            ExecError::ConfigError(
                ConfigError::EnvKeyNotFound("PWD".into())
            )
        })?;

    env.insert(OsString::from("PWD"), home.to_os_string());
    env.insert(OsString::from("OLDPWD"), old_pwd.to_os_string());
    Ok(())
}


pub fn change_directory(
    config: &mut ShellConfig,
    args: Vec<OsString>) -> Result<(), ExecError> {
    match args.len() {
        0 => set_home(config)?,
        1 => set_path(args, config)?,
        _ => Err(ExecError::ArgError(String::from("cd: Too many arguments")))
    };
}
