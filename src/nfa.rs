use rustc_hash::{FxHashMap, FxHashSet};
use std::{
    collections::{HashSet, VecDeque},
    fmt::{write, Debug, Display, Error, Write},
};

use petgraph::{
    graph::{DiGraph, Node, NodeIndex},
    visit::{EdgeRef, IntoNodeReferences, NodeRef},
    Direction,
};

use crate::bitset::BitSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharacterClass {
    Range { from: char, to: char },
    Char(char),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transition {
    Any,
    Char(char),
    ClassList {
        content: Vec<CharacterClass>,
        inclusive: bool,
    },
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Accepting,
    NotAccepting,
}

impl Display for CharacterClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharacterClass::Range { from, to } => write!(f, "{from}-{to}")?,
            CharacterClass::Char(c) => f.write_char(*c)?,
        }

        Ok(())
    }
}

impl Display for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Transition::ClassList { content, inclusive } => {
                f.write_char('[')?;
                if !*inclusive {
                    f.write_char('^')?;
                }

                for class in content {
                    std::fmt::Display::fmt(class, f)?;
                }

                f.write_char(']')?;
            }
            Transition::Char(c) => write!(f, "'{}'", *c)?,
            Transition::Any => f.write_char('.')?,
            Transition::Empty => f.write_char('ε')?,
        }

        Ok(())
    }
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
}

impl Nfa {
    pub fn new() -> Nfa {
        Nfa {
            graph: DiGraph::new(),
            start: 0.into(),
        }
    }

    pub fn set_start(&mut self, start: NodeIndex) {
        self.start = start
    }

