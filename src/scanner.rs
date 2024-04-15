use std::str::Chars;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Char(char),
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Carret,
    Star,
    Union,
    Dot,
    QuestionMark,
    Plus,
    Error,
    Eof,
}

pub struct Scanner<'a> {
    input: Chars<'a>,
}

impl<'a> Scanner<'a> {
    pub fn new(input: Chars<'a>) -> Scanner {
        Scanner {
            input
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut res = Vec::new();

        loop {
            match self.input.next() {
                Some(c) => res.push(match c {
                    '|' => Token::Union,
                    '(' =>Token::LeftParen,
                    ')' => Token::RightParen,
                    '[' => Token::LeftBracket,
                    ']' => Token::RightBracket,
                    '^' => Token::Carret,
                    '*' => Token::Star,
                    '.' => Token::Dot,
                    '?' => Token::QuestionMark,
                    '+' => Token::Plus,
                    '0'..='9' | 'a'..='z'| 'A'..='Z' => Token::Char(c),
                    _ => Token::Error,
                    
                }),
                None => {
                    res.push(Token::Eof);
                    break;
                }
            }
        }

        res
    }
}