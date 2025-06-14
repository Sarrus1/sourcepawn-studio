//! Defines the [`Semantics`](Semantics) struct.

use std::{cell::RefCell, fmt, ops, sync::Arc};

use base_db::{is_field_receiver_node, is_name_node, FilePosition, FileRange, Tree};
use hir_def::{
    resolve_include_node,
    resolver::{global_resolver, HasResolver, ValueNs},
    FileDefId, FunctionId, InFile, Name, NodePtr, PropertyItem,
};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::warn;
use preprocessor::ExpandedSymbolOffset;
use smol_str::ToSmolStr;
use sourcepawn_lexer::{SourcepawnLexer, TextSize, TokenKind};
use streaming_iterator::StreamingIterator;
use syntax::{utils::ts_range_to_text_range, TSKind};
use tree_sitter::QueryCursor;
use vfs::FileId;

use crate::{
    db::HirDatabase,
    source_analyzer::SourceAnalyzer,
    source_to_def::{SourceToDefCache, SourceToDefCtx},
    Attribute, DefResolution, Enum, EnumStruct, Field, File, Function, Global, Macro, Methodmap,
    Property, Struct, Variant,
};

/// Primary API to get semantic information, like types, from syntax trees.
///
/// For now, it only allows to get from a node in a tree-sitter CST, to a definition.
pub struct Semantics<'db, DB> {
    pub db: &'db DB,
    imp: SemanticsImpl<'db>,
}

pub struct SemanticsImpl<'db> {
    pub db: &'db dyn HirDatabase,
    s2d_cache: RefCell<SourceToDefCache>,
}

impl<DB> fmt::Debug for Semantics<'_, DB> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Semantics {{ ... }}")
    }
}

impl<'db, DB> ops::Deref for Semantics<'db, DB> {
    type Target = SemanticsImpl<'db>;

    fn deref(&self) -> &Self::Target {
        &self.imp
    }
}

