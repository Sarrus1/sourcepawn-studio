use std::sync::Arc;

use la_arena::Idx;
use lazy_static::lazy_static;
use syntax::TSKind;
use tree_sitter::QueryCursor;
use vfs::FileId;

use crate::{
    ast_id_map::AstIdMap, hir::type_ref::TypeRef, item_tree::Macro, DefDatabase, FileItem, Name,
};

use super::{EnumStruct, EnumStructItemId, Field, Function, ItemTree, Variable};

pub(super) struct Ctx<'db> {
    db: &'db dyn DefDatabase,
    tree: ItemTree,
    source_ast_id_map: Arc<AstIdMap>,
    source: Arc<str>,
    file_id: FileId,
}

impl<'db> Ctx<'db> {
    pub(super) fn new(db: &'db dyn DefDatabase, file_id: FileId) -> Self {
        Self {
            db,
            tree: ItemTree::default(),
            source_ast_id_map: db.ast_id_map(file_id),
            source: db.file_text(file_id),
            file_id,
        }
    }

    pub(super) fn finish(self) -> Arc<ItemTree> {
        Arc::new(self.tree)
    }

    pub(super) fn lower(&mut self) {
        let tree = self.db.parse(self.file_id);
        let root_node = tree.root_node();
        for child in root_node.children(&mut root_node.walk()) {
            match TSKind::from(child) {
                TSKind::function_definition => self.lower_function(&child),
                TSKind::global_variable_declaration => {
                    let type_ref = if let Some(type_node) = child.child_by_field_name("type") {
                        TypeRef::from_node(&type_node, &self.source)
                    } else {
                        None
                    };
                    for sub_child in child.children(&mut child.walk()) {
                        if TSKind::from(sub_child) == TSKind::variable_declaration {
                            if let Some(name_node) = sub_child.child_by_field_name("name") {
                                let res = Variable {
                                    name: Name::from(
                                        name_node.utf8_text(self.source.as_bytes()).unwrap(),
                                    ),
                                    type_ref: type_ref.clone(),
                                    ast_id: self.source_ast_id_map.ast_id_of(&sub_child),
                                };
                                let id = self.tree.data_mut().variables.alloc(res);
                                self.tree.top_level.push(FileItem::Variable(id));
                            }
                        }
                    }
                }
                TSKind::enum_struct => self.lower_enum_struct(&child),
                _ => (),
            }
        }

        // query for all macro definitions in the file
        lazy_static! {
            static ref MACRO_QUERY: tree_sitter::Query = tree_sitter::Query::new(
                tree_sitter_sourcepawn::language(),
                "[(preproc_macro) @macro (preproc_define) @define]"
            )
            .expect("Could not build macro query.");
        }

        let mut cursor = QueryCursor::new();
        let matches = cursor.captures(&MACRO_QUERY, tree.root_node(), self.source.as_bytes());
        for (match_, _) in matches {
            for c in match_.captures {
                let node = c.node;
                if let Some(name) = node
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(self.source.as_bytes()).ok())
                    .map(Name::from)
                {
                    let ast_id = self.source_ast_id_map.ast_id_of(&node);
                    let res = Macro { name, ast_id };
                    let id = self.tree.data_mut().macros.alloc(res);
                    self.tree.top_level.push(FileItem::Macro(id));
                }
            }
        }
    }

    fn function_return_type(&self, node: &tree_sitter::Node) -> Option<TypeRef> {
        let ret_type_node = node.child_by_field_name("returnType")?;
        for child in ret_type_node.children(&mut ret_type_node.walk()) {
            match TSKind::from(child) {
                TSKind::r#type => return TypeRef::from_node(&child, &self.source),
                TSKind::old_type => {
                    for sub_child in child.children(&mut child.walk()) {
                        match TSKind::from(sub_child) {
                            TSKind::old_builtin_type | TSKind::identifier | TSKind::any_type => {
                                return Some(TypeRef::OldString)
                            }
                            _ => (),
                        }
                    }
                    return TypeRef::from_node(&child, &self.source);
                }
                _ => (),
            }
        }
        None
    }

    fn lower_method(&mut self, node: &tree_sitter::Node, items: &mut Vec<EnumStructItemId>) {
        if let Some(id) = self.lower_function_(node) {
            items.push(EnumStructItemId::Method(id));
        }
    }

    fn lower_function(&mut self, node: &tree_sitter::Node) {
        if let Some(id) = self.lower_function_(node) {
            self.tree.top_level.push(FileItem::Function(id));
        }
    }

    fn lower_function_(&mut self, node: &tree_sitter::Node) -> Option<Idx<Function>> {
        let name_node = node.child_by_field_name("name")?;
        let res = Function {
            name: Name::from(name_node.utf8_text(self.source.as_bytes()).unwrap()),
            ret_type: self.function_return_type(node),
            ast_id: self.source_ast_id_map.ast_id_of(node),
        };
        Some(self.tree.data_mut().functions.alloc(res))
    }

    fn lower_enum_struct(&mut self, node: &tree_sitter::Node) {
        let Some(name_node) = node.child_by_field_name("name") else {
            return;
        };
        let mut items = Vec::new();
        node.children(&mut node.walk())
            .for_each(|e| match TSKind::from(e) {
                TSKind::enum_struct_field => {
                    let Some(field_name_node) = e.child_by_field_name("name") else {
                        return;
                    };
                    let Some(field_type_node) = e.child_by_field_name("type") else {
                        return;
                    };
                    let res = Field {
                        name: Name::from(
                            field_name_node.utf8_text(&self.source.as_bytes()).unwrap(),
                        ),
                        type_ref: TypeRef::from_node(&field_type_node, &self.source).unwrap(),
                        ast_id: self.source_ast_id_map.ast_id_of(&e),
                    };
                    let field_idx = self.tree.data_mut().fields.alloc(res);
                    items.push(EnumStructItemId::Field(field_idx));
                }
                TSKind::enum_struct_method => {
                    self.lower_method(&e, &mut items);
                }
                _ => return,
            });
        let res = EnumStruct {
            name: Name::from(name_node.utf8_text(self.source.as_bytes()).unwrap()),
            items: items.into_boxed_slice(),
            ast_id: self.source_ast_id_map.ast_id_of(&node),
        };
        let id = self.tree.data_mut().enum_structs.alloc(res);
        self.tree.top_level.push(FileItem::EnumStruct(id));
    }
}
