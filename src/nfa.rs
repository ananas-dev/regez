use std::{collections::HashSet, hash::Hash};

use petgraph::graph::{DiGraph, NodeIndex};

#[derive(Debug, Clone, Copy)]
pub enum Transition {
    Char(char),
    Empty,
}

#[derive(Debug, Clone, Copy)]
pub enum State {
    Accepting,
    NotAccepting
}

// pub fn convert_to_dfa(nfa: &mut Nfa, start: NodeIndex) {
//     let q0 = emclosure(HashSet::from([start]));
//     let q = q0;
//     let mut work_list = Vec::from([q0]);

//     while !work_list.is_empty() {

//     }
// }

pub fn emclosure(a: HashSet<NodeIndex>) -> HashSet<NodeIndex> {
    todo!()
}

#[derive(Clone)]
pub struct Nfa {
    pub graph: DiGraph<State, Transition>,
    pub start: NodeIndex,
    pub alphabet: HashSet<char>,
}

impl Nfa {
    pub fn new() -> Nfa {
        Nfa {
            graph: DiGraph::new(),
            start: 0.into(),
            alphabet: HashSet::new(),
        }
    }

    pub fn set_start(&mut self, start: NodeIndex) {
        self.start = start
    }

    pub fn add_transition(&mut self, s1: NodeIndex, s2: NodeIndex, transition: Transition) {
        if let Transition::Char(c) = transition {
            self.alphabet.insert(c);
        }

        self.graph.add_edge(s1, s2, transition);
    }
    
    pub fn add_e_transition(&mut self, s1: NodeIndex, s2: NodeIndex) {
        self.graph.add_edge(s1, s2, Transition::Empty);
    }

    pub fn add_state(&mut self) -> NodeIndex {
        self.graph.add_node(State::NotAccepting)
    }

    pub fn make_accepting(&mut self, state: NodeIndex) {
        self.graph[state] = State::Accepting;
    }

    pub fn reduce_to_dfa(&mut self) {
        let T = Nfa::new();
        let q0 = emclosure(HashSet::from([self.start]));
        let Q = q0.clone();
        let mut work_list = Vec::from([q0]);

        while !work_list.is_empty() {
            let q = work_list.pop();
            
            for c in self.alphabet.iter() {
                let t = emclosure(todo!());
                todo!();
                if !t.is_subset(&Q) {
                    Q.extend(t);
                    
                }
            }
        }

    }

}