impl<DB: HirDatabase> Semantics<'_, DB> {
    pub fn new(db: &DB) -> Semantics<'_, DB> {
        let impl_ = SemanticsImpl::new(db);
        Semantics { db, imp: impl_ }
    }

    pub fn parse(&self, file_id: FileId) -> Tree {
        self.db.parse(file_id)
    }

    pub fn file_text(&self, file_id: FileId) -> Arc<str> {
        self.db.file_text(file_id)
    }

    pub fn preprocess_file(&self, file_id: FileId) -> Arc<preprocessor::PreprocessingResult> {
        self.db.preprocess_file(file_id)
    }

    pub fn preprocessed_text(&self, file_id: FileId) -> Arc<str> {
        self.db.preprocessed_text(file_id)
    }

    pub fn find_name_def(
        &self,
        file_id: FileId,
        node: &tree_sitter::Node,
    ) -> Option<DefResolution> {
        if !is_name_node(node) {
            return None;
        }
        let mut parent = node.parent()?;

        if matches!(
            TSKind::from(parent),
            TSKind::methodmap_property_setter | TSKind::methodmap_property_getter
        ) {
            // Convert the setter/getter to its parent, the methodmap_property_method.
            // The actual method like declaration in the grammar is one level above in the tree.
            parent = parent.parent()?;
        }
        let src = InFile::new(file_id, NodePtr::from(&parent));
        match TSKind::from(parent) {
            TSKind::function_definition
            | TSKind::function_declaration
            | TSKind::enum_struct_method
            | TSKind::methodmap_native
            | TSKind::methodmap_native_constructor
            | TSKind::methodmap_native_destructor
            | TSKind::methodmap_method
            | TSKind::methodmap_method_constructor
            | TSKind::methodmap_method_destructor
            | TSKind::methodmap_property_native
            | TSKind::methodmap_property_method => self
                .fn_to_def(src)
                .map(Function::from)
                .map(DefResolution::Function),
            TSKind::enum_struct => self
                .enum_struct_to_def(src)
                .map(EnumStruct::from)
                .map(DefResolution::EnumStruct),
            TSKind::enum_struct_field => self
                .field_to_def(src)
                .map(Field::from)
                .map(DefResolution::Field),
            TSKind::parameter_declaration => self.local_to_def(src).map(|it| it.into()),
            TSKind::variable_declaration
            | TSKind::old_variable_declaration
            | TSKind::dynamic_array_declaration => {
                let grand_parent = parent.parent()?;
                match TSKind::from(&grand_parent) {
                    TSKind::global_variable_declaration
                    | TSKind::old_global_variable_declaration => self
                        .global_to_def(src)
                        .map(Global::from)
                        .map(DefResolution::Global),
                    TSKind::variable_declaration_statement
                    | TSKind::old_variable_declaration_statement
                    | TSKind::old_for_loop_variable_declaration_statement => {
                        self.local_to_def(src).map(|it| it.into())
                    }
                    _ => unreachable!(),
                }
            }
            TSKind::preproc_macro | TSKind::preproc_define => {
                self.macro_to_def(src).map(DefResolution::Macro)
            }
            TSKind::preproc_undefine => {
                let source = self.file_text(file_id);
                let ValueNs::MacroId(id) = file_id
                    .resolver(self.db)
                    .resolve_ident(node.utf8_text(source.as_bytes()).ok()?)?
                else {
                    return None;
                };
                DefResolution::Macro(Macro::from(id.value)).into()
            }
            TSKind::r#enum => self.enum_to_def(src).map(DefResolution::Enum),
            TSKind::enum_entry => self.variant_to_def(src).map(DefResolution::Variant),
            TSKind::methodmap => self.methodmap_to_def(src).map(DefResolution::Methodmap),
            TSKind::methodmap_property => self
                .property_to_def(src)
                .map(Property::from)
                .map(DefResolution::Property),
            TSKind::typedef => self.typedef_to_def(src).map(DefResolution::Typedef),
            TSKind::typeset => self.typeset_to_def(src).map(DefResolution::Typeset),
            TSKind::functag => self.functag_to_def(src).map(DefResolution::Functag),
            TSKind::funcenum => self.funcenum_to_def(src).map(DefResolution::Funcenum),
            TSKind::r#struct => self.struct_to_def(src).map(DefResolution::Struct),
            TSKind::struct_field => self
                .struct_field_to_def(src)
                .map(DefResolution::StructField),
            TSKind::struct_declaration => self
                .global_to_def(src)
                .map(Global::from)
                .map(DefResolution::Global),
            TSKind::struct_field_value => todo!(),
            _ => unreachable!(),
        }
    }

    /// Try to find the definition of a macro at the given [`user position`](FilePosition).
    pub fn find_macro_def(
        &self,
        fpos: &FilePosition,
    ) -> Option<(ExpandedSymbolOffset, DefResolution)> {
        let preprocessing_results = self.preprocess_file(fpos.file_id);
        let source_map = preprocessing_results.source_map();
        let offset = source_map.expanded_symbol_from_u_pos(fpos.offset)?;
        let file_id = offset.file_id();
        let idx = offset.idx();

        (
            offset,
            self.db
                .file_def_map(file_id)
                .get_macro(&idx)
                .map(Macro::from)
                .map(DefResolution::from)?,
        )
            .into()
    }

    /// Find the type of an expression node.
    ///
    /// # Arguments
    /// * `file_id` - The [`file_id`](FileId) of the file containing the node.
    /// * `node` - The expression node.
    pub fn find_type_def(
        &self,
        file_id: FileId,
        mut node: tree_sitter::Node,
    ) -> Option<DefResolution> {
        log::debug!("finding type of node: {}", node.to_sexp());
        while !matches!(TSKind::from(node), TSKind::identifier | TSKind::this) {
            node = match TSKind::from(node) {
                TSKind::array_indexed_access => node.child_by_field_name("array")?,
                TSKind::assignment_expression => node.child_by_field_name("left")?,
                TSKind::call_expression => node.child_by_field_name("function")?,
                TSKind::ternary_expression => node.child_by_field_name("consequence")?,
                TSKind::field_access => node.child_by_field_name("target")?,
                TSKind::scope_access => node.child_by_field_name("scope")?,
                TSKind::binary_expression => node.child_by_field_name("left")?,
                TSKind::unary_expression => node.child_by_field_name("argument")?,
                TSKind::update_expression => node.child_by_field_name("argument")?,
                TSKind::view_as => node.child_by_field_name("type")?,
                TSKind::old_type_cast => node.child_by_field_name("type")?,
                TSKind::parenthesized_expression => node.child_by_field_name("expression")?,
                TSKind::r#type => node.child(0)?,
                TSKind::old_type => node.child(0)?,
                TSKind::new_expression => node.child_by_field_name("class")?,
                _ => return None,
            }
        }
        self.find_def(file_id, &node)
            .and_then(|def| def.type_def(self.db))
    }

    /// Find a definition given a reference node.
    ///
    /// # Arguments
    /// * `file_id` - The [`file_id`](FileId) of the file containing the reference.
    /// * `node` - The reference node.
    pub fn find_def(&self, file_id: FileId, node: &tree_sitter::Node) -> Option<DefResolution> {
        let source = self.db.preprocessed_text(file_id);
        if let Some(res) = self.find_name_def(file_id, node) {
            return res.into();
        }

        if TSKind::from(node) == TSKind::anon_float {
            // Avoid an edge case were the type `float` is mistaken for the method `float`.
            // https://github.com/Sarrus1/sourcepawn-studio/issues/400
            let grandparent_kind = node
                .parent()
                .and_then(|parent| parent.parent())
                .map(TSKind::from);

            if grandparent_kind != Some(TSKind::call_expression) {
                return None;
            }
        }

        let mut container = node.parent()?;
        // If the node does not have a parent we are at the root, nothing to resolve.
        while !matches!(
            TSKind::from(container),
            TSKind::function_definition
                | TSKind::enum_struct_method
                | TSKind::r#enum
                | TSKind::methodmap_native
                | TSKind::methodmap_native_constructor
                | TSKind::methodmap_native_destructor
                | TSKind::methodmap_method
                | TSKind::methodmap_method_constructor
                | TSKind::methodmap_method_destructor
                | TSKind::methodmap_property_getter
                | TSKind::methodmap_property_setter
                | TSKind::methodmap_property_native
                | TSKind::methodmap_property_method
                | TSKind::typedef
                | TSKind::struct_constructor
        ) {
            if let Some(candidate) = container.parent() {
                container = candidate;
            } else {
                break;
            }
        }

        if node.is_error() {
            return None;
        }
        let parent_kind = TSKind::from(node.parent()?);
        if parent_kind == TSKind::preproc_include || parent_kind == TSKind::preproc_tryinclude {
            return self.include_node_to_def(file_id, node.parent()?, source);
        }
        match TSKind::from(container) {
            TSKind::function_definition => {
                self.function_node_to_def(file_id, container, *node, source)
            }
            TSKind::methodmap_property_getter | TSKind::methodmap_property_setter => {
                self.property_getter_setter_node_to_def(file_id, container, *node, source)
            }
            TSKind::methodmap_property_native | TSKind::methodmap_property_method => {
                self.property_method_node_to_def(file_id, container, *node, source)
            }
            TSKind::enum_struct_method
            | TSKind::methodmap_native
            | TSKind::methodmap_native_constructor
            | TSKind::methodmap_native_destructor
            | TSKind::methodmap_method
            | TSKind::methodmap_method_constructor
            | TSKind::methodmap_method_destructor => {
                self.method_node_to_def(file_id, container, *node, source)
            }
            TSKind::typedef => self.typedef_node_to_def(file_id, container, *node, source),
            TSKind::functag => self.functag_node_to_def(file_id, container, *node, source),
            TSKind::r#enum => self.source_node_to_def(file_id, *node, source), // Variants are in the global scope
            TSKind::struct_constructor => {
                self.struct_node_to_def(file_id, container.parent()?, *node, source)
            }
            TSKind::source_file => self.source_node_to_def(file_id, *node, source),
            _ => {
                warn!("{} is not expected", container.kind());
                None
            }
        }
    }

    fn source_node_to_def(
        &self,
        file_id: FileId,
        node: tree_sitter::Node,
        source: Arc<str>,
    ) -> Option<DefResolution> {
        let resolver = global_resolver(self.db, file_id);
        let text = node.utf8_text(source.as_ref().as_bytes()).ok()?;
        match resolver.resolve_ident(text)? {
            ValueNs::FunctionId(ids) => ids
                .iter()
                .find(|id| file_id == id.file_id)
                .map(|id| id.value)
                .map(Function::from)
                .map(DefResolution::Function),
            ValueNs::MacroId(id) => DefResolution::Macro(Macro::from(id.value)).into(),
            ValueNs::GlobalId(id) => DefResolution::Global(Global::from(id.value)).into(),
            ValueNs::EnumStructId(id) => {
                DefResolution::EnumStruct(EnumStruct::from(id.value)).into()
            }
            ValueNs::MethodmapId(id) => DefResolution::Methodmap(Methodmap::from(id.value)).into(),
            ValueNs::EnumId(id) => DefResolution::Enum(Enum::from(id.value)).into(),
            ValueNs::VariantId(id) => DefResolution::Variant(Variant::from(id.value)).into(),
            ValueNs::StructId(id) => DefResolution::Struct(Struct::from(id.value)).into(),
            _ => None,
        }
    }

    fn include_node_to_def(
        &self,
        file_id: FileId,
        node: tree_sitter::Node,
        source: Arc<str>,
    ) -> Option<DefResolution> {
        let (id, ..) = resolve_include_node(self.db, file_id, source.as_ref(), node)?;
        id.map(|id| DefResolution::File(id.into()))
    }

    fn property_getter_setter_node_to_def(
        &self,
        file_id: FileId,
        container: tree_sitter::Node,
        node: tree_sitter::Node,
        source: Arc<str>,
    ) -> Option<DefResolution> {
        self.property_method_node_to_def(file_id, container.parent()?, node, source)
    }

    fn property_method_node_to_def(
        &self,
        file_id: FileId,
        container: tree_sitter::Node,
        node: tree_sitter::Node,
        source: Arc<str>,
    ) -> Option<DefResolution> {
        let property_method_node = container;
        let property_node = property_method_node.parent()?;
        let methodmap_node = property_node.parent()?;
        let method_kind = property_method_node
            .children(&mut property_method_node.walk())
            .find_map(|node| match TSKind::from(node) {
                TSKind::methodmap_property_getter => Some(TSKind::methodmap_property_getter),
                TSKind::methodmap_property_setter => Some(TSKind::methodmap_property_setter),
                _ => None,
            })?;
        let property_name = property_node
            .child_by_field_name("name")?
            .utf8_text(source.as_ref().as_bytes())
            .ok()?;
        let methodmap_name = methodmap_node
            .child_by_field_name("name")?
            .utf8_text(source.as_ref().as_bytes())
            .ok()?;

        let def_map = self.db.file_def_map(file_id);
        let body_node = container.child_by_field_name("body")?;
        debug_assert_eq!(TSKind::from(body_node), TSKind::block);
        let hir_def::FileDefId::MethodmapId(id) = def_map.get_first_from_str(methodmap_name)?
        else {
            return None;
        };
        let data = self.db.methodmap_data(id);
        let property_idx = data.items(&Name::from(property_name))?;
        let property_data = data.property(property_idx)?;
        let id = property_data
            .getters_setters
            .iter()
            .find_map(|item| match *item {
                PropertyItem::Getter(fn_id) if method_kind == TSKind::methodmap_property_getter => {
                    Some(fn_id)
                }
                PropertyItem::Setter(fn_id) if method_kind == TSKind::methodmap_property_setter => {
                    Some(fn_id)
                }
                _ => None,
            })?;

        self.function_node_to_def_(file_id, container, node.parent()?, node, source, id)
    }

    fn method_node_to_def(
        &self,
        file_id: FileId,
        container: tree_sitter::Node,
        node: tree_sitter::Node,
        source: Arc<str>,
    ) -> Option<DefResolution> {
        let def_map = self.db.file_def_map(file_id);
        let parent = node.parent()?;

        let method_name = container
            .child_by_field_name("name")?
            .utf8_text(source.as_ref().as_bytes())
            .ok()?;

        let container_name = container
            .parent()?
            .child_by_field_name("name")?
            .utf8_text(source.as_ref().as_bytes())
            .ok()?;
        let id = match def_map.get_first_from_str(container_name)? {
            hir_def::FileDefId::EnumStructId(es_id) => {
                let data = self.db.enum_struct_data(es_id);
                let method_idx = data.items(&Name::from(method_name))?;
                data.method(method_idx).cloned()?
            }
            hir_def::FileDefId::MethodmapId(id) => {
                let data = self.db.methodmap_data(id);
                let method_idx = data.items(&Name::from(method_name))?;
                data.method(method_idx).cloned()?
            }
            _ => return None,
        };

        self.function_node_to_def_(file_id, container, parent, node, source, id)
    }

    fn function_node_to_def_(
        &self,
        file_id: FileId,
        container: tree_sitter::Node,
        parent: tree_sitter::Node,
        node: tree_sitter::Node,
        source: Arc<str>,
        id: FunctionId,
    ) -> Option<DefResolution> {
        let text = node.utf8_text(source.as_ref().as_bytes()).ok()?;
        let def = hir_def::DefWithBodyId::FunctionId(id);
        let Some(body_node) = container.child_by_field_name("body") else {
            // If the function has no body, try to resolve params and return type.
            let analyzer = SourceAnalyzer::new_no_body_no_infer(self.db, def, file_id);
            return DefResolution::try_from(analyzer.resolver.resolve_ident(text)?);
        };
        debug_assert_eq!(TSKind::from(body_node), TSKind::block);
        let offset = TextSize::new(node.start_byte() as u32);
        match TSKind::from(parent) {
            TSKind::field_access | TSKind::scope_access if is_field_receiver_node(&node) => {
                let analyzer = SourceAnalyzer::new_for_body(
                    self.db,
                    def,
                    InFile::new(file_id, body_node),
                    Some(offset),
                );
                if let Some(grand_parent) = parent.parent() {
                    if TSKind::call_expression == TSKind::from(&grand_parent) {
                        let method = analyzer.resolve_method(self.db, &node, &parent)?;
                        return Some(DefResolution::Function(method));
                    }
                }
                match analyzer.resolve_attribute(self.db, &node, &parent)? {
                    Attribute::Field(field) => return Some(DefResolution::Field(field)),
                    Attribute::Property(property) => {
                        return Some(DefResolution::Property(property))
                    }
                }
            }
            TSKind::new_expression => {
                let analyzer = SourceAnalyzer::new_for_body(
                    self.db,
                    def,
                    InFile::new(file_id, body_node),
                    Some(offset),
                );
                let constructor = analyzer.resolve_constructor(self.db, &node, &parent)?;
                return Some(DefResolution::Function(constructor));
            }
            TSKind::named_arg => {
                let analyzer = SourceAnalyzer::new_for_body(
                    self.db,
                    def,
                    InFile::new(file_id, body_node),
                    Some(offset),
                );

                if let Some(arg) = analyzer.resolve_named_arg(self.db, &node, &parent) {
                    // Only return if we find an argument. If we don't we were trying to resolve the value.
                    return Some(arg.into());
                }
            }
            _ => {}
        }

        let analyzer = SourceAnalyzer::new_for_body_no_infer(
            self.db,
            def,
            InFile::new(file_id, body_node),
            Some(offset),
        );
        DefResolution::try_from(analyzer.resolver.resolve_ident(text)?)
    }

    fn function_node_to_def(
        &self,
        file_id: FileId,
        container: tree_sitter::Node,
        node: tree_sitter::Node,
        source: Arc<str>,
    ) -> Option<DefResolution> {
        let def_map = self.db.file_def_map(file_id);
        let parent = node.parent()?;

        let parent_name = container
            .child_by_field_name("name")?
            .utf8_text(source.as_ref().as_bytes())
            .ok()?;
        container.child_by_field_name("body")?;
        if let hir_def::FileDefId::FunctionId(id) = def_map.get_first_from_str(parent_name)? {
            self.function_node_to_def_(file_id, container, parent, node, source, id)
        } else {
            None
        }
    }

    pub fn typedef_node_to_def(
        &self,
        file_id: FileId,
        container: tree_sitter::Node,
        node: tree_sitter::Node,
        source: Arc<str>,
    ) -> Option<DefResolution> {
        let def_map = self.db.file_def_map(file_id);

        let parent_name = container
            .child_by_field_name("name")?
            .utf8_text(source.as_ref().as_bytes())
            .ok()?;
        match def_map.get_first_from_str(parent_name)? {
            FileDefId::TypedefId(id) => {
                let def = hir_def::DefWithBodyId::TypedefId(id);
                let text = node.utf8_text(source.as_ref().as_bytes()).ok()?;
                let analyzer = SourceAnalyzer::new_no_body_no_infer(self.db, def, file_id);
                DefResolution::try_from(analyzer.resolver.resolve_ident(text)?)
            }
            _ => None,
        }
    }

    pub fn functag_node_to_def(
        &self,
        file_id: FileId,
        container: tree_sitter::Node,
        node: tree_sitter::Node,
        source: Arc<str>,
    ) -> Option<DefResolution> {
        let def_map = self.db.file_def_map(file_id);

        let parent_name = container
            .child_by_field_name("name")?
            .utf8_text(source.as_ref().as_bytes())
            .ok()?;
        match def_map.get_first_from_str(parent_name)? {
            FileDefId::FunctagId(id) => {
                let def = hir_def::DefWithBodyId::FunctagId(id);
                let text = node.utf8_text(source.as_ref().as_bytes()).ok()?;
                let analyzer = SourceAnalyzer::new_no_body_no_infer(self.db, def, file_id);
                DefResolution::try_from(analyzer.resolver.resolve_ident(text)?)
            }
            _ => None,
        }
    }

    pub fn struct_node_to_def(
        &self,
        file_id: FileId,
        container: tree_sitter::Node,
        node: tree_sitter::Node,
        source: Arc<str>,
    ) -> Option<DefResolution> {
        let resolver = global_resolver(self.db, file_id);
        let struct_name = container
            .child_by_field_name("type")?
            .utf8_text(source.as_bytes())
            .ok()?;
        let ValueNs::StructId(struct_) = resolver.resolve_ident(struct_name)? else {
            return None;
        };
        let struct_: Struct = struct_.value.into();
        let name = node.utf8_text(source.as_bytes()).ok()?;

        DefResolution::StructField(struct_.field(self.db, name)?).into()
    }

    pub fn to_file_def(&self, file_id: FileId) -> File {
        self.imp.file_to_def(file_id)
    }

    pub fn defs_in_scope(&self, file_id: FileId) -> Vec<DefResolution> {
        let resolver = global_resolver(self.db, file_id);
        resolver
            .available_defs()
            .into_iter()
            .flat_map(DefResolution::try_from)
            .collect_vec()
    }

    pub fn defs_in_function_scope(
        &self,
        pos: FilePosition,
        def: FunctionId,
        body_node: tree_sitter::Node,
    ) -> Vec<DefResolution> {
        let analyzer = SourceAnalyzer::new_for_body_no_infer(
            self.db,
            hir_def::DefWithBodyId::FunctionId(def),
            InFile::new(pos.file_id, body_node),
            Some(pos.offset),
        );
        analyzer
            .resolver
            .available_defs()
            .into_iter()
            .flat_map(DefResolution::try_from)
            .collect()
    }

    /// Find references to the definition at the given [`FilePosition`].
    /// Handles both macros and regular definitions.
    ///
    /// # Arguments
    /// * `fpos` - The [`user seen position`](FilePosition) to find references to.
    ///
    /// # Returns
    /// A tuple containing the definition of the macro or regular definition and a list of [`user seen FileRanges`](FileRange).
    pub fn find_references_from_pos(
        &self,
        mut fpos: FilePosition,
    ) -> Option<(DefResolution, Vec<FileRange>)> {
        lazy_static! {
            static ref IDENT_QUERY: tree_sitter::Query = tree_sitter::Query::new(
                &tree_sitter_sourcepawn::language(),
                "(identifier) @identifier"
            )
            .expect("Could not build identifier query.");
        }
        let preprocessing_results = self.preprocess_file(fpos.file_id);
        let source = self.preprocessed_text(fpos.file_id);
        let tree = self.parse(fpos.file_id);
        let root_node = tree.root_node();

        if let Some(macro_refs) = self.find_macro_references(fpos) {
            return Some(macro_refs);
        }
        fpos.offset = preprocessing_results
            .source_map()
            .closest_s_position_always(fpos.offset);

        let node = root_node
            .descendant_for_byte_range(fpos.raw_offset_usize(), fpos.raw_offset_usize())?;
        let src_text = node.utf8_text(source.as_bytes()).ok()?;
        let def = self.find_def(fpos.file_id, &node)?;
        let mut res = Vec::new();
        let file_ids = if let DefResolution::Local(_) = def {
            // Only search in the current file for local definitions
            vec![fpos.file_id]
        } else {
            let graph = self.db.projet_subgraph(fpos.file_id)?;
            graph.nodes.iter().map(|n| n.file_id).collect()
        };
        for file_id in file_ids {
            let file_tree = self.parse(file_id);
            let file_source = self.preprocessed_text(file_id);
            let preprocessing_results = self.preprocess_file(file_id);
            let mut cursor = QueryCursor::new();
            let mut matches =
                cursor.captures(&IDENT_QUERY, file_tree.root_node(), file_source.as_bytes());
            while let Some((match_, _)) = matches.next() {
                for c in match_.captures {
                    let node = c.node;
                    if let Ok(text) = node.utf8_text(file_source.as_bytes()) {
                        if text != src_text {
                            continue;
                        }
                        let file_def = self.find_def(file_id, &node);
                        if file_def == Some(def.clone()) {
                            res.push(FileRange {
                                file_id,
                                range: preprocessing_results
                                    .source_map()
                                    .closest_u_range_always(ts_range_to_text_range(&node.range())),
                            });
                        }
                    }
                }
            }
        }

        Some((def, res))
    }

    /// Find references to a macro at the given [`FilePosition`].
    ///
    /// # Arguments
    /// * `fpos` - The [`user seen position`](FilePosition) to find references to.
    ///
    /// # Returns
    /// A tuple containing the definition of the macro and a list of [`user seen FileRanges`](FileRange).
    fn find_macro_references(&self, fpos: FilePosition) -> Option<(DefResolution, Vec<FileRange>)> {
        let (_, def) = self.find_macro_def(&fpos)?;
        let name = def.name(self.db).map(|it| it.to_smolstr())?;
        let graph = self.db.projet_subgraph(fpos.file_id)?;
        let mut res = Vec::new();
        for graph_node in graph.nodes.iter() {
            let file_id = graph_node.file_id;
            let source = self.db.file_text(file_id);
            let lexer = SourcepawnLexer::new(&source);
            res.extend(lexer.filter_map(|token| {
                if token.token_kind == TokenKind::Identifier && token.text() == name {
                    return FileRange {
                        file_id,
                        range: token.range,
                    }
                    .into();
                }
                None
            }));
        }

        Some((def, res))
    }
}

