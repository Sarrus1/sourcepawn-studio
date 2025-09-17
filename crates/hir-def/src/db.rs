use std::sync::Arc;

use base_db::{
    infer_include_ext, FileExtension, IncludeKind, IncludeType, SourceDatabase, Tree, RE_CHEVRON,
    RE_QUOTE,
};
use fxhash::FxHashMap;
use preprocessor::db::PreprocDatabase;
use smallvec::SmallVec;
use syntax::TSKind;
use vfs::{AnchoredPath, FileId};

use crate::{
    ast_id_map::AstIdMap,
    body::{scope::ExprScopes, Body, BodySourceMap},
    data::{
        EnumData, EnumStructData, FuncenumData, FunctagData, FunctionData, GlobalData, MacroData,
        MethodmapData, PropertyData, StructData, TypedefData, TypesetData, VariantData,
    },
    infer,
    item_tree::{ItemTree, Name},
    BlockId, BlockLoc, DefDiagnostic, DefWithBodyId, EnumId, EnumLoc, EnumStructId, EnumStructLoc,
    FileDefId, FileItem, FuncenumId, FuncenumLoc, FunctagId, FunctagLoc, FunctionId, FunctionLoc,
    GlobalId, GlobalLoc, InferenceResult, Intern, ItemTreeId, Lookup, MacroId, MacroLoc,
    MethodmapId, MethodmapLoc, NodePtr, PropertyId, PropertyLoc, StructId, StructLoc, TreeId,
    TypedefId, TypedefLoc, TypesetId, TypesetLoc, VariantId, VariantLoc,
};

