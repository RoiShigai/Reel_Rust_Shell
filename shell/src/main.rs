use std::io::Write;
mod shell_core;
use crate::shell_core::shell_core::ShellCore;

fn main() {
    let mut shell = ShellCore::new();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut prompt = String::new();
        std::io::stdin().read_line(&mut prompt).expect(
            "Failed to read line"
        );
        shell.process_input(&prompt);
    }
}
