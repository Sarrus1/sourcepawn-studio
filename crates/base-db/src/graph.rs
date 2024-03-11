use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use fxhash::{FxHashMap, FxHashSet};
use vfs::FileId;

use crate::{FileExtension, SourceDatabase};

#[derive(Debug, Clone)]
pub struct Node {
    pub file_id: FileId,
    pub extension: FileExtension,
}

impl PartialEq<Node> for Node {
    fn eq(&self, other: &Node) -> bool {
        self.file_id == other.file_id
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.file_id.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    pub source: Node,
    pub target: Node,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Graph {
    pub edges: FxHashSet<Edge>,
    pub nodes: FxHashSet<Node>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SubGraph {
    pub root: Node,
    pub nodes: FxHashSet<Node>,
    pub edges: FxHashSet<Edge>,
}

impl SubGraph {
    pub fn file_ids(&self) -> FxHashSet<FileId> {
        self.nodes.iter().map(|node| node.file_id).collect()
    }

    /// Returns true if the subgraph contains the given file_id.
    /// **Note** This is O(n).
    pub fn contains_file(&self, file_id: FileId) -> bool {
        self.nodes.iter().any(|node| node.file_id == file_id)
    }
}

impl Graph {
    /// Get the root of the [subgraph](SubGraph) from a given [file_id](FileId).
    ///
    /// - If the [file_id](FileId) is not in the graph, return [None].
    /// - If the [file_id](FileId) or one of its parent has more than one parent, return [None].
    /// - If the [file_id](FileId) or one of its parent is an include file, return the [file_id](FileId) of the include file.
    pub fn projet_subgraph_query(
        db: &dyn SourceDatabase,
        file_id: FileId,
    ) -> Option<Arc<SubGraph>> {
        let graph = db.graph();
        let subgraphs = graph.find_subgraphs();

        let dummy_node = Node {
            file_id,
            extension: FileExtension::Sp, // We don't care about the extension here. The hash is based on the file_id.
        };

        subgraphs
            .iter()
            .filter(|subgraph| subgraph.root.extension == FileExtension::Sp)
            .find(|subgraph| subgraph.nodes.contains(&dummy_node))
            .cloned()
            .or_else(|| {
                subgraphs
                    .iter()
                    .filter(|subgraph| subgraph.root.extension == FileExtension::Inc)
                    .find(|subgraph| subgraph.nodes.contains(&dummy_node))
                    .cloned()
            })
            .map(Arc::new)
    }

    pub fn graph_query(db: &dyn SourceDatabase) -> Arc<Self> {
        let documents = db.known_files();
        let mut graph = Self::default();

        for (file_id, extension) in documents.iter() {
            let source = Node {
                file_id: *file_id,
                extension: *extension,
            };
            graph.nodes.insert(source.clone());
            for include in db.file_includes(source.file_id).0.iter() {
                let target = Node {
                    file_id: include.file_id(),
                    extension: include.extension(),
                };
                graph.edges.insert(Edge {
                    source: source.clone(),
                    target: target.clone(),
                });
                graph.nodes.insert(target);
            }
        }

        graph.into()
    }
}

impl Graph {
    pub fn add_file_id(
        &mut self,
        source_id: FileId,
        source_extension: FileExtension,
        target_id: FileId,
        target_extension: FileExtension,
    ) {
        let source = Node {
            file_id: source_id,
            extension: source_extension,
        };
        let target = Node {
            file_id: target_id,
            extension: target_extension,
        };
        self.edges.insert(Edge {
            source: source.clone(),
            target: target.clone(),
        });
        self.nodes.insert(source);
        self.nodes.insert(target);
    }

    fn get_adjacent_targets(&self) -> FxHashMap<Node, FxHashSet<Node>> {
        let mut adj_targets: FxHashMap<Node, FxHashSet<Node>> = FxHashMap::default();
        for edge in self.edges.iter() {
            adj_targets
                .entry(edge.source.clone())
                .or_default()
                .insert(edge.target.clone());
        }

        adj_targets
    }

    pub fn find_roots(&self) -> Vec<Node> {
        let mut adj_map: FxHashMap<Node, (u32, u32)> = FxHashMap::default();
        for edge in self.edges.iter() {
            adj_map
                .entry(edge.source.clone())
                .or_insert_with(|| (0, 0))
                .1 += 1;
            adj_map
                .entry(edge.target.clone())
                .or_insert_with(|| (0, 0))
                .0 += 1;
        }
        for node in self.nodes.iter() {
            adj_map.entry(node.clone()).or_insert_with(|| (0, 0));
        }
        adj_map
            .iter()
            .filter_map(|(node, (nb_source, nb_target))| {
                if *nb_target != 0 || *nb_source == 0 {
                    Some(node.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<Node>>()
    }

    pub fn get_subgraph_ids_from_root(&self, root_id: FileId) -> FxHashSet<FileId> {
        let adj_targets = self.get_adjacent_targets();
        let mut visited = FxHashSet::default();
        let mut nodes = FxHashSet::default();
        let mut edges = FxHashSet::default();
        let root = Node {
            file_id: root_id,
            extension: FileExtension::Sp,
        };
        dfs(&root, &adj_targets, &mut visited, &mut nodes, &mut edges);
        visited.insert(root.clone());

        nodes.iter().map(|node| node.file_id).collect()
    }

    pub fn find_subgraphs(&self) -> Vec<SubGraph> {
        let adj_targets = self.get_adjacent_targets();
        let mut subgraphs = vec![];
        for root in self.find_roots() {
            let mut visited = FxHashSet::default();
            let mut nodes = FxHashSet::default();
            let mut edges = FxHashSet::default();
            dfs(&root, &adj_targets, &mut visited, &mut nodes, &mut edges);
            visited.insert(root.clone());
            subgraphs.push(SubGraph {
                root: root.clone(),
                nodes,
                edges,
            });
        }

        subgraphs
    }

    pub fn subgraphs_with_roots(&self) -> FxHashMap<FileId, SubGraph> {
        let subgraphs = self.find_subgraphs();
        subgraphs
            .into_iter()
            .map(|subgraph| (subgraph.root.file_id, subgraph))
            .collect()
    }
}

impl Graph {
    pub fn to_graphviz<F>(&self, file_id_to_name: F) -> Option<String>
    where
        F: Fn(FileId) -> Option<String>,
    {
        let subgraphs = self.find_subgraphs();
        let mut out = vec!["digraph G {".to_string()];
        for (i, sub_graph) in subgraphs
            .iter()
            .filter(|subgraph| subgraph.root.extension == FileExtension::Sp)
            .enumerate()
        {
            out.push(format!(
                r#"  subgraph cluster_{} {{
    style=filled;
    color={};
    node [style=filled,color=white];
    label = "Project n°{}";"#,
                i,
                COLORS[i % COLORS.len()],
                i
            ));
            for edge in sub_graph.edges.iter() {
                out.push(format!(
                    "\"{}\" -> \"{}\";",
                    file_id_to_name(edge.source.file_id)?,
                    file_id_to_name(edge.target.file_id)?
                ));
            }
            out.push("}".to_string());
        }
        for sub_graph in subgraphs
            .iter()
            .filter(|subgraph| subgraph.root.extension == FileExtension::Sp)
        {
            if sub_graph.root.extension == FileExtension::Inc {
                continue;
            }
            out.push(format!(
                "\"{}\" [shape=Mdiamond];",
                file_id_to_name(sub_graph.root.file_id)?
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
    nodes: &mut FxHashSet<Node>,
    edges: &mut FxHashSet<Edge>,
) {
    visited.insert(node.clone());
    nodes.insert(node.clone());

    if let Some(neighbors) = adj_map.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                edges.insert(Edge {
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
