use petgraph::graph::NodeIndex;

use crate::{nfa::{Nfa, Transition}, scanner::Token};

// CFG
// Expr ::= Union*
// Union ::= Kleen`|`Kleen | Kleen
// Kleen ::= Term`*` | Term
// Term :: `(` Expr `)` | char

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    nfa: Nfa,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0,
            nfa: Nfa::new(),
        }
    }

    pub fn parse(mut self) -> Nfa {
        let (start, end) = self.expr();
        self.nfa.set_start(start);
        self.nfa.make_accepting(end);
        self.nfa
    }

    pub fn expr(&mut self) -> (NodeIndex, NodeIndex) {
        let (start, mut end) = self.union();

        while !matches!(self.peek(), Token::Eof | Token::RightParen) {
            let (s1, s2) = self.union();
            self.nfa.add_e_transition(end, s1);
            end = s2
        }

        (start, end)
    }

    fn union(&mut self) -> (NodeIndex, NodeIndex) {
        let conn1 = self.kleen();

        if self.matches(Token::Union) {
            let conn2 = self.kleen();

            let s1 = self.nfa.add_state();
            let s2 = self.nfa.add_state();

            self.nfa.add_e_transition(s1, conn1.0);
            self.nfa.add_e_transition(s1, conn2.0);
            self.nfa.add_e_transition(conn1.1, s2);
            self.nfa.add_e_transition(conn2.1, s2);

            return (s1, s2)
        }

        conn1
    }

    fn kleen(&mut self) -> (NodeIndex, NodeIndex) {
        let conn = self.primary();
        
        if self.matches(Token::Star) {
            let s1 = self.nfa.add_state();
            let s2 = self.nfa.add_state();

            self.nfa.add_e_transition(s1, conn.0);
            self.nfa.add_e_transition(conn.1, s2);
            self.nfa.add_e_transition(conn.1, conn.0);
            self.nfa.add_e_transition(s1, s2);

            return (s1, s2)
        }

        if self.matches(Token::QuestionMark) {
            let s1 = self.nfa.add_state();
            let s2 = self.nfa.add_state();

            self.nfa.add_e_transition(s1, s2);
            self.nfa.add_e_transition(s1, conn.0);
            self.nfa.add_e_transition(conn.1, s2);

            return (s1, s2)
        }

        conn
    }

    fn primary(&mut self) -> (NodeIndex, NodeIndex) {
        match self.peek() {
            Token::LeftParen => {
                self.advance();
                let conn = self.expr();

                if self.matches(Token::RightParen) {
                    conn
                } else {
                    panic!("Unbalanced paren")
                }
            },
            Token::Char(c) => {
                self.advance();

                let s1 = self.nfa.add_state();
                let s2 = self.nfa.add_state();
                self.nfa.add_transition(s1, s2, Transition::Char(c));
                (s1, s2)
            },
            Token::Dot => {
                self.advance();

                let s1 = self.nfa.add_state();
                let s2 = self.nfa.add_state();
                self.nfa.add_transition(s1, s2, Transition::Any);
                (s1, s2)
            }
            _ => panic!("Invalid expression: {:?}", self.peek()),
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn matches(&mut self, token: Token) -> bool {
        if self.peek() == token {
            self.advance();
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Token {
        self.tokens[self.current]
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.peek() == Token::Eof
    }

}