#[salsa::query_group(InternDatabaseStorage)]
pub trait InternDatabase: SourceDatabase {
    // region: items
    #[salsa::interned]
    fn intern_function(&'tree self, loc: FunctionLoc) -> FunctionId;
    #[salsa::interned]
    fn intern_macro(&'tree self, loc: MacroLoc) -> MacroId;
    #[salsa::interned]
    fn intern_enum_struct(&'tree self, loc: EnumStructLoc) -> EnumStructId;
    #[salsa::interned]
    fn intern_methodmap(&'tree self, loc: MethodmapLoc) -> MethodmapId;
    #[salsa::interned]
    fn intern_property(&'tree self, loc: PropertyLoc) -> PropertyId;
    #[salsa::interned]
    fn intern_enum(&'tree self, loc: EnumLoc) -> EnumId;
    #[salsa::interned]
    fn intern_variant(&'tree self, loc: VariantLoc) -> VariantId;
    #[salsa::interned]
    fn intern_typedef(&'tree self, loc: TypedefLoc) -> TypedefId;
    #[salsa::interned]
    fn intern_typeset(&'tree self, loc: TypesetLoc) -> TypesetId;
    #[salsa::interned]
    fn intern_functag(&'tree self, loc: FunctagLoc) -> FunctagId;
    #[salsa::interned]
    fn intern_funcenum(&'tree self, loc: FuncenumLoc) -> FuncenumId;
    #[salsa::interned]
    fn intern_struct(&'tree self, loc: StructLoc) -> StructId;
    #[salsa::interned]
    fn intern_variable(&'tree self, loc: GlobalLoc) -> GlobalId;
    #[salsa::interned]
    fn intern_block(&'tree self, loc: BlockLoc) -> BlockId;
    // endregion: items
}

#[salsa::query_group(DefDatabaseStorage)]
pub trait DefDatabase: InternDatabase + PreprocDatabase {
    /// Parses the file into the syntax tree.
    #[salsa::invoke(parse_query)]
    fn parse(&self, file_id: FileId) -> Tree;

    #[salsa::invoke(ItemTree::file_item_tree_query)]
    fn file_item_tree(&self, file_id: FileId) -> Arc<ItemTree>;

    #[salsa::invoke(ItemTree::block_item_tree_query)]
    fn block_item_tree(&self, block_id: BlockId) -> Arc<ItemTree>;

    #[salsa::invoke(AstIdMap::from_tree)]
    fn ast_id_map(&self, file_id: FileId) -> Arc<AstIdMap>;

    // #[salsa::transparent]
    #[salsa::invoke(DefMap::file_def_map_query)]
    fn file_def_map(&self, file_id: FileId) -> Arc<DefMap>;

    #[salsa::invoke(DefMap::block_def_map_query)]
    fn block_def_map(&self, block_id: BlockId) -> Arc<DefMap>;

    #[salsa::invoke(Body::body_with_source_map_query)]
    fn body_with_source_map(&self, def: DefWithBodyId) -> (Arc<Body>, Arc<BodySourceMap>);

    #[salsa::invoke(Body::body_query)]
    fn body(&self, def: DefWithBodyId) -> Arc<Body>;

    #[salsa::invoke(ExprScopes::expr_scopes_query)]
    fn expr_scopes(&self, def: DefWithBodyId, file_id: FileId) -> Arc<ExprScopes>;

    // region: data
    #[salsa::invoke(FunctionData::function_data_query)]
    fn function_data(&self, id: FunctionId) -> Arc<FunctionData>;

    #[salsa::invoke(MacroData::macro_data_query)]
    fn macro_data(&self, id: MacroId) -> Arc<MacroData>;

    #[salsa::invoke(EnumStructData::enum_struct_data_query)]
    fn enum_struct_data(&self, id: EnumStructId) -> Arc<EnumStructData>;

    #[salsa::invoke(EnumData::enum_data_query)]
    fn enum_data(&self, id: EnumId) -> Arc<EnumData>;

    #[salsa::invoke(VariantData::variant_data_query)]
    fn variant_data(&self, id: VariantId) -> Arc<VariantData>;

    #[salsa::invoke(PropertyData::property_data_query)]
    fn property_data(&self, id: PropertyId) -> Arc<PropertyData>;

    #[salsa::invoke(MethodmapData::methodmap_data_query)]
    fn methodmap_data(&self, id: MethodmapId) -> Arc<MethodmapData>;
    #[salsa::invoke(MethodmapData::methodmap_data_with_diagnostics_query)]
    fn methodmap_data_with_diagnostics(
        &self,
        id: MethodmapId,
    ) -> (Arc<MethodmapData>, Arc<[DefDiagnostic]>);

    #[salsa::invoke(TypedefData::typedef_data_query)]
    fn typedef_data(&self, id: TypedefId) -> Arc<TypedefData>;

    #[salsa::invoke(TypesetData::typeset_data_query)]
    fn typeset_data(&self, id: TypesetId) -> Arc<TypesetData>;

    #[salsa::invoke(FunctagData::functag_data_query)]
    fn functag_data(&self, id: FunctagId) -> Arc<FunctagData>;

    #[salsa::invoke(FuncenumData::funcenum_data_query)]
    fn funcenum_data(&self, id: FuncenumId) -> Arc<FuncenumData>;

    #[salsa::invoke(StructData::struct_data_query)]
    fn struct_data(&self, id: StructId) -> Arc<StructData>;

    #[salsa::invoke(GlobalData::global_data_query)]
    fn global_data(&self, id: GlobalId) -> Arc<GlobalData>;
    // endregion: data

    // region: infer
    #[salsa::invoke(infer::infer_query)]
    fn infer(&self, def: DefWithBodyId) -> Arc<InferenceResult>;
    // endregion: infer
}

fn parse_query(db: &dyn DefDatabase, file_id: FileId) -> Tree {
    tracing::info!("Parsing {}", file_id);
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_sourcepawn::language())
        .expect("Failed to set language");
    let text = db.preprocessed_text(file_id);
    parser
        .parse(text.as_bytes(), None)
        .expect("Failed to parse a file.")
        .into()
}

/// Resolves an include node to a file id and include type and kind.
///
/// # Returns
/// - `None` if the include is invalid (does not have a path).
/// - `Some(None, _, _, _)` if the include is unresolved.
pub fn resolve_include_node(
    db: &dyn DefDatabase,
    file_id: FileId,
    source: &str,
    node: tree_sitter::Node,
) -> Option<(
    Option<FileId>,
    IncludeKind,
    IncludeType,
    String,
    NodePtr,
    FileExtension,
)> {
    let path_node = node.child_by_field_name("path")?;
    let text = path_node.utf8_text(source.as_bytes()).ok()?;
    let type_ = match TSKind::from(&node) {
        TSKind::preproc_include => IncludeType::Include,
        TSKind::preproc_tryinclude => IncludeType::TryInclude,
        _ => unreachable!(),
    };
    let (mut text, kind) = match TSKind::from(path_node) {
        TSKind::system_lib_string => (
            RE_CHEVRON.captures(text)?.get(1)?.as_str().to_string(),
            IncludeKind::Chevrons,
        ),
        TSKind::string_literal => {
            let mut text = RE_QUOTE.captures(text)?.get(1)?.as_str().to_string();
            let extension = infer_include_ext(&mut text);
            // try to resolve path relative to the referencing file.
            if let Some(file_id) = db.resolve_path(AnchoredPath::new(file_id, &text)) {
                return Some((
                    Some(file_id),
                    IncludeKind::Quotes,
                    type_,
                    text.to_string(),
                    NodePtr::from(&path_node),
                    extension,
                ));
            }
            // Hack to detect `include` folders when it's a relative include.
            let text_with_include = format!("include/{}", text);
            if let Some(file_id) = db.resolve_path(AnchoredPath::new(file_id, &text_with_include)) {
                return Some((
                    Some(file_id),
                    IncludeKind::Quotes,
                    type_,
                    text.to_string(),
                    NodePtr::from(&path_node),
                    extension,
                ));
            }
            (text, IncludeKind::Quotes)
        }
        _ => unreachable!(),
    };
    let extension = infer_include_ext(&mut text);

    // Try resolving relative to current file directory first
    if let Some(file_id) = db.resolve_path(AnchoredPath::new(file_id, &text)) {
        return Some((
            Some(file_id),
            kind,
            type_,
            text.to_string(),
            NodePtr::from(&path_node),
            extension,
        ));
    }

    // Try resolving relative to the file's include/ subdirectory for chevrons
    if kind == IncludeKind::Chevrons {
        let text_with_include = format!("include/{}", text);
        if let Some(file_id) = db.resolve_path(AnchoredPath::new(file_id, &text_with_include)) {
            return Some((
                Some(file_id),
                kind,
                type_,
                text.to_string(),
                NodePtr::from(&path_node),
                extension,
            ));
        }
    }

    // Try resolving relative to roots
    if let Some(file_id) = db.resolve_path_relative_to_roots(&text) {
        return Some((
            Some(file_id),
            kind,
            type_,
            text.to_string(),
            NodePtr::from(&path_node),
            extension,
        ));
    }

    // Try resolving include/<path> relative to roots for chevrons
    if kind == IncludeKind::Chevrons {
        let text_with_include = format!("include/{}", text);
        if let Some(file_id) = db.resolve_path_relative_to_roots(&text_with_include) {
            return Some((
                Some(file_id),
                kind,
                type_,
                text.to_string(),
                NodePtr::from(&path_node),
                extension,
            ));
        }
    }

    (
        None,
        kind,
        type_,
        text,
        NodePtr::from(&path_node),
        extension,
    )
        .into()
}

/// For `DefMap`s computed for a block expression, this stores its location in the parent map.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct BlockInfo {
    /// The `BlockId` this `DefMap` was created from.
    block: BlockId,
    /// The containing file.
    parent: FileId,
}

// FIXME: DefMap should not be used as a scope. It should be used to map ids to defs.
// It should be useless to have a DefMap for a block, as they do not define functions etc.
#[derive(Debug, PartialEq, Eq)]
pub struct DefMap {
    file_id: FileId,
    values: FxHashMap<Name, SmallVec<[FileDefId; 1]>>,
    macros: FxHashMap<u32, MacroId>,
    declarations: Vec<FileDefId>,
    /// When this is a block def map, this will hold the block id of the block and module that
    /// contains this block.
    block: Option<BlockInfo>,
}

impl DefMap {
    pub fn new(file_id: FileId) -> Self {
        Self {
            file_id,
            values: FxHashMap::default(),
            macros: FxHashMap::default(),
            declarations: Default::default(),
            block: Default::default(),
        }
    }

    pub fn file_id(&self) -> FileId {
        self.file_id
    }

    pub fn file_def_map_query(db: &dyn DefDatabase, file_id: FileId) -> Arc<Self> {
        let mut res = DefMap::new(file_id);
        let item_tree = db.file_item_tree(file_id);
        let tree_id = TreeId::new(file_id, None);
        let mut macro_idx = 0u32;
        for item in item_tree.top_level_items() {
            match item {
                FileItem::Function(id) => {
                    let func = &item_tree[*id];
                    let fn_id = FunctionLoc {
                        container: file_id.into(),
                        id: ItemTreeId {
                            tree: tree_id,
                            value: *id,
                        },
                    }
                    .intern(db);
                    res.declare(func.name.clone(), FileDefId::FunctionId(fn_id));
                }
                FileItem::Variable(id) => {
                    let var = &item_tree[*id];
                    let var_id = GlobalLoc {
                        tree: TreeId::new(file_id, None),
                        value: *id,
                    }
                    .intern(db);
                    res.declare(var.name.clone(), FileDefId::GlobalId(var_id));
                }
                FileItem::EnumStruct(id) => {
                    let enum_struct = &item_tree[*id];
                    let enum_struct_id = EnumStructLoc {
                        container: file_id.into(),
                        id: ItemTreeId {
                            tree: tree_id,
                            value: *id,
                        },
                    }
                    .intern(db);
                    res.declare(
                        enum_struct.name.clone(),
                        FileDefId::EnumStructId(enum_struct_id),
                    );
                }
                FileItem::Methodmap(id) => {
                    let methodmap = &item_tree[*id];
                    let methodmap_id = MethodmapLoc {
                        container: file_id.into(),
                        id: ItemTreeId {
                            tree: tree_id,
                            value: *id,
                        },
                    }
                    .intern(db);
                    res.declare(methodmap.name.clone(), FileDefId::MethodmapId(methodmap_id));
                }
                FileItem::Macro(id) => {
                    let macro_ = &item_tree[*id];
                    let macro_id = MacroLoc {
                        container: file_id.into(),
                        id: ItemTreeId {
                            tree: tree_id,
                            value: *id,
                        },
                    }
                    .intern(db);
                    res.declare(macro_.name.clone(), FileDefId::MacroId(macro_id));
                    res.macros.insert(macro_idx, macro_id);
                    macro_idx += 1;
                }
                FileItem::Enum(id) => {
                    let enum_ = &item_tree[*id];
                    let enum_id = EnumLoc {
                        container: file_id.into(),
                        id: ItemTreeId {
                            tree: tree_id,
                            value: *id,
                        },
                    }
                    .intern(db);
                    for variant_idx in enum_.variants.clone() {
                        let variant = &item_tree[variant_idx];
                        let variant_id = VariantLoc {
                            container: enum_id.into(),
                            id: ItemTreeId {
                                tree: tree_id,
                                value: variant_idx,
                            },
                        }
                        .intern(db);
                        res.declare(variant.name.clone(), FileDefId::VariantId(variant_id));
                    }
                    res.declare(enum_.name.clone(), FileDefId::EnumId(enum_id));
                }
                FileItem::Variant(_) => (),
                FileItem::Typedef(id) => {
                    let typedef = &item_tree[*id];
                    let typedef_id = TypedefLoc {
                        container: file_id.into(),
                        id: ItemTreeId {
                            tree: tree_id,
                            value: *id,
                        },
                    }
                    .intern(db);
                    if let Some(name) = typedef.name.clone() {
                        res.declare(name, FileDefId::TypedefId(typedef_id))
                    }
                }
                FileItem::Typeset(id) => {
                    let typeset = &item_tree[*id];
                    let typeset_id = TypesetLoc {
                        container: file_id.into(),
                        id: ItemTreeId {
                            tree: tree_id,
                            value: *id,
                        },
                    }
                    .intern(db);
                    res.declare(typeset.name.clone(), FileDefId::TypesetId(typeset_id));
                }
                FileItem::Functag(id) => {
                    let functag = &item_tree[*id];
                    let functag_id = FunctagLoc {
                        container: file_id.into(),
                        id: ItemTreeId {
                            tree: tree_id,
                            value: *id,
                        },
                    }
                    .intern(db);
                    if let Some(name) = functag.name.clone() {
                        res.declare(name, FileDefId::FunctagId(functag_id))
                    }
                }
                FileItem::Funcenum(id) => {
                    let funcenum = &item_tree[*id];
                    let funcenum_id = FuncenumLoc {
                        container: file_id.into(),
                        id: ItemTreeId {
                            tree: tree_id,
                            value: *id,
                        },
                    }
                    .intern(db);
                    res.declare(funcenum.name.clone(), FileDefId::FuncenumId(funcenum_id));
                }
                FileItem::Struct(id) => {
                    let struct_ = &item_tree[*id];
                    let struct_id = StructLoc {
                        container: file_id.into(),
                        id: ItemTreeId {
                            tree: tree_id,
                            value: *id,
                        },
                    }
                    .intern(db);
                    res.declare(struct_.name.clone(), FileDefId::StructId(struct_id));
                }
                FileItem::Property(_) => (),
            }
        }

        Arc::new(res)
    }

    pub fn get(&self, name: &Name) -> Option<SmallVec<[FileDefId; 1]>> {
        self.values.get(name).cloned()
    }

    pub fn get_from_str(&self, name: &str) -> Option<SmallVec<[FileDefId; 1]>> {
        self.get(&Name::from(name))
    }

    pub fn get_first_from_str(&self, name: &str) -> Option<FileDefId> {
        self.get_from_str(name).and_then(|v| v.first().copied())
    }

    pub fn get_macro(&self, idx: &u32) -> Option<MacroId> {
        self.macros.get(idx).copied()
    }

    pub(crate) fn block_def_map_query(db: &dyn DefDatabase, block_id: BlockId) -> Arc<DefMap> {
        let item_tree = db.block_item_tree(block_id);
        let file_id = block_id.lookup(db).file_id;
        let mut res = DefMap::new(file_id);
        for item in item_tree.top_level_items() {
            match item {
                FileItem::Variable(id) => {
                    let var = &item_tree[*id];
                    let var_id = GlobalLoc {
                        tree: TreeId::new(file_id, Some(block_id)),
                        value: *id,
                    }
                    .intern(db);
                    res.declare(var.name.clone(), FileDefId::GlobalId(var_id));
                }
                _ => unreachable!("Only variables can be defined in a block"),
            }
        }

        Arc::new(res)
    }

    fn declare(&mut self, name: Name, def: FileDefId) {
        self.values.entry(name).or_default().push(def);
        self.declarations.push(def);
    }

    #[allow(unused)]
    pub(crate) fn block_id(&self) -> Option<BlockId> {
        self.block.map(|block| block.block)
    }

    pub fn declarations(&self) -> &[FileDefId] {
        &self.declarations
    }
}
