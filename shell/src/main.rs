use std::io::Write;
mod parser;
mod shell_config;
mod executor;
use crate::{
    parser::shell_parser::ShellParser,
    shell_config::shell_conf::ShellConfig,
};


fn main() -> Result<(), Box<dyn std::error::Error>>{
    let mut config = ShellConfig::new();
    let parser = ShellParser::new();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut prompt = String::new();
        std::io::stdin().read_line(&mut prompt)?;
        let mut ret_val = match parser.parse_user_command(&prompt) {
            Ok(commands) => commands,
            Err(e) => {
                eprintln!("Input Error: {:?}", e);
                vec![]
            },
        };
    }
}
