use std::{
    collections::HashMap,
    ffi::{CString, OsString},
    os::unix::ffi::OsStrExt
};

pub struct ShellConfig {
    env: HashMap<OsString, OsString>,
}

impl ShellConfig {
    pub fn new() -> Self {
        let env_keys = std::env::vars_os().collect();
        ShellConfig {
            env: env_keys,
        }        
    }

    pub fn return_exec_path(&self) -> Option<&OsString> {
        let exec_path = self.env.get(&OsString::from("PATH"));
        exec_path
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
