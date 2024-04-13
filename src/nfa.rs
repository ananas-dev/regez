use rustc_hash::{FxHashMap, FxHashSet};

use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
    Direction, Graph,
};

use crate::bitset::BitSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    Char(char),
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub alphabet: FxHashSet<char>,
}

impl Nfa {
    pub fn new() -> Nfa {
        Nfa {
            graph: DiGraph::new(),
            start: 0.into(),
            alphabet: FxHashSet::default(),
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

    pub fn reduce_to_dfa(&self) -> Nfa {
        let node_indices: Vec<NodeIndex> = self.graph.node_indices().collect();
        let e_closure = self.e_closure(&node_indices);
        let mut dfa = Nfa::new();
        let mut node_map: FxHashMap<BitSet<NodeIndex>, NodeIndex> = FxHashMap::default();
        let q0 = e_closure.get(&self.start).unwrap();
        let q0_index = dfa.add_state();
        node_map.insert(q0.clone(), q0_index);
        dfa.set_start(q0_index);

        if self.graph[self.start] == State::Accepting {
            dfa.make_accepting(q0_index)
        }

        let mut work_list = Vec::from([q0.clone()]);

        while let Some(q) = work_list.pop() {
            for &c in self.alphabet.iter() {
                let mut t = BitSet::empty(&node_indices);

                for el in q.iter() {
                    for edge in self.graph.edges_directed(el, Direction::Outgoing) {
                        if *edge.weight() == Transition::Char(c) {
                            t.union(e_closure.get(&edge.target()).unwrap());
                        }
                    }
                }

                if !node_map.contains_key(&t) && t.len() != 0 {
                    let node_idx = dfa.add_state();
                    node_map.insert(t.clone(), node_idx);

                    if t.iter().any(|i| self.graph[i] == State::Accepting) {
                        dfa.make_accepting(node_idx);
                    }

                    work_list.push(t.clone());
                }

                if let Some(q_index) = node_map.get(&q) {
                    if let Some(t_index) = node_map.get(&t) {
                        dfa.add_transition(*q_index, *t_index, Transition::Char(c));
                    }
                }
            }
        }

        dfa
    }

    pub fn e_closure<'a>(
        &'a self,
        node_indices: &'a [NodeIndex],
    ) -> FxHashMap<NodeIndex, BitSet<NodeIndex>> {
        let mut res: FxHashMap<NodeIndex, BitSet<NodeIndex>> = FxHashMap::default();

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
