use anyhow::bail;
use fxhash::{FxHashMap, FxHashSet};
use lazy_static::lazy_static;
use lsp_types::Url;
use regex::Regex;
use sourcepawn_lexer::{PreprocDir, SourcepawnLexer, TokenKind};
use syntax::uri_to_file_name;

use crate::{document::Document, Store};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Node {
    pub uri: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    pub source: Node,
    pub target: Node,
}

#[derive(Debug, Default, Clone)]
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

impl Store {
    /// Get all the files that are included in the given document.
    fn get_include_uris_from_document(&self, document: &Document) -> Vec<Url> {
        let mut uris = vec![];
        let lexer = SourcepawnLexer::new(&document.text);
        for symbol in lexer {
            if symbol.token_kind != TokenKind::PreprocDir(PreprocDir::MInclude) {
                continue;
            }
            let text = symbol.text();
            lazy_static! {
                static ref RE1: Regex = Regex::new(r"<([^>]+)>").unwrap();
                static ref RE2: Regex = Regex::new("\"([^>]+)\"").unwrap();
            }
            let mut uri = None;
            if let Some(caps) = RE1.captures(&text) {
                if let Some(path) = caps.get(1) {
                    uri = self.resolve_import(&mut path.as_str().to_string(), &document.uri, false);
                }
            } else if let Some(caps) = RE2.captures(&text) {
                if let Some(path) = caps.get(1) {
                    uri = self.resolve_import(&mut path.as_str().to_string(), &document.uri, true);
                }
            }
            if let Some(uri) = uri {
                uris.push(uri);
            }
        }

        uris
    }

    pub fn load_projects_graph(&mut self) -> Graph {
        let mut graph = Graph::default();

        for document in self.documents.values() {
            for uri in self.get_include_uris_from_document(document) {
                let source = Node {
                    uri: document.uri.as_ref().clone(),
                };
                let target = Node { uri };
                graph.edges.insert(Edge {
                    source: source.clone(),
                    target: target.clone(),
                });
                graph.nodes.insert(source);
                graph.nodes.insert(target);
            }
        }

        graph
    }

    pub fn add_uri_to_projects(&mut self, uri: &Url) -> anyhow::Result<()> {
        let Some(document) = self.documents.get(uri) else {
            bail!("Could not find document to insert from uri {:?}", uri);
        };
        for uri in self.get_include_uris_from_document(document) {
            let source = Node {
                uri: document.uri.as_ref().clone(),
            };
            let target = Node { uri };
            self.projects.edges.insert(Edge {
                source: source.clone(),
                target: target.clone(),
            });
            self.projects.nodes.insert(source);
            self.projects.nodes.insert(target);
        }

        Ok(())
    }

    pub fn remove_uri_from_projects(&mut self, uri: &Url) {
        self.projects
            .edges
            .retain(|edge| &edge.source.uri != uri || &edge.target.uri != uri);
        self.projects.nodes.remove(&Node { uri: uri.clone() });
    }
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

    /// Get the root of the [subgraph](SubGraph) from a given [uri](Url).
    ///
    /// - If the [uri](Url) is not in the graph, return [None].
    /// - If the [uri](Url) or one of its parent has more than one parent, return [None].
    /// - If the [uri](Url) or one of its parent is an include file, return [uri](Url) of the include file.
    pub fn find_root_from_uri(&self, uri: &Url) -> Option<Node> {
        let mut adj_sources: FxHashMap<Node, FxHashSet<Node>> = FxHashMap::default();
        for edge in self.edges.iter() {
            adj_sources
                .entry(edge.target.clone())
                .or_insert_with(FxHashSet::default)
                .insert(edge.source.clone());
        }
        let mut child = &Node { uri: uri.clone() };
        while let Some(parents) = adj_sources.get(&child) {
            if parents.len() == 1 {
                let parent = parents.iter().next().unwrap();

                // If the parent is an include file, we don't want to go further.
                // Include files can be included in multiple files.
                if child.uri.to_file_path().ok()?.ends_with(".inc")
                    && parent.uri.to_file_path().ok()?.ends_with(".sp")
                {
                    return Some(child.clone());
                }
                child = parent;
            } else {
                return None;
            }
        }

        Some(child.clone())
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
