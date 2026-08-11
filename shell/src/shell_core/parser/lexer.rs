pub struct ShellLexer;

pub enum Token<'a>{
    Word(&'a str),
    FileIN,
    FileOUT,
    FileAppend,
    Pipe,
}

impl ShellLexer {
    pub fn tokenize_input(input: &str) -> Vec<Token<'_>> {
        let mut res = Vec::new();

        for token in input.split_whitespace() {
            match token {
                ">" => res.push(Token::FileOUT),
                "<" => res.push(Token::FileIN),
                ">>" => res.push(Token::FileAppend),
                "|" => res.push(Token::Pipe),
                _ => res.push(Token::Word(token))
            }
        }
        res
    }
}
