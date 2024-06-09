use std::ops::Index;

use base_db::Tree;
use fxhash::{FxHashMap, FxHashSet};
use hir_def::FunctionKind;
use la_arena::{Arena, Idx};
use lazy_static::lazy_static;
use preprocessor::{s_range_to_u_range, Offset};
use smol_str::{SmolStr, ToSmolStr};
use syntax::{utils::ts_range_to_lsp_range, TSKind};
use tree_sitter::{Node, QueryCursor};

use crate::SymbolKind;

lazy_static! {
    static ref VARIABLE_QUERY: tree_sitter::Query = tree_sitter::Query::new(
        &tree_sitter_sourcepawn::language(),
        "[(variable_declaration) @variable_declaration (old_variable_declaration) @old_variable_declaration]"
    )
    .expect("Could not build identifier query.");
}

pub type SymbolId = Idx<Symbol>;

pub struct SymbolsBuilder<'a> {
    offsets: &'a FxHashMap<u32, Vec<Offset>>,
    top_level: Vec<SymbolId>,
    arena: Arena<Symbol>,
    deprecated: FxHashSet<usize>,
    tree: &'a Tree,
    source: &'a str,
}

impl<'a> SymbolsBuilder<'a> {
    pub fn new(offsets: &'a FxHashMap<u32, Vec<Offset>>, tree: &'a Tree, source: &'a str) -> Self {
        let mut deprecated = FxHashSet::default();
        // query for all pragmas
        lazy_static! {
            static ref MACRO_QUERY: tree_sitter::Query = tree_sitter::Query::new(
                &tree_sitter_sourcepawn::language(),
                "[(preproc_pragma) @pragma]"
            )
            .expect("Could not build pragma query.");
        }
        let mut cursor = QueryCursor::new();
        let matches = cursor.captures(&MACRO_QUERY, tree.root_node(), source.as_bytes());
        for (match_, _) in matches {
            for c in match_.captures {
                let Ok(pragma) = c.node.utf8_text(source.as_bytes()) else {
                    continue;
                };
                if pragma.starts_with("#pragma deprecated") {
                    deprecated.insert(c.node.range().start_point.row);
                }
            }
        }
        Self {
            offsets,
            top_level: Vec::new(),
            arena: Arena::new(),
            deprecated,
            tree,
            source,
        }
    }

    fn s_range(&self, u_range: &tree_sitter::Range) -> lsp_types::Range {
        s_range_to_u_range(self.offsets, ts_range_to_lsp_range(u_range))
    }

    fn is_deprecated(&self, node: &tree_sitter::Node) -> bool {
        self.deprecated
            .contains(&node.range().start_point.row.saturating_sub(1))
    }

    fn alloc(&mut self, symbol: Symbol) -> SymbolId {
        self.arena.alloc(symbol)
    }

    fn alloc_top(&mut self, symbol: Symbol) -> SymbolId {
        let idx = self.arena.alloc(symbol);
        self.top(idx)
    }

    fn top(&mut self, id: SymbolId) -> SymbolId {
        self.top_level.push(id);
        id
    }

    fn alloc_function(&mut self, node: &Node) -> Option<SymbolId> {
        let kind = match FunctionKind::from_node(node) {
            FunctionKind::Def => SymbolKind::Function,
            FunctionKind::Forward => SymbolKind::Forward,
            FunctionKind::Native => SymbolKind::Native,
        };
        self.alloc_function_(node, &node.child_by_field_name("name")?, kind)
            .map(|id| self.top(id))
    }

    fn alloc_method(&mut self, node: &Node) -> Option<SymbolId> {
        self.alloc_function_(node, &node.child_by_field_name("name")?, SymbolKind::Method)
    }

    fn alloc_property_method(&mut self, node: &Node) -> Option<SymbolId> {
        node.children(&mut node.walk()).find_map(|child| {
            if matches!(
                TSKind::from(&child),
                TSKind::methodmap_property_getter | TSKind::methodmap_property_setter
            ) {
                self.alloc_function_(
                    node,
                    &child.child_by_field_name("name")?,
                    SymbolKind::Method,
                )
            } else {
                None
            }
        })
    }

    fn alloc_function_(
        &mut self,
        node: &Node,
        name_node: &Node,
        kind: SymbolKind,
    ) -> Option<SymbolId> {
        let mut children = Vec::new();
        let mut cursor = QueryCursor::new();
        let matches = cursor.captures(&VARIABLE_QUERY, *node, self.source.as_bytes());
        for (match_, _) in matches {
            for c in match_.captures {
                if let Some(idx) = self.alloc_variable_declaration(&c.node) {
                    children.push(idx);
                }
            }
        }
        let symbol = Symbol {
            name: name_node
                .utf8_text(self.source.as_bytes())
                .ok()?
                .to_smolstr(),
            kind,
            full_range: self.s_range(&node.range()),
            focus_range: self.s_range(&name_node.range()).into(),
            children,
            details: node
                .child_by_field_name("parameters")
                .and_then(|n| n.utf8_text(self.source.as_bytes()).ok())
                .map(ToString::to_string),
            deprecated: self.is_deprecated(node),
        };
        self.alloc(symbol).into()
    }

