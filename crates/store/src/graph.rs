use fxhash::{FxHashMap, FxHashSet};
use lsp_types::Url;
use syntax::uri_to_file_name;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Node {
    pub uri: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    pub source: Node,
    pub target: Node,
}

#[derive(Debug, Default)]
pub struct Graph {
    pub edges: FxHashSet<Edge>,
    pub missing: Vec<Url>,
    pub nodes: FxHashSet<Node>,
}

#[derive(Debug)]
pub struct SubGraph {
    pub root: Node,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl Graph {
    fn get_adjacent_targets(&self) -> FxHashMap<Node, FxHashSet<Node>> {
        let mut adj_targets: FxHashMap<Node, FxHashSet<Node>> = FxHashMap::default();
        for edge in self.edges.iter() {
            adj_targets
                .entry(edge.source.clone())
                .or_insert_with(FxHashSet::default)
                .insert(edge.target.clone());
        }

        adj_targets
    }

    pub fn find_roots(&self) -> Vec<&Node> {
        let mut adj_sources: FxHashMap<Node, FxHashSet<Node>> = FxHashMap::default();
        for edge in self.edges.iter() {
            adj_sources
                .entry(edge.target.clone())
                .or_insert_with(FxHashSet::default)
                .insert(edge.source.clone());
        }
        self.nodes
            .iter()
            .filter(|node| !adj_sources.contains_key(node))
            .collect::<Vec<&Node>>()
    }

    pub fn find_subgraphs(&self) -> Vec<SubGraph> {
        let adj_targets = self.get_adjacent_targets();
        let mut subgraphs = vec![];
        for root in self.find_roots() {
            let mut visited = FxHashSet::default();
            let mut nodes = vec![];
            let mut edges = vec![];
            dfs(root, &adj_targets, &mut visited, &mut nodes, &mut edges);
            visited.insert(root.clone());
            subgraphs.push(SubGraph {
                root: root.clone(),
                nodes,
                edges,
            });
        }

        subgraphs
    }

    pub fn represent_graphs(&self) -> Option<String> {
        let mut out = vec!["digraph G {".to_string()];
        let subgraphs = self.find_subgraphs();
        for (i, sub_graph) in subgraphs.iter().enumerate() {
            if uri_to_file_name(&sub_graph.root.uri)?.ends_with(".inc") {
                continue;
            }
            out.push(format!(
                r#"  subgraph cluster_{} {{
    style=filled;
    color={};
    node [style=filled,color=white];
    label = "process {}";"#,
                i,
                COLORS[i % COLORS.len()],
                i
            ));
            for edge in sub_graph.edges.iter() {
                out.push(format!(
                    "\"{}\" -> \"{}\";",
                    uri_to_file_name(&edge.source.uri)?,
                    uri_to_file_name(&edge.target.uri)?
                ));
            }
            out.push("}".to_string());
        }
        for sub_graph in subgraphs.iter() {
            if uri_to_file_name(&sub_graph.root.uri)
                .unwrap()
                .ends_with(".inc")
            {
                continue;
            }
            out.push(format!(
                "\"{}\" [shape=Mdiamond];",
                uri_to_file_name(&sub_graph.root.uri)?
            ))
        }
        out.push("}".to_string());

        Some(out.join("\n"))
    }
}

fn dfs(
    node: &Node,
    adj_map: &FxHashMap<Node, FxHashSet<Node>>,
    visited: &mut FxHashSet<Node>,
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Edge>,
) {
    visited.insert(node.clone());
    nodes.push(node.clone());

    if let Some(neighbors) = adj_map.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                edges.push(Edge {
                    source: node.clone(),
                    target: neighbor.clone(),
                });
                dfs(neighbor, adj_map, visited, nodes, edges);
            }
        }
    }
}

static COLORS: [&str; 88] = [
    "aliceblue",
    "antiquewhite",
    "aqua",
    "aquamarine",
    "azure",
    "beige",
    "bisque",
    "black",
    "blanchedalmond",
    "blue",
    "blueviolet",
    "brown",
    "burlywood",
    "cadetblue",
    "chartreuse",
    "chocolate",
    "coral",
    "cornflowerblue",
    "cornsilk",
    "crimson",
    "cyan",
    "darkblue",
    "darkcyan",
    "darkgoldenrod",
    "darkgray",
    "darkgreen",
    "darkkhaki",
    "darkmagenta",
    "darkolivegreen",
    "darkorange",
    "darkorchid",
    "darkred",
    "darksalmon",
    "darkseagreen",
    "darkslateblue",
    "darkslategray",
    "darkturquoise",
    "darkviolet",
    "deeppink",
    "deepskyblue",
    "dimgray",
    "dodgerblue",
    "firebrick",
    "floralwhite",
    "forestgreen",
    "fuchsia",
    "gainsboro",
    "ghostwhite",
    "gold",
    "goldenrod",
    "gray",
    "green",
    "greenyellow",
    "honeydew",
    "hotpink",
    "indianred",
    "indigo",
    "ivory",
    "khaki",
    "lavender",
    "lavenderblush",
    "lawngreen",
    "lemonchiffon",
    "lightblue",
    "lightcoral",
    "lightcyan",
    "lightgoldenrodyellow",
    "lightgray",
    "lightgreen",
    "lightpink",
    "lightsalmon",
    "lightseagreen",
    "lightskyblue",
    "lightslategray",
    "lightsteelblue",
    "lightyellow",
    "lime",
    "limegreen",
    "linen",
    "magenta",
    "maroon",
    "mediumaquamarine",
    "mediumblue",
    "mediumorchid",
    "mediumpurple",
    "mediumseagreen",
    "mediumslateblue",
    "mediumspringgreen",
];
