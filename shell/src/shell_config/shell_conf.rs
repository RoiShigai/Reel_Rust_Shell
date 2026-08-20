use std::{
    collections::HashMap,
    ffi::{CString, OsString, OsStr},
    fmt,
    error::Error,
    os::unix::ffi::OsStrExt,
};
 use crate::file::file::is_executable;

pub struct ShellConfig {
    env: HashMap<OsString, OsString>,
}

#[derive(Debug)]
pub enum ConfigError {
    EnvKeyNotFound(String),
    CommandNotFound,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::EnvKeyNotFound(k) => write!(f, "Config Error: '{}' not found in Env", k),
            ConfigError::CommandNotFound => write!(f, "Config Error: Command Not Found in Path"),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::CommandNotFound => Some(&ConfigError::CommandNotFound),
            _ => None
        }
    }
}

impl ShellConfig {
    pub fn new() -> Self {
        let env_keys = std::env::vars_os().collect();
        ShellConfig {
            env: env_keys,
        }        
    }

    pub fn build_path(&self, program_name: &OsStr) -> Result<OsString, ConfigError> {
        match self.return_exec_path() {
            Ok(path) => {
                for directory in std::env::split_paths(path) {
                    let candidate = directory.join(program_name);
                    if is_executable(&candidate) {
                        return Ok(OsString::from(candidate));
                    }
                }
                Err(ConfigError::CommandNotFound)
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_env(&mut self) -> &mut HashMap<OsString, OsString> {
        &mut self.env
    }

    pub fn return_exec_path(&self) -> Result<&OsString, ConfigError> {
        match self.env.get(&OsString::from("PATH")) {
            Some(path) => Ok(path),
            None => Err(ConfigError::EnvKeyNotFound(String::from("PATH"))),
        }
    }

    pub fn get_c_env(&self) -> Result<Vec<CString>, Box<dyn std::error::Error>> {
        self.env.iter().map(|(key, value)| {
            let mut env = key.as_os_str().as_bytes().to_vec();
            env.push(b'=');
            env.extend_from_slice(
                value.as_os_str().as_bytes()
            );
            Ok(CString::new(env)?)
        }).collect()
    }
}
