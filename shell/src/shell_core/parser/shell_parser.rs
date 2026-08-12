use crate::shell_core::parser::{
    lexer::{ShellLexer, Token},
    commands::{
        CommandGroup,
        InputCommand,
        Input,
        Output
    },
};

use std::{
    error::Error,
    ffi::OsString,
    fmt,
    path::PathBuf
};

#[derive(Debug)]
pub enum ParseError {
    MissingRedirectTarget,
    UnexpectedToken,
    InvalidRedirection,
    PipeFromNoProgram,
    ProgramIsEmpty,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidRedirection => write!(f, "Parser Error: Invalid stream I/O in command"),
            ParseError::MissingRedirectTarget => write!(f, "Parser Error: Missing target value for Stream Redirection"),
            ParseError::UnexpectedToken => write!(f, "Parser Error: UnexpectedToken found in command"),
            ParseError::ProgramIsEmpty => write!(f, "Parser Error: No program Extract from input"),
            ParseError::PipeFromNoProgram => write!(f, "Parser Error: Pipe has no valid target"),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseError::InvalidRedirection => Some(&ParseError::InvalidRedirection),
            ParseError::MissingRedirectTarget => Some(&ParseError::MissingRedirectTarget),
            ParseError::UnexpectedToken => Some(&ParseError::UnexpectedToken),
            ParseError::ProgramIsEmpty => Some(&ParseError::ProgramIsEmpty),
            ParseError::PipeFromNoProgram => Some(&ParseError::PipeFromNoProgram),
        }
    }
}

#[derive(Debug)]
enum ParserState {
    Command,
    Arg,
    ExpectOut,
    ExpectIn,
    ExpectAppend,
}

pub struct ShellParser {
    state: ParserState,
}

impl ShellParser {

    pub fn new() -> Self {
        ShellParser{
            state: ParserState::Command,
        }
    }

