use std::sync::Arc;

use base_db::Tree;
use fxhash::FxHashSet;
use la_arena::{Idx, IdxRange, RawIdx};
use lazy_static::lazy_static;
use syntax::TSKind;
use tree_sitter::QueryCursor;
use vfs::FileId;

use crate::{
    ast_id_map::AstIdMap, hir::type_ref::TypeRef, item_tree::Macro, DefDatabase, FileItem, Name,
};

use super::{
    Enum, EnumStruct, EnumStructItemId, Field, Funcenum, Functag, Function, FunctionKind, ItemTree,
    Methodmap, MethodmapItemId, Param, Property, RawVisibilityId, SpecialMethod, Struct,
    StructField, Typedef, Typeset, Variable, Variant,
};

pub(super) struct Ctx<'db> {
    db: &'db dyn DefDatabase,
    tree: ItemTree,
    source_ast_id_map: Arc<AstIdMap>,
    source: Arc<str>,
    file_id: FileId,
    deprecated: FxHashSet<usize>,
}

impl<'db> Ctx<'db> {
    pub(super) fn new(db: &'db dyn DefDatabase, file_id: FileId) -> Self {
        Self {
            db,
            tree: ItemTree::default(),
            source_ast_id_map: db.ast_id_map(file_id),
            source: db.preprocessed_text(file_id),
            file_id,
            deprecated: Default::default(),
        }
    }

    pub(super) fn finish(self) -> Arc<ItemTree> {
        Arc::new(self.tree)
    }