    fn alloc_methodmap(&mut self, node: &Node) -> Option<SymbolId> {
        let name_node = node.child_by_field_name("name")?;
        let children = node
            .children(&mut node.walk())
            .flat_map(|child| match TSKind::from(&child) {
                TSKind::methodmap_property => self.alloc_property(&child),
                TSKind::methodmap_method => self.alloc_method(&child),
                TSKind::methodmap_method_constructor | TSKind::methodmap_native_constructor => self
                    .alloc_function_(
                        &child,
                        &child.child_by_field_name("name")?,
                        SymbolKind::Constructor,
                    ),
                TSKind::methodmap_method_destructor | TSKind::methodmap_native_destructor => self
                    .alloc_function_(
                        &child,
                        &child.child_by_field_name("name")?,
                        SymbolKind::Destructor,
                    ),
                _ => None,
            })
            .collect();
        let symbol = Symbol {
            name: name_node
                .utf8_text(self.source.as_bytes())
                .ok()?
                .to_smolstr(),
            kind: SymbolKind::Methodmap,
            full_range: self.s_range(&node.range()),
            focus_range: self.s_range(&name_node.range()).into(),
            children,
            details: None,
            deprecated: self.is_deprecated(node),
        };
        self.alloc_top(symbol).into()
    }

    fn alloc_property(&mut self, node: &Node) -> Option<SymbolId> {
        let name_node = node.child_by_field_name("name")?;
        let type_ = node.child_by_field_name("type").and_then(|n| {
            n.utf8_text(self.source.as_bytes())
                .ok()
                .map(ToString::to_string)
        });
        let children = node
            .children(&mut node.walk())
            .flat_map(|child| {
                if matches!(
                    TSKind::from(&child),
                    TSKind::methodmap_property_method | TSKind::methodmap_property_native
                ) {
                    self.alloc_property_method(&child)
                } else {
                    None
                }
            })
            .collect();
        let symbol = Symbol {
            name: name_node
                .utf8_text(self.source.as_bytes())
                .ok()?
                .to_smolstr(),
            kind: SymbolKind::Property,
            full_range: self.s_range(&node.range()),
            focus_range: self.s_range(&name_node.range()).into(),
            children,
            details: type_,
            deprecated: self.is_deprecated(node),
        };
        self.alloc(symbol).into()
    }

    fn alloc_enum(&mut self, node: &Node) -> Option<SymbolId> {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .and_then(|n| n.utf8_text(self.source.as_bytes()).ok())
            .unwrap_or("enum")
            .to_smolstr();
        let children = node
            .child_by_field_name("entries")
            .map(|entries_node| {
                {
                    entries_node
                        .children(&mut entries_node.walk())
                        .filter(|e| TSKind::from(e) == TSKind::enum_entry)
                        .flat_map(|e| {
                            let name_node = e.child_by_field_name("name")?;
                            let symbol = Symbol {
                                name: name_node
                                    .utf8_text(self.source.as_bytes())
                                    .ok()?
                                    .to_smolstr(),
                                kind: SymbolKind::Variant,
                                full_range: self.s_range(&e.range()),
                                focus_range: self.s_range(&name_node.range()).into(),
                                children: vec![],
                                details: None,
                                deprecated: self.is_deprecated(&e),
                            };
                            Some(self.alloc(symbol))
                        })
                }
                .collect()
            })
            .unwrap_or_default();
        let symbol = Symbol {
            name,
            kind: SymbolKind::Property,
            full_range: self.s_range(&node.range()),
            focus_range: name_node.map(|node| self.s_range(&node.range())),
            children,
            details: None,
            deprecated: self.is_deprecated(node),
        };
        self.alloc(symbol).into()
    }

    fn alloc_variable_declaration(&mut self, node: &Node) -> Option<SymbolId> {
        let name_node = node.child_by_field_name("name")?;
        let type_ = node
            .parent()
            .and_then(|n| {
                n.child_by_field_name("type")
                    .and_then(|n| n.utf8_text(self.source.as_bytes()).ok())
            })
            .map(ToString::to_string);
        let symbol = Symbol {
            name: name_node
                .utf8_text(self.source.as_bytes())
                .ok()?
                .to_smolstr(),
            kind: SymbolKind::Local,
            full_range: self.s_range(&node.range()),
            focus_range: self.s_range(&name_node.range()).into(),
            children: vec![],
            details: type_,
            deprecated: self.is_deprecated(node),
        };
        self.alloc(symbol).into()
    }

