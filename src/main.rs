use parser::Parser;
use petgraph::{
    dot::{Config, Dot},
    graph::NodeIndex,
};
use scanner::Scanner;

use std::process::{Command, Stdio};
use std::io::Write;

mod bitset;
mod nfa;
mod parser;
mod scanner;

fn render_graph(out_file: &str, content: &str) {
    let mut process = Command::new("dot")
        .args(["-T", "png", "-o", out_file])
        .stdin(Stdio::piped())
        .spawn()
        .expect("failed to launch graphviz");

    let stdin = process.stdin.as_mut().expect("failed to get stdin");

    stdin
        .write_all(content.as_bytes())
        .expect("failed to write to stdin");

    process.wait().expect("failed to wait for end of process");
}

fn main() {
    let input = "f.ck";
    let mut scanner = Scanner::new(input.chars());
    let parser = Parser::new(scanner.scan_tokens());

    let nfa = parser.parse();
    render_graph("stage1.png", &nfa.to_dot().unwrap());

    let dfa = nfa.reduce_to_dfa();
    render_graph("stage2.png", &dfa.to_dot().unwrap());

    println!("{}", dfa.compile().unwrap());
}
