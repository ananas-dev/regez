use petgraph::graph::NodeIndex;

use crate::{
    nfa::{Nfa, Transition},
    scanner::Token,
};

// CFG
// Expr ::= Concat (`|` Concat)* | Concat
// Concat ::= Duplication*
// Duplication ::= Grouping`*` | Grouping`+` | Grouping`?` | Grouping`{`(0-9)*`}` | Grouping
// Grouping ::= `(` Expr `)` | BracketExpr
// BracketExpr ::= `[` CharacterClass | `^`CharacterClass `]` | char
// CharacterClass ::=

fn merge_ranges(mut ranges: Vec<(u8, u8)>) -> Vec<(u8, u8)> {
    if ranges.is_empty() {
        return ranges;
    }

    // Sort ranges by the starting value
    ranges.sort_by(|a, b| a.0.cmp(&b.0));

    let mut merged_ranges = vec![];

    // Start with the first range
    let mut current_range = ranges[0].clone();

    for next_range in ranges.into_iter().skip(1) {
        if next_range.0 <= current_range.1 {
            // If the next range overlaps or is consecutive, merge it
            current_range.1 = current_range.1.max(next_range.1);
        } else {
            // If the next range does not overlap, push the current range and move to the next
            merged_ranges.push(current_range);
            current_range = next_range;
        }
    }

    // Push the last merged range
    merged_ranges.push(current_range);

    merged_ranges
}

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

    fn expr(&mut self) -> (NodeIndex, NodeIndex) {
        let mut conn1 = self.concat();

        while self.matches(Token::Union) {
            let conn2 = self.concat();

            let s1 = self.nfa.add_state();
            let s2 = self.nfa.add_state();

            self.nfa.add_e_transition(s1, conn1.0);
            self.nfa.add_e_transition(s1, conn2.0);
            self.nfa.add_e_transition(conn1.1, s2);
            self.nfa.add_e_transition(conn2.1, s2);

            conn1 = (s1, s2);
        }

        conn1
    }

    pub fn concat(&mut self) -> (NodeIndex, NodeIndex) {
        let (start, mut end) = self.duplication();

        while !matches!(self.peek(), Token::Eof | Token::RightParen | Token::Union) {
            let (s1, s2) = self.duplication();
            self.nfa.add_e_transition(end, s1);
            end = s2
        }

        (start, end)
    }

    fn duplication(&mut self) -> (NodeIndex, NodeIndex) {
        let conn = self.primary();

        match self.peek() {
            Token::Star => {
                self.advance();

                let s1 = self.nfa.add_state();
                let s2 = self.nfa.add_state();

                self.nfa.add_e_transition(s1, conn.0);
                self.nfa.add_e_transition(conn.1, s2);
                self.nfa.add_e_transition(conn.1, conn.0);
                self.nfa.add_e_transition(s1, s2);

                return (s1, s2);
            }
            Token::QuestionMark => {
                self.advance();

                let s1 = self.nfa.add_state();
                let s2 = self.nfa.add_state();

                self.nfa.add_e_transition(s1, s2);
                self.nfa.add_e_transition(s1, conn.0);
                self.nfa.add_e_transition(conn.1, s2);

                return (s1, s2);
            }
            Token::Plus => {
                self.advance();

                let conn2 = self.nfa.clone_subgraph(conn.0, conn.1);

                let s1 = self.nfa.add_state();
                let s2 = self.nfa.add_state();

                self.nfa.add_e_transition(conn.1, s1);
                self.nfa.add_e_transition(s1, conn2.0);
                self.nfa.add_e_transition(conn2.1, s2);
                self.nfa.add_e_transition(conn2.1, conn2.0);
                self.nfa.add_e_transition(s1, s2);

                return (conn.0, s2);
            }
            Token::Repeat(n) => {
                self.advance();

                let mut connector = conn.1;

                for _ in 0..n - 1 {
                    let new_conn = self.nfa.clone_subgraph(conn.0, conn.1);

                    self.nfa.add_e_transition(connector, new_conn.0);
                    connector = new_conn.1;
                }

                return (conn.0, connector);
            }
            Token::RepeatRange(Some(a), Some(b)) => {
                todo!()
            }

            _ => {}
        };

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
                    self.peek();
                    panic!("Unbalanced paren")
                }
            }
            Token::LeftBracket => {
                self.advance();

                let mut inclusive = true;

                if self.matches(Token::Carret) {
                    inclusive = false;
                }

                let mut ranges: Vec<(u8, u8)> = Vec::new();

                while !self.matches(Token::RightBracket) {
                    let range = self.character_class();
                    ranges.push(range);
                }

                assert!(inclusive, "Not implemented");

                let mut ranges = merge_ranges(ranges);

                let mut conn1 = {
                    let first_range = ranges.pop().unwrap();

                    let s1 = self.nfa.add_state();
                    let s2 = self.nfa.add_state();

                    self.nfa.add_transition(
                        s1,
                        s2,
                        Transition::Range(first_range.0, first_range.1),
                    );

                    (s1, s2)
                };

                for range in ranges {
                    let conn2 = {
                        let s1 = self.nfa.add_state();
                        let s2 = self.nfa.add_state();

                        self.nfa
                            .add_transition(s1, s2, Transition::Range(range.0, range.1));

                        (s1, s2)
                    };

                    let s1 = self.nfa.add_state();
                    let s2 = self.nfa.add_state();

                    self.nfa.add_e_transition(s1, conn1.0);
                    self.nfa.add_e_transition(s1, conn2.0);
                    self.nfa.add_e_transition(conn1.1, s2);
                    self.nfa.add_e_transition(conn2.1, s2);

                    conn1 = (s1, s2);
                }

                conn1
            }
            Token::Char(c) => {
                self.advance();

                let s1 = self.nfa.add_state();
                let s2 = self.nfa.add_state();
                self.nfa
                    .add_transition(s1, s2, Transition::Range(c as u8, c as u8));
                (s1, s2)
            }
            Token::Dot => {
                self.advance();

                let s1 = self.nfa.add_state();
                let s2 = self.nfa.add_state();
                self.nfa
                    .add_transition(s1, s2, Transition::Range(0, 127));
                (s1, s2)
            }
            _ => panic!("Invalid expression: {:?}", self.peek()),
        }
    }

    fn character_class(&mut self) -> (u8, u8) {
        match self.advance() {
            Token::Char(c1) => {
                if self.matches(Token::Hyphen) {
                    match self.advance() {
                        Token::Char(c2) => (c1 as u8, c2 as u8),
                        Token::Dot => ('.' as u8, '.' as u8),
                        Token::QuestionMark => ('?' as u8, '?' as u8),
                        Token::Plus => ('+' as u8, '+' as u8),
                        Token::Star => ('*' as u8, '*' as u8),
                        Token::Union => ('|' as u8, '|' as u8),
                        t => panic!("Not implemented: {t:?}"),
                    }
                } else {
                    (c1 as u8, c1 as u8)
                }
            }
            Token::Dot => ('.' as u8, '.' as u8),
            Token::QuestionMark => ('?' as u8, '?' as u8),
            Token::Plus => ('+' as u8, '+' as u8),
            Token::Star => ('*' as u8, '*' as u8),
            Token::Union => ('|' as u8, '|' as u8),
            t => panic!("Not implemented"),
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