    pub fn add_transition(&mut self, s1: NodeIndex, s2: NodeIndex, transition: Transition) {
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

    pub fn clone_subgraph(&mut self, s1: NodeIndex, s2: NodeIndex) -> (NodeIndex, NodeIndex) {
        let mut stack = VecDeque::new();
        let mut mapping = FxHashMap::default();
        stack.push_back(s1);
        mapping.insert(s1, self.add_state());

        while let Some(node) = stack.pop_front() {
            if node == s2 {
                continue;
            }

            let neighbors: Vec<NodeIndex> = self.graph.neighbors(node).collect();
            for neighbor in neighbors {
                let transition = self
                    .graph
                    .edges_connecting(node, neighbor)
                    .nth(0)
                    .unwrap()
                    .weight()
                    .clone();
                if let Some(&neighbor_clone) = mapping.get(&neighbor) {
                    let node_clone = *mapping.get(&node).unwrap();
                    self.add_transition(node_clone, neighbor_clone, transition);
                } else {
                    let neighbor_clone = self.add_state();
                    mapping.insert(neighbor, neighbor_clone);
                    let node_clone = *mapping.get(&node).unwrap();
                    self.add_transition(node_clone, neighbor_clone, transition);
                    stack.push_back(neighbor);
                }
            }
        }

        (*mapping.get(&s1).unwrap(), *mapping.get(&s2).unwrap())
    }

    pub fn reduce_to_dfa(&self) -> Nfa {
        let e_closure = self.e_closure();
        let mut dfa = Nfa::new();
        let mut node_map: FxHashMap<BitSet<NodeIndex>, NodeIndex> = FxHashMap::default();
        let q0 = e_closure.get(&self.start).unwrap();

        {
            let q0_index = dfa.add_state();

            node_map.insert(q0.clone(), q0_index);
            dfa.set_start(q0_index);

            if q0
                .iter()
                .any(|i| self.graph[NodeIndex::new(i)] == State::Accepting)
            {
                dfa.make_accepting(q0_index);
            }
        }

        let mut work_list = Vec::from([q0.clone()]);

        while let Some(q) = work_list.pop() {
            // FIXME inefficient
            for el in q.iter() {
                let mut t = BitSet::empty(self.graph.node_count());

                for edge in self
                    .graph
                    .edges_directed(NodeIndex::new(el), Direction::Outgoing)
                {
                    if *edge.weight() != Transition::Empty {
                        t.union(e_closure.get(&edge.target()).unwrap());
                    }

                    if !t.is_empty() && !node_map.contains_key(&t) {
                        let node_idx = dfa.add_state();
                        node_map.insert(t.clone(), node_idx);

                        if t.iter()
                            .any(|i| self.graph[NodeIndex::new(i)] == State::Accepting)
                        {
                            dfa.make_accepting(node_idx);
                        }

                        work_list.push(t.clone());
                    }

                    if let Some(q_index) = node_map.get(&q) {
                        if let Some(t_index) = node_map.get(&t) {
                            dfa.add_transition(*q_index, *t_index, edge.weight().clone());
                        }
                    }
                }
            }
        }

        dfa
    }

    pub fn e_closure(&self) -> FxHashMap<NodeIndex, BitSet<NodeIndex>> {
        let mut res: FxHashMap<NodeIndex, BitSet<NodeIndex>> = FxHashMap::default();

        for n in self.graph.node_indices() {
            let mut t = BitSet::empty(self.graph.node_count());

            t.insert(n.index());

            for edge in self.graph.edges_directed(n, Direction::Outgoing) {
                if *edge.weight() == Transition::Empty {
                    t.insert(edge.target().index());
                }
            }

            res.insert(n, t);
        }
        let mut work_list: BitSet<NodeIndex> = BitSet::full(self.graph.node_count());

        while let Some(n) = work_list.pop() {
            let n = NodeIndex::new(n);
            let t = res.get(&n).unwrap().clone();

            for edge in self.graph.edges_directed(n, Direction::Incoming) {
                if *edge.weight() == Transition::Empty {
                    let m = edge.source();
                    // Backpropagate
                    if work_list.insert(m.index()) {
                        res.get_mut(&m).unwrap().union(&t);
                    }
                }
            }
        }

        res
    }

    pub fn to_dot(&self) -> Result<String, Error> {
        let mut s = String::new();

        s.push_str("digraph {\n");
        s.push_str("\trankdir=LR;\n");
        s.push_str("\tnode [shape = circle];\n");

        for (index, state) in self.graph.node_references() {
            if *state == State::Accepting {
                write!(&mut s, "\t\"{}\" [shape = doublecircle];\n", index.index())?;
            }
        }

        for edge in self.graph.edge_references() {
            write!(
                &mut s,
                "\t\"{}\" -> \"{}\" [label = \"{}\"]\n",
                edge.source().index(),
                edge.target().index(),
                edge.weight(),
            )?;
        }

        s.push_str("}");

        Ok(s)
    }

    fn c_condition(&self, t: &Transition) -> Result<String, Error> {
        let mut res = String::new();
        match t {
            Transition::Any => (),
            Transition::Char(c) => write!(&mut res, "if (c == '{c}') ")?,
            Transition::ClassList { content, inclusive } => {
                write!(&mut res, "if (")?;

                for (index, class) in content.iter().enumerate() {
                    match class {
                        CharacterClass::Range { from, to } => {
                            if *inclusive {
                                write!(&mut res, "c >= '{from}' && c <= '{to}'")?
                            } else {
                                write!(&mut res, "c < '{from}' || c > '{to}'")?
                            }
                        },
                        CharacterClass::Char(c) => {
                            if *inclusive {
                                write!(&mut res, "c == '{c}'")?
                            } else {
                                write!(&mut res, "c != '{c}'")?
                            }
                        },
                    }

                    if index < content.len() - 1 {
                        if *inclusive {
                            write!(&mut res, " || ")?;
                        } else {
                            write!(&mut res, " && ")?;
                        }
                    }
                }

                write!(&mut res, ") ")?
            },
            Transition::Empty => panic!(),
        }

        Ok(res)
    }

    pub fn compile(&self) -> Result<String, Error> {
        let mut res = String::from("#include \"stack.h\"\n\n");
        let mut s = String::new();

        let mut accepting_table = String::from("int accepting[] = {");

        s.push_str("int matches(char *input) {\n");

        s.push_str("\tint state;\n");
        s.push_str("\tchar c;\n");
        s.push_str("\tint cursor = 0;\n");
        s.push_str("\tStack stack = {};\n");
        s.push_str("\tstack_init(&stack);\n");
        s.push_str("start:\n");
        s.push_str("\tpush(&stack, -1);\n");
        write!(&mut s, "\tgoto s{};\n", self.start.index())?;

        for (index, state) in self.graph.node_references() {
            write!(&mut s, "s{}:\n", index.index())?;

            write!(&mut s, "\tstate = {};\n", index.index())?;
            s.push_str("\tif ((c = input[cursor++]) == '\\0') goto end;\n");

            if *state == State::Accepting {
                write!(&mut accepting_table, "{}", "1,")?;
                s.push_str("\tclear(&stack);\n");
            } else {
                write!(&mut accepting_table, "{}", "0,")?; // bit janky
            }

            write!(&mut s, "\tpush(&stack, {});\n", index.index())?;
            for neighbor in self.graph.neighbors_directed(index, Direction::Outgoing) {
                let transition = self
                    .graph
                    .edges_connecting(index, neighbor)
                    .nth(0)
                    .unwrap()
                    .weight();

                write!(&mut s, "\t{}goto s{};\n", self.c_condition(&transition)?, neighbor.index())?;
            }
            s.push_str("\tgoto end;\n");
        }

        s.push_str("end:\n");

        // s.push_str("\twhile (state != -1 && !accepting[state]) {\n");
        // s.push_str("\t\tstate = pop(&stack);\n");
        // // s.push_str("\t\trollback();\n");
        // s.push_str("\t}\n");
        s.push_str("\treturn accepting[state];\n");

        s.push_str("}\n");

        accepting_table.pop();
        accepting_table.push_str("};\n\n");

        res.extend(accepting_table.chars());
        res.extend(s.chars());

        Ok(res)
    }
}
