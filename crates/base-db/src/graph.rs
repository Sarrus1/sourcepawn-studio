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
            .into_iter()
            .find(|subgraph| subgraph.nodes.contains(&dummy_node))
            .map(Arc::new)

        // let mut adj_sources: FxHashMap<Node, FxHashSet<Node>> = FxHashMap::default();
        // for edge in graph.edges.iter() {
        //     adj_sources
        //         .entry(edge.target.clone())
        //         .or_default()
        //         .insert(edge.source.clone());
        // }
        // let mut child = &Node {
        //     file_id,
        //     extension: FileExtension::Sp,
        // };

        // // Keep track of the nodes we visited to avoid infinite loops.
        // let mut visited: FxHashSet<&Node> = FxHashSet::default();
        // while let Some(parents) = adj_sources.get(child) {
        //     if visited.contains(child) {
        //         return Some(child.file_id);
        //     }
        //     visited.insert(child);
        //     if parents.len() == 1 {
        //         let parent = parents.iter().next().unwrap();

        //         // If the parent is an include file, we don't want to go further.
        //         // Include files can be included in multiple files.
        //         if child.extension == FileExtension::Inc && parent.extension == FileExtension::Sp {
        //             return Some(child.file_id);
        //         }
        //         child = parent;
        //     } else if child.extension == FileExtension::Inc {
        //         return Some(child.file_id);
        //     } else {
        //         return None;
        //     }
        // }

        // Some(child.file_id)
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

/*
impl Store {
    /// Get all the files that are included in the given document.
    fn get_include_ids_from_document(&self, document: &Document) -> Vec<(FileId, FileExtension)> {
        let mut file_ids = vec![];
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
            let mut file_id = None;
            if let Some(caps) = RE1.captures(&text) {
                if let Some(path) = caps.get(1) {
                    file_id =
                        self.resolve_import(&mut path.as_str().to_string(), &document.uri, false);
                }
            } else if let Some(caps) = RE2.captures(&text) {
                if let Some(path) = caps.get(1) {
                    file_id =
                        self.resolve_import(&mut path.as_str().to_string(), &document.uri, true);
                }
            }
            if let Some(file_id) = file_id {
                file_ids.push((
                    file_id,
                    uri_to_file_extension(self.vfs.lookup(file_id)).unwrap_or_default(),
                ));
            }
        }

        file_ids
    }

    pub fn load_projects_graph(&mut self) -> Graph {
        let mut graph = Graph::default();

        for document in self.documents.values() {
            let source = Node {
                file_id: document.file_id,
                extension: document.extension(),
            };
            graph.nodes.insert(source.clone());
            for (file_id, extension) in self.get_include_ids_from_document(document) {
                let target = Node { file_id, extension };
                graph.edges.insert(Edge {
                    source: source.clone(),
                    target: target.clone(),
                });
                graph.nodes.insert(target);
            }
        }

        graph
    }

    pub fn add_file_to_projects(&mut self, file_id: &FileId) -> anyhow::Result<()> {
        let Some(document) = self.documents.get(file_id) else {
            bail!(
                "Could not find document to insert from uri {:?}",
                self.vfs.lookup(*file_id)
            );
        };
        for (file_id, extension) in self.get_include_ids_from_document(document) {
            self.projects
                .add_file_id(document.file_id, document.extension(), file_id, extension)
        }

        Ok(())
    }

    pub fn remove_file_from_projects(&mut self, file_id: &FileId) {
        self.projects
            .edges
            .retain(|edge| &edge.source.file_id != file_id || &edge.target.file_id != file_id);
        self.projects.nodes.remove(&Node {
            file_id: *file_id,
            extension: FileExtension::Sp, // We don't care about the extension here.
        });
    }
}
*/

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
}

impl Graph {
    pub fn to_graphviz<F>(&self, file_id_to_name: F) -> Option<String>
    where
        F: Fn(FileId) -> Option<String>,
    {
        let subgraphs = self.find_subgraphs();
        let mut out = vec!["digraph G {".to_string()];
        for (i, sub_graph) in subgraphs.iter().enumerate() {
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
        for sub_graph in subgraphs.iter() {
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