    pub(super) fn lower(&mut self) {
        let tree = self.db.parse(self.file_id);
        self.collect_deprecated(&tree);

        let root_node = tree.root_node();
        for child in root_node.children(&mut root_node.walk()) {
            match TSKind::from(child) {
                TSKind::function_definition | TSKind::function_declaration => {
                    self.lower_function(&child)
                }
                TSKind::r#enum => self.lower_enum(&child),
                TSKind::struct_declaration => self.lower_struct_declaration(&child),
                TSKind::global_variable_declaration => self.lower_global_variable(&child),
                TSKind::old_global_variable_declaration => self.lower_old_global_variable(&child),
                TSKind::enum_struct => self.lower_enum_struct(&child),
                TSKind::methodmap => self.lower_methodmap(&child),
                TSKind::typedef => self.lower_typedef(&child),
                TSKind::typeset => self.lower_typeset(&child),
                TSKind::functag => self.lower_functag(&child),
                TSKind::funcenum => self.lower_funcenum(&child),
                TSKind::r#struct => self.lower_struct(&child),
                _ => (),
            }
        }

        // query for all macro definitions in the file
        lazy_static! {
            static ref MACRO_QUERY: tree_sitter::Query = tree_sitter::Query::new(
                &tree_sitter_sourcepawn::language(),
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
                    let res = Macro {
                        name,
                        ast_id,
                        deprecated: self.is_deprecated(&node),
                    };
                    let id = self.tree.data_mut().macros.alloc(res);
                    self.tree.top_level.push(FileItem::Macro(id));
                }
            }
        }
    }

    fn collect_deprecated(&mut self, tree: &Tree) {
        // query for all pragmas
        lazy_static! {
            static ref MACRO_QUERY: tree_sitter::Query = tree_sitter::Query::new(
                &tree_sitter_sourcepawn::language(),
                "[(preproc_pragma) @pragma]"
            )
            .expect("Could not build pragma query.");
        }
        let mut cursor = QueryCursor::new();
        let matches = cursor.captures(&MACRO_QUERY, tree.root_node(), self.source.as_bytes());
        for (match_, _) in matches {
            for c in match_.captures {
                let Ok(pragma) = c.node.utf8_text(self.source.as_bytes()) else {
                    return;
                };
                if pragma.starts_with("#pragma deprecated") {
                    self.deprecated.insert(c.node.range().start_point.row);
                }
            }
        }
    }

    fn is_deprecated(&self, node: &tree_sitter::Node) -> bool {
        self.deprecated
            .contains(&node.range().start_point.row.saturating_sub(1))
    }

    fn lower_struct_declaration(&mut self, node: &tree_sitter::Node) {
        let visibility = RawVisibilityId::PUBLIC;
        let type_ref = TypeRef::from_returntype_node(node, "type", &self.source);
        if let Some(name_node) = node.child_by_field_name("name") {
            let res = Variable {
                name: Name::from(name_node.utf8_text(self.source.as_bytes()).unwrap()),
                visibility,
                type_ref: type_ref.clone(),
                ast_id: self.source_ast_id_map.ast_id_of(node),
            };
            let id = self.tree.data_mut().variables.alloc(res);
            self.tree.top_level.push(FileItem::Variable(id));
        }
    }

    fn lower_global_variable(&mut self, node: &tree_sitter::Node) {
        let visibility = RawVisibilityId::from_node(node);
        let type_ref = TypeRef::from_returntype_node(node, "type", &self.source);
        for child in node.children(&mut node.walk()) {
            if matches!(
                TSKind::from(child),
                TSKind::variable_declaration | TSKind::dynamic_array_declaration
            ) {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let res = Variable {
                        name: Name::from(name_node.utf8_text(self.source.as_bytes()).unwrap()),
                        visibility,
                        type_ref: type_ref.clone(),
                        ast_id: self.source_ast_id_map.ast_id_of(&child),
                    };
                    let id = self.tree.data_mut().variables.alloc(res);
                    self.tree.top_level.push(FileItem::Variable(id));
                }
            }
        }
    }

    fn lower_old_global_variable(&mut self, node: &tree_sitter::Node) {
        let visibility = RawVisibilityId::from_node(node);
        for child in node
            .children(&mut node.walk())
            .filter(|n| TSKind::from(n) == TSKind::old_variable_declaration)
        {
            let type_ref = TypeRef::from_returntype_node(&child, "type", &self.source);
            if let Some(name_node) = child.child_by_field_name("name") {
                let res = Variable {
                    name: Name::from(name_node.utf8_text(self.source.as_bytes()).unwrap()),
                    visibility,
                    type_ref: type_ref.clone(),
                    ast_id: self.source_ast_id_map.ast_id_of(&child),
                };
                let id = self.tree.data_mut().variables.alloc(res);
                self.tree.top_level.push(FileItem::Variable(id));
            }
        }
    }

    fn function_return_type(&self, node: &tree_sitter::Node) -> Option<TypeRef> {
        TypeRef::from_returntype_node(node, "returnType", &self.source)
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
                        deprecated: self.is_deprecated(&e),
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
            deprecated: self.is_deprecated(node),
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
            deprecated: self.is_deprecated(node),
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
                        type_ref: TypeRef::from_returntype_node(&n, "type", &self.source),
                        ast_id: self.source_ast_id_map.ast_id_of(&n),
                        has_default: n.child_by_field_name("defaultValue").is_some(),
                        is_rest: TSKind::from(n) == TSKind::rest_parameter,
                        is_const: n.child_by_field_name("storage_class").is_some(),
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
            deprecated: self.is_deprecated(node),
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
                    deprecated: self.is_deprecated(&typedef_expr_node),
                };
                let _ = self.tree.data_mut().typedefs.alloc(res);
            });
        let end = self.next_typedef_idx();
        let res = Typeset {
            name,
            typedefs: IdxRange::new(start..end),
            ast_id: self.source_ast_id_map.ast_id_of(node),
            deprecated: self.is_deprecated(node),
        };
        let id = self.tree.data_mut().typesets.alloc(res);

        self.tree.top_level.push(FileItem::Typeset(id));
    }

    fn lower_functag(&mut self, node: &tree_sitter::Node) {
        let Some(name_node) = node.child_by_field_name("name") else {
            return;
        };
        let name = Name::from_node(&name_node, &self.source);
        let type_ref = self.function_return_type(node);
        let params = self.lower_parameters(node);
        let res = Functag {
            name: name.into(),
            params,
            type_ref,
            ast_id: self.source_ast_id_map.ast_id_of(node),
            deprecated: self.is_deprecated(node),
        };
        let id = self.tree.data_mut().functags.alloc(res);
        self.tree.top_level.push(FileItem::Functag(id));
    }

    fn lower_funcenum(&mut self, node: &tree_sitter::Node) {
        let Some(name_node) = node.child_by_field_name("name") else {
            return;
        };
        let name = Name::from_node(&name_node, &self.source);

        let start = self.next_functag_idx();
        node.children(&mut node.walk())
            .filter(|n| TSKind::from(n) == TSKind::funcenum_member)
            .for_each(|funcenum_member_node| {
                let type_ref = self.function_return_type(&funcenum_member_node);
                let params = self.lower_parameters(&funcenum_member_node);
                let res = Functag {
                    name: None,
                    params,
                    type_ref,
                    ast_id: self.source_ast_id_map.ast_id_of(&funcenum_member_node),
                    deprecated: self.is_deprecated(&funcenum_member_node),
                };
                let _ = self.tree.data_mut().functags.alloc(res);
            });
        let end = self.next_functag_idx();
        let res = Funcenum {
            name,
            functags: IdxRange::new(start..end),
            ast_id: self.source_ast_id_map.ast_id_of(node),
            deprecated: self.is_deprecated(node),
        };
        let id = self.tree.data_mut().funcenums.alloc(res);
        self.tree.top_level.push(FileItem::Funcenum(id));
    }

    fn lower_struct(&mut self, node: &tree_sitter::Node) {
        let Some(name_node) = node.child_by_field_name("name") else {
            return;
        };
        let name = Name::from_node(&name_node, &self.source);

        let start = self.next_struct_field_idx();
        node.children(&mut node.walk())
            .filter(|n| TSKind::from(n) == TSKind::struct_field)
            .for_each(|struct_field_node| {
                let Some(type_ref) =
                    TypeRef::from_returntype_node(&struct_field_node, "type", &self.source)
                else {
                    return;
                };
                let Some(name_node) = struct_field_node.child_by_field_name("name") else {
                    return;
                };
                let name = Name::from_node(&name_node, &self.source);
                let res = StructField {
                    name,
                    const_: struct_field_node
                        .children(&mut struct_field_node.walk())
                        .any(|c| TSKind::from(c) == TSKind::anon_const),
                    type_ref,
                    ast_id: self.source_ast_id_map.ast_id_of(&struct_field_node),
                    deprecated: self.is_deprecated(&struct_field_node),
                };
                let _ = self.tree.data_mut().struct_fields.alloc(res);
            });
        let end = self.next_struct_field_idx();
        let res = Struct {
            name,
            fields: IdxRange::new(start..end),
            ast_id: self.source_ast_id_map.ast_id_of(node),
            deprecated: self.is_deprecated(node),
        };
        let id = self.tree.data_mut().structs.alloc(res);
        self.tree.top_level.push(FileItem::Struct(id));
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
                    let Some(type_) = TypeRef::from_returntype_node(&e, "type", &self.source)
                    else {
                        return;
                    };

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
                        deprecated: self.is_deprecated(&e),
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
        let nullable = node
            .children(&mut node.walk())
            .any(|n| TSKind::from(n) == TSKind::anon___nullable__);
        let res = Methodmap {
            name: Name::from(name_node.utf8_text(self.source.as_bytes()).unwrap()),
            items: items.into_boxed_slice(),
            inherits,
            nullable,
            ast_id: self.source_ast_id_map.ast_id_of(node),
            deprecated: self.is_deprecated(node),
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
                    deprecated: self.is_deprecated(node),
                };
                self.tree.data_mut().functions.alloc(res);
            }
            TSKind::methodmap_property_setter => {
                let Some(param_node) = node.child_by_field_name("parameter") else {
                    return;
                };
                let storage_class_node = param_node.child_by_field_name("storage_class");
                let param = Param {
                    type_ref: TypeRef::from_returntype_node(&param_node, "type", &self.source),
                    ast_id: self.source_ast_id_map.ast_id_of(&param_node),
                    has_default: false,
                    is_rest: false,
                    is_const: storage_class_node.is_some(),
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
                    deprecated: self.is_deprecated(node),
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
                    let Some(type_ref) = TypeRef::from_returntype_node(&e, "type", &self.source)
                    else {
                        return;
                    };
                    let res = Field {
                        name: Name::from(
                            field_name_node.utf8_text(self.source.as_bytes()).unwrap(),
                        ),
                        type_ref,
                        ast_id: self.source_ast_id_map.ast_id_of(&e),
                        deprecated: self.is_deprecated(&e),
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
            deprecated: self.is_deprecated(node),
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

    fn next_functag_idx(&self) -> Idx<Functag> {
        Idx::from_raw(RawIdx::from(
            self.tree
                .data
                .as_ref()
                .map_or(0, |data| data.functags.len() as u32),
        ))
    }

    fn next_struct_field_idx(&self) -> Idx<StructField> {
        Idx::from_raw(RawIdx::from(
            self.tree
                .data
                .as_ref()
                .map_or(0, |data| data.struct_fields.len() as u32),
        ))
    }
}
