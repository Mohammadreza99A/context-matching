use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Clone, Debug)]
struct Edge<N> {
    dest: N,
    weight: f64,
}

#[derive(Debug)]
pub struct MarkovGraph<N> {
    adj_list: HashMap<N, Vec<Edge<N>>>,
}

impl<N> MarkovGraph<N>
where
    N: Clone + Eq + PartialEq + std::hash::Hash,
{
    pub fn new() -> Self {
        Self {
            adj_list: HashMap::new(),
        }
    }

    pub fn add_edge(&mut self, src: N, dest: N, weight: f64) {
        self.adj_list
            .entry(src)
            .or_insert(Vec::new())
            .push(Edge { dest, weight });
    }

    pub fn get_dest(&self, src: N, weight: f64) -> Option<N> {
        if let Some(edges) = self.adj_list.get(&src) {
            for edge in edges {
                if edge.weight == weight {
                    return Some(edge.dest.clone());
                }
            }
        }
        None
    }

    pub fn get_all_nodes(&self) -> Vec<N> {
        self.adj_list.keys().cloned().collect()
    }
}

impl<N: fmt::Display> fmt::Display for MarkovGraph<N>
where
    N: Clone + Eq + PartialEq + std::hash::Hash,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (src, edges) in &self.adj_list {
            write!(f, "{} -> ", src)?;
            for edge in edges {
                write!(f, "{}({:.2}), ", edge.dest, edge.weight)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub fn read_graph_from_file<N>(filename: &str) -> MarkovGraph<N>
where
    N: Clone + Eq + std::hash::Hash + std::str::FromStr,
    <N as std::str::FromStr>::Err: std::fmt::Debug,
{
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    let mut graph = MarkovGraph::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();

        let src: N = parts[0].parse().unwrap();
        let dest: N = parts[1].parse().unwrap();
        let weight: f64 = parts[2].parse().unwrap();

        graph.add_edge(src, dest, weight);
    }

    graph
}
