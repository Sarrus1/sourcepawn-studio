use std::sync::Arc;

use fxhash::FxHashMap;
use la_arena::{Arena, ArenaMap, Idx};
use smol_str::ToSmolStr;
use syntax::TSKind;

use crate::{
    hir::type_ref::TypeRef,
    item_tree::{EnumStructItemId, MethodmapItemId, Name, SpecialMethod},
    resolver::{global_resolver, ValueNs},
    src::{HasChildSource, HasSource},
    DefDatabase, DefDiagnostic, EnumStructId, FuncenumId, FunctagId, FunctagLoc, FunctionId,
    FunctionLoc, InFile, Intern, ItemTreeId, LocalFieldId, Lookup, MacroId, MethodmapId, NodePtr,
    PropertyId, PropertyLoc, TypedefId, TypedefLoc, TypesetId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionData {
    pub name: Name,
    pub type_ref: Option<TypeRef>,
}

impl FunctionData {
    pub(crate) fn function_data_query(db: &dyn DefDatabase, id: FunctionId) -> Arc<FunctionData> {
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let function = &item_tree[loc.value];
        let function_data = FunctionData {
            name: function.name.clone(),
            type_ref: function.ret_type.clone(),
        };

        Arc::new(function_data)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroData {
    pub name: Name,
}

impl MacroData {
    pub(crate) fn macro_data_query(db: &dyn DefDatabase, id: MacroId) -> Arc<MacroData> {
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let macro_ = &item_tree[loc.value];
        let macro_data = MacroData {
            name: macro_.name.clone(),
        };

        Arc::new(macro_data)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodmapData {
    pub name: Name,
    pub items: Arc<Arena<MethodmapItemData>>,
    pub items_map: Arc<FxHashMap<Name, Idx<MethodmapItemData>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodmapItemData {
    Property(PropertyData),
    Method(FunctionId),
    Constructor(FunctionId),
    Destructor(FunctionId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyItem {
    Getter(FunctionId),
    Setter(FunctionId),
}

impl PropertyItem {
    pub fn function_id(&self) -> FunctionId {
        match self {
            PropertyItem::Getter(id) | PropertyItem::Setter(id) => *id,
        }
    }
}

/// A single property of a methodmap
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyData {
    pub id: PropertyId,
    pub getters_setters: Vec<PropertyItem>,
}

impl MethodmapData {
    pub(crate) fn methodmap_data_query(
        db: &dyn DefDatabase,
        id: MethodmapId,
    ) -> Arc<MethodmapData> {
        Self::methodmap_data_with_diagnostics_query(db, id).0
    }

    pub(crate) fn methodmap_data_with_diagnostics_query(
        db: &dyn DefDatabase,
        id: MethodmapId,
    ) -> (Arc<MethodmapData>, Arc<[DefDiagnostic]>) {
        let mut diags = Vec::new();
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let methodmap = &item_tree[loc.value];
        let mut items = Arena::new();
        let mut items_map = FxHashMap::default();
        // Compute inherits first, so that they get properly overwritten.
        if let Some(inherits_name) = methodmap.inherits.clone() {
            let resolver = global_resolver(db, loc.file_id());
            if let Some(inherits) = resolver.resolve_ident(inherits_name.to_string().as_str()) {
                if let ValueNs::MethodmapId(inherits) = inherits {
                    let inherits_data = db.methodmap_data(inherits.value);
                    for ((name, _), (_, item)) in inherits_data
                        .items_map
                        .iter()
                        .zip(inherits_data.items.iter())
                    {
                        items_map.insert(name.clone(), items.alloc(item.clone()));
                    }
                } else {
                    diags.push(DefDiagnostic::UnresolvedInherit {
                        inherit_name: inherits_name,
                        methodmap_ast_id: methodmap.ast_id,
                        exists: true,
                    });
                };
            } else {
                diags.push(DefDiagnostic::UnresolvedInherit {
                    inherit_name: inherits_name,
                    methodmap_ast_id: methodmap.ast_id,
                    exists: false,
                });
            }
        }
        methodmap.items.iter().for_each(|e| match *e {
            MethodmapItemId::Property(property_idx) => {
                let property = &item_tree[property_idx];
                let property_id = PropertyLoc {
                    container: id.into(),
                    id: ItemTreeId {
                        tree: loc.tree_id(),
                        value: property_idx,
                    },
                }
                .intern(db);
                let property_data = MethodmapItemData::Property(PropertyData {
                    id: property_id,
                    getters_setters: property
                        .getters_setters
                        .clone()
                        .map(|fn_id| {
                            let id = FunctionLoc {
                                container: id.into(),
                                id: ItemTreeId {
                                    tree: loc.tree_id(),
                                    value: fn_id,
                                },
                            }
                            .intern(db);
                            let data = db.function_data(id);
                            match data.name.to_smolstr().as_str() {
                                "get" => PropertyItem::Getter(id),
                                "set" => PropertyItem::Setter(id),
                                _ => unreachable!("Invalid getter/setter function"),
                            }
                        })
                        .collect(),
                });
                let property_id = items.alloc(property_data);
                items_map.insert(property.name.clone(), property_id);
            }
            MethodmapItemId::Method(method_idx) => {
                let method = &item_tree[method_idx];
                let fn_id = FunctionLoc {
                    container: id.into(),
                    id: ItemTreeId {
                        tree: loc.tree_id(),
                        value: method_idx,
                    },
                }
                .intern(db);
                let method_ = match method.special {
                    Some(SpecialMethod::Constructor) => MethodmapItemData::Constructor(fn_id),
                    Some(SpecialMethod::Destructor) => MethodmapItemData::Destructor(fn_id),
                    None => MethodmapItemData::Method(fn_id),
                };
                let method_id = items.alloc(method_);
                // FIXME: Not sure if we should intern like this...
                items_map.insert(method.name.clone(), method_id);
            } // TODO: Add diagnostic for duplicate methodmap items
        });
        let methodmap_data = MethodmapData {
            name: methodmap.name.clone(),
            items: Arc::new(items),
            items_map: Arc::new(items_map),
        };

        (Arc::new(methodmap_data), diags.into())
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn constructor(&self) -> Option<FunctionId> {
        self.items.iter().find_map(|(_, item)| match item {
            MethodmapItemData::Constructor(id) => Some(*id),
            MethodmapItemData::Method(_)
            | MethodmapItemData::Property(_)
            | MethodmapItemData::Destructor(_) => None,
        })
    }

    pub fn item(&self, item: Idx<MethodmapItemData>) -> &MethodmapItemData {
        &self.items[item]
    }

    pub fn method(&self, item: Idx<MethodmapItemData>) -> Option<&FunctionId> {
        match &self.items[item] {
            MethodmapItemData::Property(_) => None,
            MethodmapItemData::Method(function_id) => Some(function_id),
            MethodmapItemData::Constructor(function_id) => Some(function_id),
            MethodmapItemData::Destructor(function_id) => Some(function_id),
        }
    }

    pub fn property(&self, item: Idx<MethodmapItemData>) -> Option<&PropertyData> {
        match &self.items[item] {
            MethodmapItemData::Property(property_data) => Some(property_data),
            MethodmapItemData::Method(_)
            | MethodmapItemData::Constructor(_)
            | MethodmapItemData::Destructor(_) => None,
        }
    }

    pub fn items(&self, name: &Name) -> Option<Idx<MethodmapItemData>> {
        self.items_map.get(name).cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedefData {
    pub name: Option<Name>,
    pub type_ref: TypeRef,
}

impl TypedefData {
    pub(crate) fn typedef_data_query(db: &dyn DefDatabase, id: TypedefId) -> Arc<TypedefData> {
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let typedef = &item_tree[loc.value];
        let typedef_data = TypedefData {
            name: typedef.name.clone(),
            type_ref: typedef.type_ref.clone(),
        };

        Arc::new(typedef_data)
    }

    pub fn name(&self) -> Option<Name> {
        self.name.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypesetData {
    pub name: Name,
    pub typedefs: Arc<Arena<TypedefId>>,
}

impl TypesetData {
    pub(crate) fn typeset_data_query(db: &dyn DefDatabase, id: TypesetId) -> Arc<TypesetData> {
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let typeset = &item_tree[loc.value];
        let mut typedefs = Arena::new();
        typeset.typedefs.clone().for_each(|typedef_idx| {
            let typedef_id = TypedefLoc {
                container: id.into(),
                id: ItemTreeId {
                    tree: loc.tree_id(),
                    value: typedef_idx,
                },
            }
            .intern(db);
            let _ = typedefs.alloc(typedef_id);
        });
        let typeset_data = TypesetData {
            name: typeset.name.clone(),
            typedefs: typedefs.into(),
        };

        Arc::new(typeset_data)
    }

    pub fn name(&self) -> Name {
        self.name.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctagData {
    pub name: Option<Name>,
    pub type_ref: Option<TypeRef>,
}

impl FunctagData {
    pub(crate) fn functag_data_query(db: &dyn DefDatabase, id: FunctagId) -> Arc<Self> {
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let functag = &item_tree[loc.value];
        let functag_data = FunctagData {
            name: functag.name.clone(),
            type_ref: functag.type_ref.clone(),
        };

        Arc::new(functag_data)
    }

    pub fn name(&self) -> Option<Name> {
        self.name.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncenumData {
    pub name: Name,
    pub functags: Arc<Arena<FunctagId>>,
}

impl FuncenumData {
    pub(crate) fn funcenum_data_query(db: &dyn DefDatabase, id: FuncenumId) -> Arc<Self> {
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let funcenum = &item_tree[loc.value];
        let mut functags = Arena::new();
        funcenum.functags.clone().for_each(|functag_idx| {
            let functag_id = FunctagLoc {
                container: id.into(),
                id: ItemTreeId {
                    tree: loc.tree_id(),
                    value: functag_idx,
                },
            }
            .intern(db);
            let _ = functags.alloc(functag_id);
        });
        let functag_data = Self {
            name: funcenum.name.clone(),
            functags: functags.into(),
        };

        Arc::new(functag_data)
    }

    pub fn name(&self) -> Name {
        self.name.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumStructData {
    pub name: Name,
    pub items: Arc<Arena<EnumStructItemData>>,
    pub items_map: Arc<FxHashMap<Name, Idx<EnumStructItemData>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumStructItemData {
    Field(FieldData),
    Method(FunctionId),
}

/// A single field of a struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldData {
    pub name: Name,
    pub type_ref: TypeRef,
}

impl EnumStructData {
    pub(crate) fn enum_struct_data_query(
        db: &dyn DefDatabase,
        id: EnumStructId,
    ) -> Arc<EnumStructData> {
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let enum_struct = &item_tree[loc.value];
        let mut items = Arena::new();
        let mut items_map = FxHashMap::default();
        enum_struct.items.iter().for_each(|e| match e {
            EnumStructItemId::Field(field_idx) => {
                let field = &item_tree[*field_idx];
                let field_data = EnumStructItemData::Field(FieldData {
                    name: field.name.clone(),
                    type_ref: field.type_ref.clone(),
                });
                let field_id = items.alloc(field_data);
                items_map.insert(field.name.clone(), field_id);
            }
            EnumStructItemId::Method(method_idx) => {
                let method = &item_tree[*method_idx];
                let fn_id = FunctionLoc {
                    container: id.into(),
                    id: ItemTreeId {
                        tree: loc.tree_id(),
                        value: *method_idx,
                    },
                }
                .intern(db);
                let method_id = items.alloc(EnumStructItemData::Method(fn_id));
                // FIXME: Not sure if we should intern like this...
                items_map.insert(method.name.clone(), method_id);
            } // TODO: Add diagnostic for duplicate enum struct items
        });
        let enum_struct_data = EnumStructData {
            name: enum_struct.name.clone(),
            items: Arc::new(items),
            items_map: Arc::new(items_map),
        };

        Arc::new(enum_struct_data)
    }

    pub fn item(&self, item: Idx<EnumStructItemData>) -> &EnumStructItemData {
        &self.items[item]
    }

    pub fn method(&self, item: Idx<EnumStructItemData>) -> Option<&FunctionId> {
        match &self.items[item] {
            EnumStructItemData::Field(_) => None,
            EnumStructItemData::Method(function_id) => Some(function_id),
        }
    }

    pub fn items(&self, name: &Name) -> Option<Idx<EnumStructItemData>> {
        self.items_map.get(name).cloned()
    }

    pub fn field_type(&self, field: Idx<EnumStructItemData>) -> Option<&TypeRef> {
        match &self.items[field] {
            EnumStructItemData::Field(field_data) => Some(&field_data.type_ref),
            EnumStructItemData::Method(_) => None,
        }
    }
}

impl HasChildSource<LocalFieldId> for EnumStructId {
    type Value = NodePtr;

    fn child_source(&self, db: &dyn DefDatabase) -> InFile<ArenaMap<LocalFieldId, Self::Value>> {
        let loc = self.lookup(db).id;
        let mut map = ArenaMap::default();
        let tree = db.parse(loc.file_id());
        // We use fields to get the Idx of the field, even if they are dropped at the end of the call.
        // The Idx will be the same when we rebuild the EnumStructData.
        // It feels like we could just be inserting empty data...
        // Why can't we just treat fields as item_tree members instead of making them local to the enum_struct?
        // TODO: Is there a better way to do this?
        // FIXME: Why does it feel like we are doing this twice?
        let mut items = Arena::new();
        let enum_struct_node = loc.source(db, &tree).value;
        for child in enum_struct_node
            .children(&mut enum_struct_node.walk())
            .filter(|c| TSKind::from(c) == TSKind::enum_struct_field)
        {
            let name_node = child.child_by_field_name("name").unwrap();
            let name = Name::from_node(&name_node, &db.preprocessed_text(loc.file_id()));
            let type_ref_node = child.child_by_field_name("type").unwrap();
            let type_ref = TypeRef::from_node(&type_ref_node, &db.preprocessed_text(loc.file_id()));
            let field = EnumStructItemData::Field(FieldData { name, type_ref });
            map.insert(items.alloc(field), NodePtr::from(&child));
        }
        InFile::new(loc.file_id(), map)
    }
}

impl HasChildSource<Idx<TypedefData>> for TypesetId {
    type Value = NodePtr;

    fn child_source(
        &self,
        db: &dyn DefDatabase,
    ) -> InFile<ArenaMap<Idx<TypedefData>, Self::Value>> {
        let loc = self.lookup(db).id;
        let mut map = ArenaMap::default();
        let tree = db.parse(loc.file_id());
        let mut typedefs: Arena<TypedefData> = Arena::new();
        let typeset_node = loc.source(db, &tree).value;
        for child in typeset_node
            .children(&mut typeset_node.walk())
            .filter(|c| TSKind::from(c) == TSKind::typedef_expression)
        {
            if let Some(type_ref_node) = child.child_by_field_name("returnType") {
                let type_ref =
                    TypeRef::from_node(&type_ref_node, &db.preprocessed_text(loc.file_id()));
                let typedef = TypedefData {
                    name: None,
                    type_ref,
                };
                map.insert(typedefs.alloc(typedef), NodePtr::from(&child));
            }
        }
        InFile::new(loc.file_id(), map)
    }
}

impl HasChildSource<Idx<FunctagData>> for FuncenumId {
    type Value = NodePtr;

    fn child_source(
        &self,
        db: &dyn DefDatabase,
    ) -> InFile<ArenaMap<Idx<FunctagData>, Self::Value>> {
        let loc = self.lookup(db).id;
        let mut map = ArenaMap::default();
        let tree = db.parse(loc.file_id());
        let mut functags: Arena<FunctagData> = Arena::new();
        let typeset_node = loc.source(db, &tree).value;
        for child in typeset_node
            .children(&mut typeset_node.walk())
            .filter(|c| TSKind::from(c) == TSKind::typedef_expression)
        {
            let type_ref = if let Some(type_ref_node) = child.child_by_field_name("returnType") {
                TypeRef::from_node(&type_ref_node, &db.preprocessed_text(loc.file_id())).into()
            } else {
                None
            };
            let functag = FunctagData {
                name: None,
                type_ref,
            };
            map.insert(functags.alloc(functag), NodePtr::from(&child));
        }
        InFile::new(loc.file_id(), map)
    }
}
