use std::{
    collections::{hash_set, HashMap, HashSet},
    hash::Hash,
};

use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
    Direction,
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

    // pub fn reduce_to_dfa(&self) {
    //     let e_closure = self.e_closure();
    //     let T = Nfa::new();
    //     let q0 = e_closure.get(&self.start).unwrap();
    //     let mut Q = q0.clone();
    //     let mut work_list = Vec::from([q0]);

    //     while let Some(q) = work_list.pop() {
    //         for &c in self.alphabet.iter() {
    //             let mut t = HashSet::new();

    //             for &el in q.iter() {
    //                 // FIXME: Not efficient at all
    //                 for edge in self.graph.edges_directed(el, petgraph::Direction::Outgoing) {
    //                     if *edge.weight() == Transition::Char(c) {
    //                         t.extend(e_closure.get(&el).unwrap());
    //                     }
    //                 }
    //             }

    //             // T.graph.index_twice_mut(i, j)

    //             if !t.is_subset(&Q) {
    //                 Q.extend(t);
    //             }
    //         }

    //         println!("{:?}", Q);
    //     }
    // }

    pub fn e_closure<'a>(&'a self, node_indices: &'a [NodeIndex]) -> HashMap<NodeIndex, BitSet<NodeIndex>> {
        let mut res: HashMap<NodeIndex, BitSet<NodeIndex>> = HashMap::new();

        for &n in node_indices.iter() {
            let mut t = BitSet::empty(node_indices);

            t.insert(n);

            for edge in self.graph.edges_directed(n, Direction::Outgoing) {
                if *edge.weight() == Transition::Empty {
                    t.insert(edge.target());
                }
            }

            res.insert(n, t);
        }
        let mut work_list = BitSet::full(node_indices);

        while let Some(n) = work_list.pop() {
            let t = res.get(&n).unwrap().clone();

            for edge in self.graph.edges_directed(n, Direction::Incoming) {
                if *edge.weight() == Transition::Empty {
                    let m = edge.source();
                    // Backpropagate
                    res.get_mut(&m).unwrap().union(&t);
                    work_list.insert(m);
                }
            }
        }

        res
    }
}
