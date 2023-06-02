use std::{fmt, fs};

use clap::{arg, command, Parser};
use petgraph::{
    dot::{Config, Dot},
    Graph,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    nodes_path: String,

    #[arg(short, long)]
    edges_path: String,
}

fn main() {
    let args = Args::parse();

    let nodes_file =
        fs::read_to_string(args.nodes_path).expect("Should have been able to read the nodes file");
    let edges_file =
        fs::read_to_string(args.edges_path).expect("Should have been able to read the edges file");
    let nodes = process_node_file(nodes_file);
    let edges = process_edge_file(edges_file);
    // Create an undirected graph with `i32` nodes and edges with `()` associated data.
    let mut g = Graph::<Node, Edge>::new();
    for node in nodes {
        g.add_node(node);
    }
    for edge in edges {
        let n1 = g.node_indices().find(|i| g[*i].id == edge.source);
        let n2 = g.node_indices().find(|i| g[*i].id == edge.target);
        if n1.is_some() && n2.is_some() {
            g.add_edge(n1.unwrap(), n2.unwrap(), edge);
        }
    }
    println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));
}

fn process_node_file(nodes_file: String) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::new();
    for line in nodes_file.lines() {
        let id = line
            .split('-')
            .next()
            .unwrap()
            .to_string()
            .split(' ')
            .nth(1)
            .unwrap()
            .to_string();
        let label = line.split('-').nth(1).unwrap().to_string();
        nodes.push(Node { id, label });
    }
    nodes
}

fn process_edge_file(edge_file: String) -> Vec<Edge> {
    let mut edges: Vec<Edge> = Vec::new();
    for line in edge_file.lines() {
        let source = line
            .split(" -> ")
            .next()
            .unwrap()
            .to_string()
            .split(' ')
            .nth(1)
            .unwrap()
            .to_string();
        let target = line.split(" -> ").nth(1).unwrap().to_string();
        edges.push(Edge {
            source,
            target,
            label: None,
        });
    }
    edges
}

struct Node {
    id: String,
    label: String,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.label.fmt(f)
    }
}

#[derive(Debug)]
struct Edge {
    source: String,
    target: String,
    label: Option<String>,
}
