use petgraph::{graph::NodeIndex, dot::{Config, Dot}};
use scanner::Scanner;
use parser::Parser;

mod nfa;
mod scanner;
mod parser;
mod bitset;

fn main() {
    let input = "a(b|c)*";
    let mut scanner = Scanner::new(input.chars());
    let parser = Parser::new(scanner.scan_tokens());

    let nfa = parser.parse();
    // println!("{:?}", Dot::with_config(&nfa.graph, &[Config::NodeIndexLabel]));

    // let node_indices: Vec<NodeIndex>= nfa.graph.node_indices().collect();
    let test = nfa.emclosure();
    println!("{:#?}", test);
}
