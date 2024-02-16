use std::sync::Arc;

use base_db::{
    infer_include_ext, FileExtension, IncludeKind, IncludeType, SourceDatabase, RE_CHEVRON,
    RE_QUOTE,
};
use fxhash::FxHashMap;
use syntax::TSKind;
use vfs::{AnchoredPath, FileId};

use crate::{
    ast_id_map::AstIdMap,
    body::{scope::ExprScopes, Body, BodySourceMap},
    data::{EnumStructData, FunctionData, MacroData},
    infer,
    item_tree::{ItemTree, Name},
    BlockId, BlockLoc, DefWithBodyId, EnumStructId, EnumStructLoc, FileDefId, FileItem, FunctionId,
    FunctionLoc, GlobalId, GlobalLoc, InferenceResult, Intern, ItemTreeId, Lookup, MacroId,
    MacroLoc, NodePtr, TreeId,
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
    fn intern_variable(&'tree self, loc: GlobalLoc) -> GlobalId;
    #[salsa::interned]
    fn intern_block(&'tree self, loc: BlockLoc) -> BlockId;
    // endregion: items
}

#[salsa::query_group(DefDatabaseStorage)]
pub trait DefDatabase: InternDatabase {
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
    // endregion: data

    // region: infer
    #[salsa::invoke(infer::infer_query)]
    fn infer(&self, def: DefWithBodyId) -> Arc<InferenceResult>;
    // endregion: infer
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
            (text, IncludeKind::Quotes)
        }
        _ => unreachable!(),
    };
    let extension = infer_include_ext(&mut text);

    (
        db.resolve_path_relative_to_roots(&text),
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
#[derive(Debug, Default, PartialEq, Eq)]
pub struct DefMap {
    values: FxHashMap<Name, FileDefId>,
    macros: FxHashMap<u32, MacroId>,
    declarations: Vec<FileDefId>,
    /// When this is a block def map, this will hold the block id of the block and module that
    /// contains this block.
    block: Option<BlockInfo>,
}

impl DefMap {
    pub fn file_def_map_query(db: &dyn DefDatabase, file_id: FileId) -> Arc<Self> {
        let mut res = DefMap::default();
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
            }
        }

        Arc::new(res)
    }

    pub fn get(&self, name: &Name) -> Option<FileDefId> {
        self.values.get(name).copied()
    }

    pub fn get_from_str(&self, name: &str) -> Option<FileDefId> {
        self.get(&Name::from(name))
    }

    pub fn get_macro(&self, idx: &u32) -> Option<MacroId> {
        self.macros.get(idx).copied()
    }

    pub(crate) fn block_def_map_query(db: &dyn DefDatabase, block_id: BlockId) -> Arc<DefMap> {
        let item_tree = db.block_item_tree(block_id);
        let file_id = block_id.lookup(db).file_id;
        let mut res = DefMap::default();
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
        self.values.insert(name, def);
        self.declarations.push(def);
    }

    pub(crate) fn block_id(&self) -> Option<BlockId> {
        self.block.map(|block| block.block)
    }

    pub fn declarations(&self) -> &[FileDefId] {
        &self.declarations
    }
}
