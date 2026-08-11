pub struct ShellLexer;

pub enum Token<'a>{
    Word(&'a str),
    FileIN,
    FileOUT,
    FileAppend,
    Pipe,
    Or,
    And,
    Sequence,
}

impl ShellLexer {

    pub fn tokenize_input(input: &str) -> Vec<Token<'_>> {
        let mut tokens = Vec::new();
        let mut word_start: Option<usize> = None;
        let mut chars = input.char_indices().peekable();

        while let Some((index, ch)) = chars.next() {
            if ch.is_whitespace() {
                if let Some(start) = word_start.take() {
                    tokens.push(Token::Word(&input[start..index]));
                }
                continue;
            }
            let operator_len = match ch {
                '>' | '<' | '|' | '&' | ';' => {
                    if let Some(start) = word_start.take() {
                        tokens.push(Token::Word(&input[start..index]));
                    }
                    match ch {
                        '>' if chars.peek().is_some_and(|(_, c)| *c == '>') => {
                            chars.next();
                            2
                        }
                        '|' if chars.peek().is_some_and(|(_, c)| *c == '|') => {
                            chars.next();
                            2
                        }
                        '&' if chars.peek().is_some_and(|(_, c)| *c == '&') => {
                            chars.next();
                            2
                        }
                        _ => 1,
                    }
                }
                _ => {
                    if word_start.is_none() {
                        word_start = Some(index);
                    }
                    continue;
                }
            };
            let token_start = index;
            let operator = &input[token_start..token_start + operator_len];

            match operator {
                ">" => tokens.push(Token::FileOUT),
                "<" => tokens.push(Token::FileIN),
                ">>" => tokens.push(Token::FileAppend),
                "|" => tokens.push(Token::Pipe),
                "||" => tokens.push(Token::Or),
                "&&" => tokens.push(Token::And),
                ";" => tokens.push(Token::Sequence),
                _ => unreachable!(),
            }
        }
        if let Some(start) = word_start {
            tokens.push(Token::Word(&input[start..]));
        }
        tokens
    }
}
