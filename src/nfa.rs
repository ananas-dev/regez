use rustc_hash::{FxHashMap, FxHashSet, FxHasher};
use std::{
    collections::{btree_map::Range, HashSet, VecDeque},
    fmt::{write, Debug, Display, Error, Write},
    os::linux::raw::stat, slice::Windows,
};

use petgraph::{
    graph::{DiGraph, Node, NodeIndex},
    visit::{EdgeRef, IntoEdgesDirected, IntoNodeReferences, NodeRef},
    Direction,
};

use crate::bitset::BitSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Transition {
    Range(u8, u8),
    RangeList(Vec<(u8, u8)>),
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Accepting,
    NotAccepting,
}

// FIXME move elsewhere
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

impl Display for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Transition::Range(a, b) if *a == 0 && *b == 127 => write!(f, ".")?,
            Transition::Range(a, b) if *a == *b => write!(f, "'{}'", *a as char)?,
            Transition::Range(a, b) => write!(f, "[{}-{}]", *a as char, *b as char)?,
            Transition::RangeList(l) => {
                write!(f, "[")?;

                for (a, b) in l.iter() {
                    write!(f, "{}-{}", *a as char, *b as char)?;
                }

                write!(f, "]")?;
            }
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
        let mut e_closure = self.e_closure();
        let mut dfa = Nfa::new();
        let mut node_map: FxHashMap<BitSet<NodeIndex>, NodeIndex> = FxHashMap::default();
        let mut q0 = e_closure.get(&self.start).unwrap().clone();
        q0.remove(self.start.index());

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

        let mut work_list = VecDeque::from([q0.clone()]);

        while let Some(q) = work_list.pop_front() {
            // FIXME inefficient
            let mut mapping: FxHashMap<Transition, BitSet<NodeIndex>> = FxHashMap::default();

            for el in q.iter() {
                for edge in self
                    .graph
                    .edges_directed(NodeIndex::new(el), Direction::Outgoing)
                    .filter(|edge| *edge.weight() != Transition::Empty)
                {
                    if !mapping.contains_key(edge.weight()) {
                        mapping.insert(edge.weight().clone(), BitSet::empty(self.graph.node_count()));
                    }

                    mapping.get_mut(edge.weight()).unwrap().union_inplace(e_closure.get_mut(&edge.target()).unwrap());
                }
            }

            for (weight, t) in mapping {
                if !node_map.contains_key(&t) {
                    let node_idx = dfa.add_state();
                    node_map.insert(t.clone(), node_idx);

                    if t.iter()
                        .any(|i| self.graph[NodeIndex::new(i)] == State::Accepting)
                    {
                        dfa.make_accepting(node_idx);
                    }

                    work_list.push_back(t.clone());

                }

                if let Some(q_index) = node_map.get(&q) {
                    if let Some(t_index) = node_map.get(&t) {
                        dfa.add_transition(*q_index, *t_index, weight);
                    }
                }
            }
        }

        dfa
    }

    fn e_closure(&self) -> FxHashMap<NodeIndex, BitSet<NodeIndex>> {
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
                    work_list.insert(m.index());
                    res.get_mut(&m).unwrap().union_inplace(&t);
                }
            }
        }

        res
    }

    pub fn minimize(&self) -> Nfa {
        let mut res = Nfa::new();

        let mut T = FxHashSet::default();
        let mut P = FxHashSet::default();

        let mut accepting_set: BitSet<NodeIndex> = BitSet::empty(self.graph.node_count());

        for (i, s) in self.graph.node_references() {
            if *s == State::Accepting {
                accepting_set.insert(i.index());
            }
        }

        let non_accepting_set = accepting_set.complement();

        T.insert(accepting_set);
        T.insert(non_accepting_set);

        while T != P {
            P = T;
            T = FxHashSet::default();
            for p in P.iter() {
                let splited = self.split(p);
                if !splited.0.is_empty() {
                    T.insert(splited.0);
                }

                if !splited.1.is_empty() {
                    T.insert(splited.1);
                }
            }
        }

        let mut mapping = FxHashMap::default();

        for new_state in T.iter() {
            let state_id = res.add_state();
            mapping.insert(new_state.clone(), state_id);

            for state in new_state.iter() {
                let state = NodeIndex::new(state);

                if state == self.start {
                    res.set_start(state_id);
                }

                if self.graph[state] == State::Accepting {
                    res.make_accepting(state_id);
                }
            }
        }

        for new_state in T.iter() {
            let mut classes = FxHashMap::default();

            for state in new_state.iter() {
                let state = NodeIndex::new(state);
                for edge in self.graph.edges_directed(state, Direction::Outgoing) {
                    let target_state = T
                        .iter()
                        .find(|s| s.contains(edge.target().index()))
                        .unwrap();

                    match edge.weight() {
                        Transition::Range(a, b) => {
                            let new_edge = *mapping.get(target_state).unwrap();

                            if !classes.contains_key(&new_edge) {
                                classes.insert(new_edge, Vec::new());
                            }

                            let vec = classes.get_mut(&new_edge).unwrap();
                            vec.push((*a, *b));
                        }
                        _ => todo!(),
                    };
                }
            }

            for (node, edge) in classes {
                res.graph.update_edge(
                    *mapping.get(new_state).unwrap(),
                    node,
                    Transition::RangeList(merge_ranges(edge)),
                );
            }
        }

        res
    }

    fn split(&self, s: &BitSet<NodeIndex>) -> (BitSet<NodeIndex>, BitSet<NodeIndex>) {
        let mut alphabet = FxHashSet::default();
        let mut set_1 = s.clone();
        let mut set_2: BitSet<NodeIndex> = BitSet::empty(set_1.universe_len);

        for sample in s.iter().take(1) {
            let node_index = NodeIndex::new(sample);
            for edge in self.graph.edges_directed(node_index, Direction::Outgoing) {
                alphabet.insert(edge.weight().clone());
            }
        }

        for c in alphabet.iter() {
            let mut should_return = false;
            for index in s.iter() {
                let node_index = NodeIndex::new(index);

                if let Some(edge) = self
                    .graph
                    .edges_directed(node_index, Direction::Outgoing)
                    .find(|t| *t.weight() == *c)
                {
                    if !set_1.contains(edge.target().index()) {
                        set_2.insert(index);
                        set_1.remove(index);
                        should_return = true;
                    }
                }

                if self
                    .graph
                    .edges_directed(node_index, Direction::Outgoing)
                    .any(|t| !alphabet.contains(t.weight()))
                {
                    set_2.insert(index);
                    set_1.remove(index);
                    should_return = true;
                }
            }

            if should_return {
                break;
            }
        }

        // dbg!(set_1.iter().collect::<Vec<usize>>());
        // dbg!(set_2.iter().collect::<Vec<usize>>());

        (set_1, set_2)
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
                "\t\"{}\" -> \"{}\" [label = \"{}\"];\n",
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
            Transition::Range(a, b) if *a == 0 && *b == 127 => (),
            Transition::Range(a, b) if *a == *b => write!(&mut res, "if (c == '{}') ", *a as char)?,
            Transition::Range(a, b) => write!(
                &mut res,
                "if (c >= '{}' && c <= '{}') ",
                *a as char, *b as char,
            )?,
            Transition::RangeList(l) => {
                write!(&mut res, "if (")?;

                for (index, &(a, b)) in l.iter().enumerate() {
                    if a == b {
                        write!(&mut res, "(c == '{}')", a as char)?;
                    } else if !(a == 0 && b == 127) {
                        write!(&mut res, "(c >= '{}' && c <= '{}')", a as char, b as char)?;
                    }

                    if index < l.len() - 1 {
                        write!(&mut res, " || ")?;
                    }
                }

                write!(&mut res, ") ")?;
            }
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

                write!(
                    &mut s,
                    "\t{}goto s{};\n",
                    self.c_condition(&transition)?,
                    neighbor.index()
                )?;
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