    fn alloc_global_variable_declaration(&mut self, node: &Node) {
        node.children(&mut node.walk())
            .filter(|child| {
                matches!(
                    TSKind::from(child),
                    TSKind::variable_declaration | TSKind::old_variable_declaration
                )
            })
            .for_each(|child| {
                self.alloc_variable_declaration(&child)
                    .map(|id| self.top(id));
            });
    }

    fn alloc_enum_struct(&mut self, node: &Node) -> Option<SymbolId> {
        let name_node = node.child_by_field_name("name")?;
        let children = node
            .children(&mut node.walk())
            .flat_map(|child| match TSKind::from(&child) {
                TSKind::enum_struct_method => self.alloc_method(&child),
                TSKind::enum_struct_field => {
                    let name_node = child.child_by_field_name("name")?;
                    let type_ = child
                        .child_by_field_name("type")
                        .and_then(|n| n.utf8_text(self.source.as_bytes()).ok())
                        .map(ToString::to_string);
                    let symbol = Symbol {
                        name: name_node
                            .utf8_text(self.source.as_bytes())
                            .ok()?
                            .to_smolstr(),
                        kind: SymbolKind::Variant,
                        full_range: self.s_range(&child.range()),
                        focus_range: self.s_range(&name_node.range()).into(),
                        children: vec![],
                        details: type_,
                        deprecated: self.is_deprecated(&child),
                    };
                    Some(self.alloc(symbol))
                }
                _ => None,
            })
            .collect();
        let symbol = Symbol {
            name: name_node
                .utf8_text(self.source.as_bytes())
                .ok()?
                .to_smolstr(),
            kind: SymbolKind::EnumStruct,
            full_range: self.s_range(&node.range()),
            focus_range: self.s_range(&name_node.range()).into(),
            children,
            details: None,
            deprecated: self.is_deprecated(node),
        };
        self.alloc_top(symbol).into()
    }

    fn alloc_typedef(&mut self, node: &Node) -> Option<SymbolId> {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .and_then(|n| n.utf8_text(self.source.as_bytes()).ok())
            .unwrap_or("typedef")
            .to_smolstr();
        let symbol = Symbol {
            name,
            kind: SymbolKind::Typedef,
            full_range: self.s_range(&node.range()),
            focus_range: name_node.map(|node| self.s_range(&node.range())),
            children: vec![],
            details: node
                .child_by_field_name("parameters")
                .and_then(|n| n.utf8_text(self.source.as_bytes()).ok())
                .map(ToString::to_string),
            deprecated: self.is_deprecated(node),
        };
        if name_node.is_some() {
            self.alloc_top(symbol).into()
        } else {
            self.alloc(symbol).into()
        }
    }

    fn alloc_typeset(&mut self, node: &Node) -> Option<SymbolId> {
        let name_node = node.child_by_field_name("name")?;
        let children = node
            .children(&mut node.walk())
            .filter(|n| TSKind::from(n) == TSKind::typedef_expression)
            .flat_map(|n| self.alloc_typedef(&n))
            .collect();
        let symbol = Symbol {
            name: name_node
                .utf8_text(self.source.as_bytes())
                .ok()?
                .to_smolstr(),
            kind: SymbolKind::Typeset,
            full_range: self.s_range(&node.range()),
            focus_range: self.s_range(&name_node.range()).into(),
            children,
            details: None,
            deprecated: self.is_deprecated(node),
        };
        self.alloc_top(symbol).into()
    }

    fn alloc_functag(&mut self, node: &Node) -> Option<SymbolId> {
        let name_node = node.child_by_field_name("name")?;
        let symbol = Symbol {
            name: name_node
                .utf8_text(self.source.as_bytes())
                .ok()?
                .to_smolstr(),
            kind: SymbolKind::Functag,
            full_range: self.s_range(&node.range()),
            focus_range: self.s_range(&name_node.range()).into(),
            children: vec![],
            details: node
                .child_by_field_name("parameters")
                .and_then(|n| n.utf8_text(self.source.as_bytes()).ok())
                .map(ToString::to_string),
            deprecated: self.is_deprecated(node),
        };
        self.alloc_top(symbol).into()
    }

