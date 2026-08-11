use crate::shell_core::parser::{
    lexer::{ShellLexer, Token},
    input_commands::{InputCommand, Input, Output},
};

use std::{
    path::{PathBuf},
};

#[derive(Debug)]
pub enum ParseError {
    MissingRedirectTarget,
    UnexpectedToken,
    InvalidRedirection,
    PipeFromNoProgram,
    ProgramIsEmpty,
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
    user_input: &str) -> Result<Vec<InputCommand>, ParseError> {
        let tokens = ShellLexer::tokenize_input(user_input);
        let mut command_lst = Vec::new();
        let mut command = InputCommand::new();
        for token in tokens {
            match (&self.state, token) {
                (ParserState::Command, Token::Word(word)) => {
                    command.program = String::from(word);
                    self.state = ParserState::Arg;
                },
                (ParserState::Arg, Token::Word(word)) => {
                    command.args.push(String::from(word));
                },
                (ParserState::Command, Token::FileIN) => {
                    self.state = ParserState::ExpectIn;
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
                    command.stdout = Output::Pipe;
                    command_lst.push(command);
                    command = InputCommand::new();
                    command.stdin = Input::Pipe;
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
        command_lst.push(command);
        Ok(command_lst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test_01() {
        let parser = ShellParser::new();
        let res_cmd = vec![
            InputCommand{
                program: "cat".to_string(),
                args: Vec::from(["-e".to_string(), "test".to_string()]),
                stdin: Input::Inherit,
                stdout: Output::Inherit
            }
        ];
        let result = parser
            .parse_user_command("cat -e test")
            .unwrap(); 
        assert_eq!(result, res_cmd);
    }
    #[test]
    fn basic_test_02() {
        let parser = ShellParser::new();
        let expected = vec![
            InputCommand {
                program: "cat".to_string(),
                args: vec![],
                stdin: Input::Inherit,
                stdout: Output::Inherit,
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
        let expected = vec![
            InputCommand {
                program: "echo".to_string(),
                args: vec![
                    "hello".to_string(),
                    "world".to_string(),
                ],
                stdin: Input::Inherit,
                stdout: Output::Inherit,
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

        let expected = vec![
            InputCommand {
                program: "cat".to_string(),
                args: vec![],
                stdin: Input::File("input.txt".into()),
                stdout: Output::Inherit,
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

        let expected = vec![
            InputCommand {
                program: "cat".to_string(),
                args: vec![],
                stdin: Input::Inherit,
                stdout: Output::File("output.txt".into()),
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

        let expected = vec![
            InputCommand {
                program: "cat".to_string(),
                args: vec![],
                stdin: Input::Inherit,
                stdout: Output::AppendFile("output.txt".into()),
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

        let expected = vec![
            InputCommand {
                program: "cat".to_string(),
                args: vec![
                    "-n".to_string(),
                    "input.txt".to_string(),
                ],
                stdin: Input::Inherit,
                stdout: Output::File("output.txt".into()),
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

        let expected = vec![
            InputCommand {
                program: "cat".to_string(),
                args: vec![
                    "file.txt".to_string(),
                ],
                stdin: Input::Inherit,
                stdout: Output::Pipe,
            },
            InputCommand {
                program: "grep".to_string(),
                args: vec![
                    "hello".to_string(),
                ],
                stdin: Input::Pipe,
                stdout: Output::Inherit,
            },
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

        let expected = vec![
            InputCommand {
                program: "cat".to_string(),
                args: vec!["file.txt".to_string()],
                stdin: Input::Inherit,
                stdout: Output::Pipe,
            },
            InputCommand {
                program: "grep".to_string(),
                args: vec!["hello".to_string()],
                stdin: Input::Pipe,
                stdout: Output::Pipe,
            },
            InputCommand {
                program: "wc".to_string(),
                args: vec!["-l".to_string()],
                stdin: Input::Pipe,
                stdout: Output::Inherit,
            },
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

        let expected = vec![
            InputCommand {
                program: "cat".to_string(),
                args: vec!["file.txt".to_string()],
                stdin: Input::Inherit,
                stdout: Output::Pipe,
            },
            InputCommand {
                program: "grep".to_string(),
                args: vec!["hello".to_string()],
                stdin: Input::Pipe,
                stdout: Output::File(
                    "result.txt".into()
                ),
            },
        ];

        let result = parser
            .parse_user_command(
                "cat file.txt | grep hello > result.txt"
            )
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
    fn whitespace_01() {
        let parser = ShellParser::new();

        let expected = vec![
            InputCommand {
                program: "cat".to_string(),
                args: vec![
                    "-e".to_string(),
                    "test".to_string(),
                ],
                stdin: Input::Inherit,
                stdout: Output::Inherit,
            }
        ];

        let result = parser
            .parse_user_command(
                "   cat    -e     test   "
            )
            .unwrap();

        assert_eq!(result, expected);
    }
}
