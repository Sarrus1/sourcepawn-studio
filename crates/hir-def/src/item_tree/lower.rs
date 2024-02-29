use std::sync::Arc;

use la_arena::{Idx, IdxRange, RawIdx};
use lazy_static::lazy_static;
use syntax::TSKind;
use tree_sitter::QueryCursor;
use vfs::FileId;

use crate::{
    ast_id_map::AstIdMap, hir::type_ref::TypeRef, item_tree::Macro, DefDatabase, FileItem, Name,
};

use super::{
    Enum, EnumStruct, EnumStructItemId, Field, Function, FunctionKind, ItemTree, Methodmap,
    MethodmapItemId, Param, Property, RawVisibilityId, SpecialMethod, Typedef, Typeset, Variable,
    Variant,
};

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
            source: db.preprocessed_text(file_id),
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
                TSKind::function_definition | TSKind::function_declaration => {
                    self.lower_function(&child)
                }
                TSKind::r#enum => self.lower_enum(&child),
                TSKind::global_variable_declaration => {
                    let visibility = RawVisibilityId::from_node(&child);
                    let type_ref = child
                        .child_by_field_name("type")
                        .map(|n| TypeRef::from_node(&n, &self.source));
                    for sub_child in child.children(&mut child.walk()) {
                        if TSKind::from(sub_child) == TSKind::variable_declaration {
                            if let Some(name_node) = sub_child.child_by_field_name("name") {
                                let res = Variable {
                                    name: Name::from(
                                        name_node.utf8_text(self.source.as_bytes()).unwrap(),
                                    ),
                                    visibility,
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
                TSKind::methodmap => self.lower_methodmap(&child),
                TSKind::typedef => self.lower_typedef(&child),
                TSKind::typeset => self.lower_typeset(&child),
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
                TSKind::r#type | TSKind::builtin_type | TSKind::identifier => {
                    return TypeRef::from_node(&child, &self.source).into()
                }
                TSKind::old_type => {
                    for sub_child in child.children(&mut child.walk()) {
                        match TSKind::from(sub_child) {
                            TSKind::old_builtin_type | TSKind::identifier | TSKind::any_type => {
                                // FIXME: This is wrong.
                                return Some(TypeRef::OldString);
                            }
                            _ => (),
                        }
                    }
                    return TypeRef::from_node(&child, &self.source).into();
                }
                _ => (),
            }
        }

        None
    }

    fn lower_enum(&mut self, node: &tree_sitter::Node) {
        let start_idx = self.next_variant_idx();
        if let Some(entries_node) = node.child_by_field_name("entries") {
            entries_node
                .children(&mut entries_node.walk())
                .filter(|e| TSKind::from(e) == TSKind::enum_entry)
                .for_each(|e| {
                    let Some(variant_name_node) = e.child_by_field_name("name") else {
                        return;
                    };
                    let name =
                        Name::from(variant_name_node.utf8_text(self.source.as_bytes()).unwrap());
                    let res = Variant {
                        name,
                        ast_id: self.source_ast_id_map.ast_id_of(&e),
                    };
                    let id = self.tree.data_mut().variants.alloc(res);
                    self.tree.top_level.push(FileItem::Variant(id));
                });
        }
        let end_idx = self.next_variant_idx();

        let ast_id = self.source_ast_id_map.ast_id_of(node);
        let name = if let Some(name_node) = node.child_by_field_name("name") {
            Name::from(name_node.utf8_text(self.source.as_bytes()).unwrap())
        } else {
            Name::from(format!("unnamed_enum_{}", ast_id.to_u32()).as_str())
        };
        let res = Enum {
            name,
            variants: IdxRange::new(start_idx..end_idx),
            ast_id: self.source_ast_id_map.ast_id_of(node),
        };
        let id = self.tree.data_mut().enums.alloc(res);
        self.tree.top_level.push(FileItem::Enum(id));
    }

    fn lower_enum_struct_method(
        &mut self,
        node: &tree_sitter::Node,
        items: &mut Vec<EnumStructItemId>,
    ) {
        if let Some(id) = self.lower_function_(node, None, None, None) {
            items.push(EnumStructItemId::Method(id));
        }
    }

    fn lower_methodmap_method(
        &mut self,
        node: &tree_sitter::Node,
        items: &mut Vec<MethodmapItemId>,
        kind: Option<FunctionKind>,
        special: Option<SpecialMethod>,
    ) {
        let mut visibility = RawVisibilityId::PUBLIC;
        if node
            .children(&mut node.walk())
            .any(|n| TSKind::from(n) == TSKind::anon_static)
        {
            visibility |= RawVisibilityId::STATIC;
        }
        if let Some(id) = self.lower_function_(node, visibility.into(), kind, special) {
            items.push(MethodmapItemId::Method(id));
        }
    }

    fn lower_function(&mut self, node: &tree_sitter::Node) {
        if let Some(id) = self.lower_function_(node, None, None, None) {
            self.tree.top_level.push(FileItem::Function(id));
        }
    }

    fn lower_function_(
        &mut self,
        node: &tree_sitter::Node,
        visibility: Option<RawVisibilityId>,
        kind: Option<FunctionKind>,
        special: Option<SpecialMethod>,
    ) -> Option<Idx<Function>> {
        let kind = match kind {
            Some(kind) => kind,
            None => FunctionKind::from_node(node),
        };
        let params = self.lower_parameters(node);
        let name_node = node.child_by_field_name("name")?;
        let visibility = visibility.unwrap_or_else(|| RawVisibilityId::from_node(node));
        let res = Function {
            name: Name::from(name_node.utf8_text(self.source.as_bytes()).unwrap()),
            kind,
            ret_type: self.function_return_type(node),
            visibility,
            params,
            special,
            ast_id: self.source_ast_id_map.ast_id_of(node),
        };

        self.tree.data_mut().functions.alloc(res).into()
    }

    fn lower_parameters(&mut self, node: &tree_sitter::Node) -> IdxRange<Param> {
        let start_param_idx = self.next_param_idx();
        let Some(params_node) = node.child_by_field_name("parameters") else {
            return IdxRange::new(start_param_idx..start_param_idx);
        };
        assert!(TSKind::from(params_node) == TSKind::parameter_declarations);
        params_node
            .children(&mut params_node.walk())
            .for_each(|n| match TSKind::from(n) {
                TSKind::parameter_declaration | TSKind::rest_parameter => {
                    let res = Param {
                        type_ref: n
                            .child_by_field_name("type")
                            .map(|n| TypeRef::from_node(&n, &self.source)),
                        ast_id: self.source_ast_id_map.ast_id_of(&n),
                        has_default: n.child_by_field_name("defaultValue").is_some(),
                    };
                    self.tree.data_mut().params.alloc(res);
                }
                _ => (),
            });
        let end_param_idx = self.next_param_idx();
        IdxRange::new(start_param_idx..end_param_idx)
    }

    fn lower_typedef(&mut self, node: &tree_sitter::Node) {
        let Some(name_node) = node.child_by_field_name("name") else {
            return;
        };
        let name = Name::from_node(&name_node, &self.source);
        let Some(typedef_expr_node) = node
            .children(&mut node.walk())
            .find(|n| TSKind::from(n) == TSKind::typedef_expression)
        else {
            return;
        };
        let Some(type_ref) = self.function_return_type(&typedef_expr_node) else {
            return;
        };
        let params = self.lower_parameters(&typedef_expr_node);
        let res = Typedef {
            name: name.into(),
            params,
            type_ref,
            ast_id: self.source_ast_id_map.ast_id_of(node),
        };
        let id = self.tree.data_mut().typedefs.alloc(res);
        self.tree.top_level.push(FileItem::Typedef(id));
    }

    fn lower_typeset(&mut self, node: &tree_sitter::Node) {
        let Some(name_node) = node.child_by_field_name("name") else {
            return;
        };
        let name = Name::from_node(&name_node, &self.source);

        let start = self.next_typedef_idx();
        node.children(&mut node.walk())
            .filter(|n| TSKind::from(n) == TSKind::typedef_expression)
            .for_each(|typedef_expr_node| {
                let Some(type_ref) = self.function_return_type(&typedef_expr_node) else {
                    return;
                };
                let params = self.lower_parameters(&typedef_expr_node);
                let res = Typedef {
                    name: None,
                    params,
                    type_ref,
                    ast_id: self.source_ast_id_map.ast_id_of(&typedef_expr_node),
                };
                let _ = self.tree.data_mut().typedefs.alloc(res);
            });
        let end = self.next_typedef_idx();
        let res = Typeset {
            name,
            typedefs: IdxRange::new(start..end),
            ast_id: self.source_ast_id_map.ast_id_of(node),
        };
        let id = self.tree.data_mut().typesets.alloc(res);

        self.tree.top_level.push(FileItem::Typeset(id));
    }

    fn lower_methodmap(&mut self, node: &tree_sitter::Node) {
        let Some(name_node) = node.child_by_field_name("name") else {
            return;
        };
        let mut items = Vec::new();
        node.children(&mut node.walk())
            .for_each(|e| match TSKind::from(e) {
                TSKind::methodmap_property => {
                    let Some(property_name_node) = e.child_by_field_name("name") else {
                        return;
                    };
                    let Some(property_type_node) = e.child_by_field_name("type") else {
                        return;
                    };
                    let type_ = TypeRef::from_node(&property_type_node, &self.source);

                    let start_idx = self.next_function_idx();
                    e.children(&mut e.walk())
                        .for_each(|e1| match TSKind::from(e1) {
                            TSKind::methodmap_property_method => {
                                e1.children(&mut e1.walk()).for_each(|e2| {
                                    self.lower_property_method(
                                        &e2,
                                        &e1,
                                        type_.clone(),
                                        FunctionKind::Def,
                                    )
                                })
                            }
                            TSKind::methodmap_property_native => {
                                e1.children(&mut e.walk()).for_each(|e2| {
                                    self.lower_property_method(
                                        &e2,
                                        &e1,
                                        type_.clone(),
                                        FunctionKind::Native,
                                    )
                                })
                            }
                            TSKind::methodmap_property_alias => (), //TODO: Handle this node
                            _ => (),
                        });
                    let end_idx = self.next_function_idx();

                    let res = Property {
                        name: Name::from(
                            property_name_node
                                .utf8_text(self.source.as_bytes())
                                .unwrap(),
                        ),
                        getters_setters: IdxRange::new(start_idx..end_idx),
                        type_ref: type_,
                        ast_id: self.source_ast_id_map.ast_id_of(&e),
                    };
                    let property_idx = self.tree.data_mut().properties.alloc(res);
                    items.push(MethodmapItemId::Property(property_idx));
                }
                TSKind::methodmap_method => self.lower_methodmap_method(&e, &mut items, None, None),
                TSKind::methodmap_method_constructor => self.lower_methodmap_method(
                    &e,
                    &mut items,
                    None,
                    Some(SpecialMethod::Constructor),
                ),
                TSKind::methodmap_method_destructor => self.lower_methodmap_method(
                    &e,
                    &mut items,
                    None,
                    Some(SpecialMethod::Destructor),
                ),
                TSKind::methodmap_native => {
                    self.lower_methodmap_method(&e, &mut items, Some(FunctionKind::Native), None)
                }
                TSKind::methodmap_native_constructor => self.lower_methodmap_method(
                    &e,
                    &mut items,
                    Some(FunctionKind::Native),
                    Some(SpecialMethod::Constructor),
                ),
                TSKind::methodmap_native_destructor => self.lower_methodmap_method(
                    &e,
                    &mut items,
                    Some(FunctionKind::Native),
                    Some(SpecialMethod::Destructor),
                ),
                _ => (),
            });
        let inherits = node
            .child_by_field_name("inherits")
            .and_then(|n| n.utf8_text(self.source.as_bytes()).map(Name::from).ok());
        let res = Methodmap {
            name: Name::from(name_node.utf8_text(self.source.as_bytes()).unwrap()),
            items: items.into_boxed_slice(),
            inherits,
            ast_id: self.source_ast_id_map.ast_id_of(node),
        };
        let id = self.tree.data_mut().methodmaps.alloc(res);
        self.tree.top_level.push(FileItem::Methodmap(id));
    }

    fn lower_property_method(
        &mut self,
        node: &tree_sitter::Node,
        parent: &tree_sitter::Node,
        type_: TypeRef,
        kind: FunctionKind,
    ) {
        match TSKind::from(node) {
            TSKind::methodmap_property_getter => {
                let idx = self.next_param_idx();
                let res = Function {
                    name: Name::from("get"),
                    kind,
                    ret_type: type_.clone().into(),
                    visibility: RawVisibilityId::PUBLIC,
                    params: IdxRange::new(idx..idx),
                    special: None,
                    ast_id: self.source_ast_id_map.ast_id_of(parent),
                };
                self.tree.data_mut().functions.alloc(res);
            }
            TSKind::methodmap_property_setter => {
                let Some(param_node) = node.child_by_field_name("parameter") else {
                    return;
                };
                let storage_class_node = param_node.child_by_field_name("storage_class"); // FIXME: Handle this node
                let Some(param_type_node) = param_node.child_by_field_name("type") else {
                    return;
                };
                let param = Param {
                    type_ref: TypeRef::from_node(&param_type_node, &self.source).into(),
                    ast_id: self.source_ast_id_map.ast_id_of(&param_node),
                    has_default: false,
                };
                let start_idx = self.next_param_idx();
                self.tree.data_mut().params.alloc(param);
                let end_idx = self.next_param_idx();
                let res = Function {
                    name: Name::from("set"),
                    kind,
                    ret_type: None,
                    visibility: RawVisibilityId::NONE,
                    params: IdxRange::new(start_idx..end_idx),
                    special: None,
                    ast_id: self.source_ast_id_map.ast_id_of(parent), // We care about the method itself, not the getter/setter in the grammar.
                };
                self.tree.data_mut().functions.alloc(res);
            }
            _ => (),
        }
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
                            field_name_node.utf8_text(self.source.as_bytes()).unwrap(),
                        ),
                        type_ref: TypeRef::from_node(&field_type_node, &self.source),
                        ast_id: self.source_ast_id_map.ast_id_of(&e),
                    };
                    let field_idx = self.tree.data_mut().fields.alloc(res);
                    items.push(EnumStructItemId::Field(field_idx));
                }
                TSKind::enum_struct_method => self.lower_enum_struct_method(&e, &mut items),
                _ => (),
            });
        let res = EnumStruct {
            name: Name::from(name_node.utf8_text(self.source.as_bytes()).unwrap()),
            items: items.into_boxed_slice(),
            ast_id: self.source_ast_id_map.ast_id_of(node),
        };
        let id = self.tree.data_mut().enum_structs.alloc(res);
        self.tree.top_level.push(FileItem::EnumStruct(id));
    }

    fn next_param_idx(&self) -> Idx<Param> {
        Idx::from_raw(RawIdx::from(
            self.tree
                .data
                .as_ref()
                .map_or(0, |data| data.params.len() as u32),
        ))
    }

    fn next_function_idx(&self) -> Idx<Function> {
        Idx::from_raw(RawIdx::from(
            self.tree
                .data
                .as_ref()
                .map_or(0, |data| data.functions.len() as u32),
        ))
    }

    fn next_variant_idx(&self) -> Idx<Variant> {
        Idx::from_raw(RawIdx::from(
            self.tree
                .data
                .as_ref()
                .map_or(0, |data| data.variants.len() as u32),
        ))
    }

    fn next_typedef_idx(&self) -> Idx<Typedef> {
        Idx::from_raw(RawIdx::from(
            self.tree
                .data
                .as_ref()
                .map_or(0, |data| data.typedefs.len() as u32),
        ))
    }
}
