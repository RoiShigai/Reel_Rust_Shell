mod executor;
mod shell_conf;
mod parser;
use crate::shell_core::shell_conf::ShellConfig;

pub struct ShellCore {
    shell_config: ShellConfig,
}

impl ShellCore {
    pub fn  new() -> ShellCore {
        ShellCore{
            shell_config: ShellConfig::new(),
        }
    }

}