    fn alloc_funcenum(&mut self, node: &Node) -> Option<SymbolId> {
        let name_node = node.child_by_field_name("name")?;
        let children = node
            .children(&mut node.walk())
            .filter(|n| TSKind::from(n) == TSKind::funcenum_member)
            .map(|n| {
                let symbol = Symbol {
                    name: "functag".to_smolstr(),
                    kind: SymbolKind::Functag,
                    full_range: self.s_range(&n.range()),
                    focus_range: self.s_range(&n.range()).into(),
                    children: vec![],
                    details: node
                        .child_by_field_name("parameters")
                        .and_then(|n| n.utf8_text(self.source.as_bytes()).ok())
                        .map(ToString::to_string),
                    deprecated: self.is_deprecated(&n),
                };
                self.alloc(symbol)
            })
            .collect();
        let symbol = Symbol {
            name: name_node
                .utf8_text(self.source.as_bytes())
                .ok()?
                .to_smolstr(),
            kind: SymbolKind::Funcenum,
            full_range: self.s_range(&node.range()),
            focus_range: self.s_range(&name_node.range()).into(),
            children,
            details: None,
            deprecated: self.is_deprecated(node),
        };
        self.alloc_top(symbol).into()
    }

    fn alloc_struct(&mut self, node: &Node) -> Option<SymbolId> {
        let name_node = node.child_by_field_name("name")?;
        let children = node
            .children(&mut node.walk())
            .filter(|n| TSKind::from(n) == TSKind::struct_field)
            .flat_map(|n| {
                let name_node = n.child_by_field_name("name")?;
                let symbol = Symbol {
                    name: name_node
                        .utf8_text(self.source.as_bytes())
                        .ok()?
                        .to_smolstr(),
                    kind: SymbolKind::Field,
                    full_range: self.s_range(&n.range()),
                    focus_range: self.s_range(&name_node.range()).into(),
                    children: vec![],
                    details: None,
                    deprecated: self.is_deprecated(&n),
                };
                Some(self.alloc(symbol))
            })
            .collect();
        let symbol = Symbol {
            name: name_node
                .utf8_text(self.source.as_bytes())
                .ok()?
                .to_smolstr(),
            kind: SymbolKind::Struct,
            full_range: self.s_range(&node.range()),
            focus_range: self.s_range(&name_node.range()).into(),
            children,
            details: None,
            deprecated: self.is_deprecated(node),
        };
        self.alloc_top(symbol).into()
    }

    fn alloc_struct_declaration(&mut self, node: &Node) -> Option<SymbolId> {
        let name_node = node.child_by_field_name("name")?;
        let symbol = Symbol {
            name: name_node
                .utf8_text(self.source.as_bytes())
                .ok()?
                .to_smolstr(),
            kind: SymbolKind::Struct,
            full_range: self.s_range(&node.range()),
            focus_range: self.s_range(&name_node.range()).into(),
            children: vec![],
            details: None,
            deprecated: self.is_deprecated(node),
        };
        self.alloc_top(symbol).into()
    }

    pub fn build(mut self) -> Symbols {
        self.tree
            .root_node()
            .children(&mut self.tree.root_node().walk())
            .for_each(|node| match TSKind::from(&node) {
                TSKind::function_definition | TSKind::function_declaration => {
                    self.alloc_function(&node);
                }
                TSKind::methodmap => {
                    self.alloc_methodmap(&node);
                }
                TSKind::r#enum => {
                    self.alloc_enum(&node);
                }
                TSKind::global_variable_declaration | TSKind::old_global_variable_declaration => {
                    self.alloc_global_variable_declaration(&node)
                }
                TSKind::enum_struct => {
                    self.alloc_enum_struct(&node);
                }
                TSKind::typedef => {
                    self.alloc_typedef(&node);
                }
                TSKind::typeset => {
                    self.alloc_typeset(&node);
                }
                TSKind::functag => {
                    self.alloc_functag(&node);
                }
                TSKind::funcenum => {
                    self.alloc_funcenum(&node);
                }
                TSKind::r#struct => {
                    self.alloc_struct(&node);
                }
                TSKind::struct_declaration => {
                    self.alloc_struct_declaration(&node);
                }
                _ => (),
            });
        Symbols {
            top_level: self.top_level,
            arena: self.arena,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbols {
    top_level: Vec<SymbolId>,
    arena: Arena<Symbol>,
}

impl Index<&SymbolId> for Symbols {
    type Output = Symbol;
    fn index(&self, id: &SymbolId) -> &Symbol {
        &self.arena[*id]
    }
}

impl<'a> IntoIterator for &'a Symbols {
    type Item = &'a SymbolId;
    type IntoIter = std::slice::Iter<'a, SymbolId>;

    fn into_iter(self) -> Self::IntoIter {
        self.top_level.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: SmolStr,
    pub details: Option<String>,
    pub kind: SymbolKind,
    pub full_range: lsp_types::Range,
    pub focus_range: Option<lsp_types::Range>,
    pub children: Vec<SymbolId>,
    pub deprecated: bool,
}
