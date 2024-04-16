use clap::{Arg, Parser as ClapParser};

use scanner::Scanner;
use parser::Parser;

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

#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    input: String,

    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    let input: Vec<char> = args.input.chars().collect();

    let mut scanner = Scanner::new(&input);
    let parser = Parser::new(scanner.scan_tokens());

    let nfa = parser.parse();

    if args.debug {
        render_graph("stage1.png", &nfa.to_dot().unwrap());
    }

    let dfa = nfa.reduce_to_dfa();

    if args.debug {
        render_graph("stage2.png", &dfa.to_dot().unwrap());
    }

    let minimized_dfa = dfa.minimize();

    if args.debug {
        render_graph("stage3.png", &minimized_dfa.to_dot().unwrap());
    }

    println!("{}", minimized_dfa.compile().unwrap());
}