// FIXME: This is a hacky way to implement the `ToDef` trait...
macro_rules! to_def_methods {
    ($(($def:path, $meth:ident)),* ,) => {$(
        pub fn $meth(&self, src: InFile<NodePtr>) -> Option<$def> {
            self.with_ctx(|ctx| ctx.$meth(src)).map(<$def>::from)
        }
    )*}
}

impl<'db> SemanticsImpl<'db> {
    fn new(db: &'db dyn HirDatabase) -> Self {
        SemanticsImpl {
            db,
            s2d_cache: Default::default(),
        }
    }

    fn with_ctx<F: FnOnce(&mut SourceToDefCtx<'_, '_>) -> T, T>(&self, f: F) -> T {
        let mut cache = self.s2d_cache.borrow_mut();
        let mut ctx = SourceToDefCtx {
            db: self.db,
            cache: &mut cache,
        };
        f(&mut ctx)
    }

    pub fn file_to_def(&self, file_id: FileId) -> File {
        self.with_ctx(|ctx: &mut SourceToDefCtx<'_, '_>| ctx.file_to_def(file_id))
    }

    to_def_methods![
        (crate::FunctionId, fn_to_def),
        (crate::EnumStructId, enum_struct_to_def),
        (hir_def::FieldId, field_to_def),
        (crate::GlobalId, global_to_def),
        (crate::Local, local_to_def),
        (crate::Macro, macro_to_def),
        (crate::Enum, enum_to_def),
        (crate::Variant, variant_to_def),
        (crate::Methodmap, methodmap_to_def),
        (hir_def::PropertyId, property_to_def),
        (crate::Typedef, typedef_to_def),
        (crate::Typeset, typeset_to_def),
        (crate::Functag, functag_to_def),
        (crate::Funcenum, funcenum_to_def),
        (crate::Struct, struct_to_def),
        (crate::StructField, struct_field_to_def),
    ];
}