    pub fn parse_user_command(
        mut self,
    user_input: &str) -> Result<Vec<CommandGroup>, ParseError> {
        let tokens = ShellLexer::tokenize_input(user_input);
        let mut command_lst = Vec::new();
        let mut commandgroup = CommandGroup::new();
        let mut command = InputCommand::new();
        for token in tokens {
            match (&self.state, token) {
                (ParserState::Command, Token::Word(word)) => {
                    command.program = OsString::from(word);
                    self.state = ParserState::Arg;
                },
                (ParserState::Arg, Token::Word(word)) => {
                    command.args.push(String::from(word));
                },
                (ParserState::Arg, Token::FileIN) => {
                    self.state = ParserState::ExpectIn;
                },
                (ParserState::ExpectIn, Token::Word(word)) => {
                    if !matches!(command.stdin, Input::Inherit) {
                        return Err(ParseError::InvalidRedirection);
                    }
                    command.stdin = Input::File(
                        PathBuf::from(String::from(word))
                    );
                    self.state = ParserState::Arg;
                },
                (ParserState::Arg, Token::FileOUT) => {
                    if !matches!(command.stdout, Output::Inherit) {
                        return Err(ParseError::InvalidRedirection);
                    }
                    self.state = ParserState::ExpectOut;
                },
                (ParserState::ExpectOut, Token::Word(word)) => {
                    command.stdout = Output::File(
                        PathBuf::from(String::from(word))
                    );
                    self.state = ParserState::Arg;
                },
                (ParserState::Arg, Token::FileAppend) => {
                    if !matches!(command.stdout, Output::Inherit) {
                        return Err(ParseError::InvalidRedirection);
                    }
                    self.state = ParserState::ExpectAppend;
                },
                (ParserState::ExpectAppend, Token::Word(word)) => {
                        command.stdout = Output::AppendFile(
                        PathBuf::from(String::from(word))
                    );
                    self.state = ParserState::Arg;
                },
                (ParserState::Arg, Token::Pipe) => {
                    if command.program.is_empty() {
                        return Err(ParseError::PipeFromNoProgram)
                    }
                    if !matches!(command.stdout, Output::Inherit) {
                        return Err(ParseError::InvalidRedirection);
                    }
                    command.stdout = Output::Pipe;
                    commandgroup.add_command(command);
                    command = InputCommand::new();
                    command.stdin = Input::Pipe;
                    self.state = ParserState::Command;
                },
                (ParserState::Arg, token @ (
                    Token::And 
                    | Token::Or 
                    | Token::Sequence)) => {
                    if !matches!(command.stdout, Output::Inherit) 
                    | matches!(command.stdin, Input::File(_)) {
                        return Err(ParseError::InvalidRedirection);
                    }
                    commandgroup.add_command(command);
                    commandgroup.set_operator(token)?; 
                    command_lst.push(commandgroup);
                    command = InputCommand::new();
                    commandgroup = CommandGroup::new();
                    self.state = ParserState::Command;
                },
                _ => return Err(ParseError::UnexpectedToken),
            }
        };
        match self.state {
            ParserState::ExpectIn
            | ParserState::ExpectOut
            | ParserState::ExpectAppend => {
                return Err(ParseError::MissingRedirectTarget);
            }
            _ => {}
        };
        if command.program.is_empty() {
            return Err(ParseError::ProgramIsEmpty);
        }
        commandgroup.add_command(command);
        command_lst.push(commandgroup);
        Ok(command_lst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell_core::parser::commands::CommandOperator;

    #[test]
    fn basic_test_01() {
        let parser = ShellParser::new();
        let input = vec![
            InputCommand{
                program: OsString::from("cat"),
                args: Vec::from(["-e".to_string(), "test".to_string()]),
                stdin: Input::Inherit,
                stdout: Output::Inherit
            }
        ];
        let expected = vec![
            CommandGroup {
                command: input,
                next: None,
            }
        ];
        let result = parser
            .parse_user_command("cat -e test")
            .unwrap(); 
        assert_eq!(result, expected);
    }
    #[test]
    fn basic_test_02() {
        let parser = ShellParser::new();
        let input = vec![
            InputCommand {
                program: OsString::from("cat"),
                args: vec![],
                stdin: Input::Inherit,
                stdout: Output::Inherit,
            }
        ];
        let expected = vec![
            CommandGroup {
                command: input,
                next: None,
            }
        ];
        let result = parser
            .parse_user_command("cat")
            .unwrap();
        assert_eq!(result, expected);
    }
    #[test]
    fn basic_test_03() {
        let parser = ShellParser::new();
        let input = vec![
            InputCommand {
                program: OsString::from("echo"),
                args: vec![
                    "hello".to_string(),
                    "world".to_string(),
                ],
                stdin: Input::Inherit,
                stdout: Output::Inherit,
            }
        ];
        let expected = vec![
            CommandGroup {
                command: input,
                next: None,
            }
        ];
        let result = parser
            .parse_user_command("echo hello world")
            .unwrap();
        assert_eq!(result, expected);
    }
    #[test]
    fn redirect_input_01() {
        let parser = ShellParser::new();

        let input = vec![
            InputCommand {
                program: OsString::from("cat"),
                args: vec![],
                stdin: Input::File("input.txt".into()),
                stdout: Output::Inherit,
            }
        ];
        let expected = vec![
            CommandGroup {
                command: input,
                next: None,
            }
        ];
        let result = parser
            .parse_user_command("cat < input.txt")
            .unwrap();

        assert_eq!(result, expected);
    }
    #[test]
    fn redirect_output_01() {
        let parser = ShellParser::new();

        let input = vec![
            InputCommand {
                program: OsString::from("cat"),
                args: vec![],
                stdin: Input::Inherit,
                stdout: Output::File("output.txt".into()),
            }
        ];
        let expected = vec![
            CommandGroup {
                command: input,
                next: None,
            }
        ];
        let result = parser
            .parse_user_command("cat > output.txt")
            .unwrap();

        assert_eq!(result, expected);
    }
    #[test]
    fn redirect_append_01() {
        let parser = ShellParser::new();

        let input = vec![
            InputCommand {
                program: OsString::from("cat"),
                args: vec![],
                stdin: Input::Inherit,
                stdout: Output::AppendFile("output.txt".into()),
            }
        ];
        let expected = vec![
            CommandGroup {
                command: input,
                next: None,
            }
        ];
        let result = parser
            .parse_user_command("cat >> output.txt")
            .unwrap();

        assert_eq!(result, expected);
    }
    #[test]
    fn redirect_output_02() {
        let parser = ShellParser::new();

        let input = vec![
            InputCommand {
                program: OsString::from("cat"),
                args: vec![
                    "-n".to_string(),
                    "input.txt".to_string(),
                ],
                stdin: Input::Inherit,
                stdout: Output::File("output.txt".into()),
            }
        ];
        let expected = vec![
            CommandGroup {
                command: input,
                next: None,
            }
        ];
        let result = parser
            .parse_user_command(
                "cat -n input.txt > output.txt"
            )
            .unwrap();

        assert_eq!(result, expected);
    }
    #[test]
    fn pipe_01() {
        let parser = ShellParser::new();

        let input = vec![
            InputCommand {
                program: OsString::from("cat"),
                args: vec![
                    "file.txt".to_string(),
                ],
                stdin: Input::Inherit,
                stdout: Output::Pipe,
            },
            InputCommand {
                program: OsString::from("grep"),
                args: vec![
                    "hello".to_string(),
                ],
                stdin: Input::Pipe,
                stdout: Output::Inherit,
            },
        ];
        let expected = vec![
            CommandGroup {
                command: input,
                next: None,
            }
        ];
        let result = parser
            .parse_user_command(
                "cat file.txt | grep hello"
            )
            .unwrap();

        assert_eq!(result, expected);
    }
    #[test]
    fn pipe_02() {
        let parser = ShellParser::new();

        let input = vec![
            InputCommand {
                program: OsString::from("cat"),
                args: vec!["file.txt".to_string()],
                stdin: Input::Inherit,
                stdout: Output::Pipe,
            },
            InputCommand {
                program: OsString::from("grep"),
                args: vec!["hello".to_string()],
                stdin: Input::Pipe,
                stdout: Output::Pipe,
            },
            InputCommand {
                program: OsString::from("wc"),
                args: vec!["-l".to_string()],
                stdin: Input::Pipe,
                stdout: Output::Inherit,
            },
        ];
        let expected = vec![
            CommandGroup {
                command: input,
                next: None,
            }
        ];
        let result = parser
            .parse_user_command(
                "cat file.txt | grep hello | wc -l"
            )
            .unwrap();

        assert_eq!(result, expected);
    }
    #[test]
    fn pipe_redirect_01() {
        let parser = ShellParser::new();

        let input = vec![
            InputCommand {
                program: OsString::from("cat"),
                args: vec!["file.txt".to_string()],
                stdin: Input::Inherit,
                stdout: Output::Pipe,
            },
            InputCommand {
                program: OsString::from("grep"),
                args: vec!["hello".to_string()],
                stdin: Input::Pipe,
                stdout: Output::File(
                    "result.txt".into()
                ),
            },
        ];
        let expected = vec![
            CommandGroup {
                command: input,
                next: None,
            }
        ];
        let result = parser
            .parse_user_command(
                "cat file.txt | grep hello > result.txt"
            )
            .unwrap();

        assert_eq!(result, expected);
    }
    #[test]
    fn basic_group_01() {
        let parser = ShellParser::new();
        let input_1 = vec![
            InputCommand{
                program: OsString::from("cat"),
                args: Vec::from(["-e".to_string(), "test".to_string()]),
                stdin: Input::Inherit,
                stdout: Output::Inherit
            }
        ];
        let input_2 = vec![
            InputCommand {
                program: OsString::from("echo"),
                args: vec![
                    "hello".to_string(),
                    "world".to_string(),
                ],
                stdin: Input::Inherit,
                stdout: Output::Inherit,
            }
        ];
        let expected = vec![
            CommandGroup {
                command: input_1,
                next: Some(CommandOperator::Or),
            },
            CommandGroup {
                command: input_2,
                next: None,
            }
        ];
        let result = parser
            .parse_user_command("cat -e test || echo hello world")
            .unwrap(); 
        assert_eq!(result, expected);
    }
    #[test]
    fn basic_group_02() {
        let parser = ShellParser::new();
        let input_1 = vec![
            InputCommand{
                program: OsString::from("cat"),
                args: Vec::from(["-e".to_string(), "test".to_string()]),
                stdin: Input::Inherit,
                stdout: Output::Inherit
            }
        ];
        let input_2 = vec![
            InputCommand {
                program: OsString::from("echo"),
                args: vec![
                    "hello".to_string(),
                    "world".to_string(),
                ],
                stdin: Input::Inherit,
                stdout: Output::Inherit,
            }
        ];
        let expected = vec![
            CommandGroup {
                command: input_1,
                next: Some(CommandOperator::And),
            },
            CommandGroup {
                command: input_2,
                next: None,
            }
        ];
        let result = parser
            .parse_user_command("cat -e test && echo hello world")
            .unwrap(); 
        assert_eq!(result, expected);
    }
    #[test]
    fn basic_group_03() {
        let parser = ShellParser::new();
        let input_1 = vec![
            InputCommand{
                program: OsString::from("cat"),
                args: Vec::from(["-e".to_string(), "test".to_string()]),
                stdin: Input::Inherit,
                stdout: Output::Inherit
            }
        ];
        let input_2 = vec![
            InputCommand {
                program: OsString::from("echo"),
                args: vec![
                    "hello".to_string(),
                    "world".to_string(),
                ],
                stdin: Input::Inherit,
                stdout: Output::Inherit,
            }
        ];
        let expected = vec![
            CommandGroup {
                command: input_1,
                next: Some(CommandOperator::Sequence),
            },
            CommandGroup {
                command: input_2,
                next: None,
            }
        ];
        let result = parser
            .parse_user_command("cat -e test ; echo hello world")
            .unwrap(); 
        assert_eq!(result, expected);
    }
    #[test]
    fn syntax_error_01() {
        let parser = ShellParser::new();

        let result = parser
            .parse_user_command("cat |");

        assert!(result.is_err());
    }
    #[test]
    fn syntax_error_02() {
        let parser = ShellParser::new();

        let result = parser
            .parse_user_command("| cat");

        assert!(result.is_err());
    }
    #[test]
    fn syntax_error_03() {
        let parser = ShellParser::new();

        let result = parser
            .parse_user_command("cat <");

        assert!(result.is_err());
    }
    #[test]
    fn syntax_error_04() {
        let parser = ShellParser::new();

        let result = parser
            .parse_user_command("cat >");

        assert!(result.is_err());
    }
    #[test]
    fn syntax_error_05() {
        let parser = ShellParser::new();

        let result = parser
            .parse_user_command(
                "cat > file1 > file2"
            );

        assert!(result.is_err());
    }
    #[test]
    fn syntax_error_06() {
        let parser = ShellParser::new();

        let result = parser
            .parse_user_command(
                "cat < file1 < file2"
            );

        assert!(result.is_err());
    }
    #[test]
    fn syntax_error_07() {
        let parser = ShellParser::new();

        let result = parser
            .parse_user_command(
                "cat > file1 | grep hello"
            );

        assert!(result.is_err());
    }
    #[test]
    fn whitespace_01() {
        let parser = ShellParser::new();

        let input = InputCommand {
            program: OsString::from("cat"),
            args: vec![
                "-e".to_string(),
                "test".to_string(),
            ],
            stdin: Input::Inherit,
            stdout: Output::Inherit,
        };
        let expected = vec![CommandGroup {
            command: vec![input],
            next: None,
        },
        ];

        let result = parser
            .parse_user_command(
                "   cat    -e     test   "
            )
            .unwrap();

        assert_eq!(result, expected);
    }
}
