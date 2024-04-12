use petgraph::dot::{Config, Dot};
use scanner::Scanner;
use parser::Parser;

mod nfa;
mod scanner;
mod parser;

fn main() {
    let input = "a(b|c)*";
    let mut scanner = Scanner::new(input.chars());
    let parser = Parser::new(scanner.scan_tokens());

    let nfa = parser.parse();
    println!("{:?}", Dot::with_config(&nfa.graph, &[Config::NodeIndexLabel]));
}
