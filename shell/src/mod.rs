pub mod executor;
pub mod shell_config;
pub mod parser;
use crate::shell_config::ShellConfig;

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
