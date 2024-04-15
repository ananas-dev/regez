use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Char(char),
    RepeatRange(Option<u64>, Option<u64>),
    Repeat(u64),
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
    Hyphen,
    Error,
    Eof,
}

pub struct Scanner<'a> {
    input: &'a [char],
    current: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(input: &'a [char]) -> Scanner {
        Scanner { input, current: 0 }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut res = Vec::new();

        while !self.is_at_end() {
            let c = self.advance();

            res.push(match c {
                '|' => Token::Union,
                '(' => Token::LeftParen,
                ')' => Token::RightParen,
                '[' => Token::LeftBracket,
                ']' => Token::RightBracket,
                '^' => Token::Carret,
                '*' => Token::Star,
                '.' => Token::Dot,
                '?' => Token::QuestionMark,
                '+' => Token::Plus,
                '-' => Token::Hyphen,
                '{' => self.repeat(),
                c => Token::Char(c),
            });
        }

        res.push(Token::Eof);

        res
    }

    fn repeat(&mut self) -> Token {
        let mut first = 0;
        let mut second = 0;

        if self.matches(',') {
            while self.peek().is_ascii_digit() {
                second = second * 10 + self.peek() as u64 - '0' as u64;
                self.advance();
            }

            if self.matches('}') {
                return Token::RepeatRange(None, Some(second));
            }
        }

        while self.peek().is_ascii_digit() {
            first = first * 10 + self.peek() as u64 - '0' as u64;
            self.advance();
        }

        if self.matches('}') {
            return Token::Repeat(first);
        }

        if !self.matches(',') {
            return Token::Error;
        }

        if self.matches('}') {
            return Token::RepeatRange(Some(first), None);
        }

        while self.peek().is_ascii_digit() {
            second = second * 10 + self.peek() as u64 - '0' as u64;
            self.advance();
        }

        if self.matches('}') {
            return Token::RepeatRange(Some(first), Some(second));
        }

        Token::Error
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.input[self.current] != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.input[self.current - 1]
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.input.len() {
            '\0'
        } else {
            self.input[self.current + 1]
        }
    }

    fn is_at_end(&self) -> bool {
        self.current == self.input.len()
    }
}
