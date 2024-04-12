use std::{
    collections::{hash_set, HashMap, HashSet},
    hash::Hash,
};

use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};

use crate::bitset::BitSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    Char(char),
    Empty,
}

#[derive(Debug, Clone, Copy)]
pub enum State {
    Accepting,
    NotAccepting,
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

    pub fn emclosure(&self) -> HashMap<NodeIndex, HashSet<NodeIndex>> {
        let mut res: HashMap<NodeIndex, HashSet<NodeIndex>> = HashMap::new();
        let node_indicies: Vec<NodeIndex> = self.graph.node_indices().collect();

        for &n in node_indicies.iter() {
            res.insert(n, HashSet::from([n]));
        }
        let mut work_list = BitSet::full(&node_indicies);

        while let Some(n) = work_list.pop() {
            let mut t = res.get(&n).cloned().unwrap();

            for &p in node_indicies.iter() {
                if let Some(edge) = self.graph.find_edge(n, p) {
                    if self.graph[edge] == Transition::Empty {
                        t.insert(p);
                    }
                }
            }

            if t != *res.get(&n).unwrap() {
                res.insert(n, t.clone());

                for edge in self.graph.edges_directed(n, petgraph::Direction::Incoming) {
                    if *edge.weight() == Transition::Empty {
                        let m = edge.source();
                        // Backpropagate
                        res.insert(m, res.get(&m).unwrap().union(&t).cloned().collect());
                        work_list.insert(m);
                    }
                }
            }
        }

        res
    }
}
