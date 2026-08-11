use crate::shell_core::parser::input_commands::InputCommand;

pub struct Executor {
    command_lst: Vec<InputCommand>,
}

impl Executor {
    pub fn new() -> Self {
        Executor{
            command_lst: Vec::new(),
        }
    }
}
