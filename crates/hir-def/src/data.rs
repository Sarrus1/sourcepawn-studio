use std::{fmt, sync::Arc};

use fxhash::FxHashMap;
use itertools::Itertools;
use la_arena::{Arena, ArenaMap, Idx};
use smol_str::ToSmolStr;
use syntax::TSKind;

use crate::{
    hir::type_ref::TypeRef,
    item_tree::{
        EnumStructItemId, FunctionKind, MethodmapItemId, Name, Param, RawVisibilityId,
        SpecialMethod,
    },
    resolver::{global_resolver, ValueNs},
    src::{HasChildSource, HasSource},
    DefDatabase, DefDiagnostic, EnumId, EnumStructId, FuncenumId, FunctagId, FunctagLoc,
    FunctionId, FunctionLoc, GlobalId, InFile, Intern, ItemContainerId, ItemTreeId, LocalFieldId,
    LocalStructFieldId, Lookup, MacroId, MethodmapId, NodePtr, PropertyId, PropertyLoc, StructId,
    TypedefId, TypedefLoc, TypesetId, VariantId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamData {
    pub type_ref: Option<TypeRef>,
    pub has_default: bool,
    pub is_rest: bool,
    pub is_const: bool,
}

impl From<&Param> for ParamData {
    fn from(param: &Param) -> Self {
        ParamData {
            type_ref: param.type_ref.clone(),
            has_default: param.has_default,
            is_rest: param.is_rest,
            is_const: param.is_const,
        }
    }
}

impl fmt::Display for ParamData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        if self.is_const {
            s.push_str("const ");
        }
        if self.is_rest {
            s.push_str("...");
        }
        if let Some(type_ref) = &self.type_ref {
            s.push_str(&type_ref.to_string());
        }
        if self.has_default {
            // TODO: Show the actual default value
            s.push_str(" = ...");
        }

        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionData {
    pub name: Name,
    pub type_ref: Option<TypeRef>,
    params: Vec<ParamData>,
    pub kind: FunctionKind,
    pub visibility: RawVisibilityId,
    pub special: Option<SpecialMethod>,
    pub deprecated: bool,
}

impl FunctionData {
    pub(crate) fn function_data_query(db: &dyn DefDatabase, id: FunctionId) -> Arc<FunctionData> {
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let function = &item_tree[loc.value];
        let params = function
            .params
            .clone()
            .map(|param_idx| ParamData::from(&item_tree[param_idx]))
            .collect_vec();

        let function_data = FunctionData {
            name: function.name.clone(),
            type_ref: function.ret_type.clone(),
            params,
            kind: function.kind,
            visibility: function.visibility,
            special: function.special,
            deprecated: function.deprecated,
        };

        Arc::new(function_data)
    }

    pub fn name(&self) -> Name {
        self.name.clone()
    }

    pub fn type_ref(&self) -> Option<TypeRef> {
        self.type_ref.clone()
    }

    pub fn params(&self) -> &[ParamData] {
        &self.params
    }

    pub fn number_of_mandatory_parameters(&self) -> usize {
        self.params
            .iter()
            .filter(|p| !(p.has_default || p.is_rest))
            .count()
    }

    pub fn number_of_parameters(&self) -> Option<usize> {
        if self.params.iter().any(|p| p.is_rest) {
            None
        } else {
            Some(self.params.len())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroData {
    pub name: Name,
    pub deprecated: bool,
}

impl MacroData {
    pub(crate) fn macro_data_query(db: &dyn DefDatabase, id: MacroId) -> Arc<MacroData> {
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let macro_ = &item_tree[loc.value];
        let macro_data = MacroData {
            name: macro_.name.clone(),
            deprecated: macro_.deprecated,
        };

        Arc::new(macro_data)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodmapData {
    pub name: Name,
    pub items: Arc<Arena<MethodmapItemData>>,
    pub items_map: Arc<FxHashMap<Name, Idx<MethodmapItemData>>>,
    pub last_inherited_item_idx: Option<u32>,
    pub extension: Option<MethodmapExtension>,
    pub inherits: Option<MethodmapId>,
    pub constructor: Option<Idx<MethodmapItemData>>,
    pub destructor: Option<Idx<MethodmapItemData>>,
    pub deprecated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodmapExtension {
    Inherits(Name),
    Nullable,
}

impl MethodmapExtension {
    pub fn from(inherits: Option<Name>, nullable: bool) -> Option<Self> {
        match (inherits, nullable) {
            (Some(name), false) => Some(MethodmapExtension::Inherits(name)),
            (None, true) => Some(MethodmapExtension::Nullable),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodmapItemData {
    Property(PropertyData),
    Method(FunctionId),
    Static(FunctionId),
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
    pub name: Name,
    pub type_ref: TypeRef,
    pub deprecated: bool,
}

impl PropertyData {
    pub(crate) fn property_data_query(db: &dyn DefDatabase, id: PropertyId) -> Arc<PropertyData> {
        let ItemContainerId::MethodmapId(methodmap_id) = id.lookup(db).container else {
            panic!("expected a methodmap id, got {:?}", id);
        };

        let methodmap_data = db.methodmap_data(methodmap_id);
        methodmap_data
            .property_from_id(id)
            .expect("expected a property given an id")
            .to_owned()
            .into()
    }
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
        let mut inherits_id = None;
        let mut constructor = None;
        let mut destructor = None;
        let mut last_inherited_item_idx = None;
        if let Some(inherits_name) = methodmap.inherits.clone() {
            let resolver = global_resolver(db, loc.file_id());
            if let Some(inherits) = resolver.resolve_ident(inherits_name.to_string().as_str()) {
                if let ValueNs::MethodmapId(inherits) = inherits {
                    inherits_id = Some(inherits.value);
                    let inherits_data = db.methodmap_data(inherits.value);
                    items_map.extend(
                        inherits_data
                            .items_map
                            .iter()
                            .filter(|(_, v)| {
                                // The constructors and destructors are not inherited
                                !matches!(
                                    inherits_data.item(**v),
                                    MethodmapItemData::Constructor(_)
                                        | MethodmapItemData::Destructor(_)
                                )
                            })
                            .map(|(k, v)| (k.clone(), items.alloc(inherits_data.item(*v).clone()))),
                    );
                    last_inherited_item_idx = items_map
                        .values()
                        .max()
                        .map(|max| max.into_raw().into_u32());
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
                    name: property.name.clone(),
                    type_ref: property.type_ref.clone(),
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
                    deprecated: property.deprecated,
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
                    None if method.visibility.contains(RawVisibilityId::STATIC) => {
                        MethodmapItemData::Static(fn_id)
                    }
                    None => MethodmapItemData::Method(fn_id),
                };
                let method_id = items.alloc(method_);
                match method.special {
                    Some(SpecialMethod::Constructor) => constructor = method_id.into(),
                    Some(SpecialMethod::Destructor) => destructor = method_id.into(),
                    _ => (),
                }
                // FIXME: Not sure if we should intern like this...
                items_map.insert(method.name.clone(), method_id);
            } // TODO: Add diagnostic for duplicate methodmap items
        });
        let methodmap_data = MethodmapData {
            name: methodmap.name.clone(),
            items: Arc::new(items),
            items_map: Arc::new(items_map),
            last_inherited_item_idx,
            extension: MethodmapExtension::from(methodmap.inherits.clone(), methodmap.nullable),
            inherits: inherits_id,
            constructor,
            destructor,
            deprecated: methodmap.deprecated,
        };

        (Arc::new(methodmap_data), diags.into())
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn constructor(&self) -> Option<&FunctionId> {
        self.method(self.constructor?)
    }

    pub fn destructor(&self) -> Option<&FunctionId> {
        self.method(self.destructor?)
    }

    pub fn item(&self, item: Idx<MethodmapItemData>) -> &MethodmapItemData {
        &self.items[item]
    }

    pub fn method(&self, item: Idx<MethodmapItemData>) -> Option<&FunctionId> {
        match &self.items[item] {
            MethodmapItemData::Property(_) => None,
            MethodmapItemData::Static(function_id) => Some(function_id),
            MethodmapItemData::Method(function_id) => Some(function_id),
            MethodmapItemData::Constructor(function_id) => Some(function_id),
            MethodmapItemData::Destructor(function_id) => Some(function_id),
        }
    }

    pub fn property(&self, item: Idx<MethodmapItemData>) -> Option<&PropertyData> {
        match &self.items[item] {
            MethodmapItemData::Property(property_data) => Some(property_data),
            MethodmapItemData::Method(_)
            | MethodmapItemData::Static(_)
            | MethodmapItemData::Constructor(_)
            | MethodmapItemData::Destructor(_) => None,
        }
    }

    pub fn property_from_id(&self, id: PropertyId) -> Option<&PropertyData> {
        // FIXME: O(n)
        for idx in self.items_map.values() {
            if let MethodmapItemData::Property(property_data) = &self.items[*idx] {
                if property_data.id == id {
                    return Some(property_data);
                }
            }
        }

        None
    }

    pub fn items(&self, name: &Name) -> Option<Idx<MethodmapItemData>> {
        self.items_map.get(name).cloned()
    }

    pub fn static_methods(&self) -> impl Iterator<Item = FunctionId> + '_ {
        self.items.iter().filter_map(|(_, item)| match item {
            MethodmapItemData::Static(id) => Some(*id),
            _ => None,
        })
    }

    fn is_local(&self, idx: Idx<MethodmapItemData>) -> bool {
        match self.last_inherited_item_idx {
            Some(max_idx) => idx.into_raw().into_u32() > max_idx,
            None => true,
        }
    }

    /// Items that were not inherited.
    pub fn local_items(
        &self,
    ) -> impl Iterator<Item = (Idx<MethodmapItemData>, &MethodmapItemData)> + '_ {
        self.items
            .iter()
            .filter(move |(idx, _)| self.is_local(*idx))
    }

    pub fn methods(&self) -> impl Iterator<Item = FunctionId> + '_ {
        self.items.iter().filter_map(|(_, item)| match item {
            MethodmapItemData::Method(id)
            | MethodmapItemData::Static(id)
            | MethodmapItemData::Constructor(id)
            | MethodmapItemData::Destructor(id) => Some(*id),
            _ => None,
        })
    }

    pub fn properties(&self) -> impl Iterator<Item = PropertyId> + '_ {
        self.items.iter().filter_map(|(_, item)| match item {
            MethodmapItemData::Property(property_data) => Some(property_data.id),
            _ => None,
        })
    }

    pub fn getters_setters(&self) -> impl Iterator<Item = FunctionId> + '_ {
        self.items
            .iter()
            .filter_map(|(_, item)| match item {
                MethodmapItemData::Property(property_data) => Some(property_data),
                _ => None,
            })
            .flat_map(|property_data| {
                property_data.getters_setters.iter().map(|it| match it {
                    PropertyItem::Getter(id) => *id,
                    PropertyItem::Setter(id) => *id,
                })
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedefData {
    pub name: Option<Name>,
    pub type_ref: TypeRef,
    pub deprecated: bool,
}

impl TypedefData {
    pub(crate) fn typedef_data_query(db: &dyn DefDatabase, id: TypedefId) -> Arc<TypedefData> {
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let typedef = &item_tree[loc.value];
        let typedef_data = TypedefData {
            name: typedef.name.clone(),
            type_ref: typedef.type_ref.clone(),
            deprecated: typedef.deprecated,
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
    pub deprecated: bool,
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
            deprecated: typeset.deprecated,
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
    pub deprecated: bool,
}

impl FunctagData {
    pub(crate) fn functag_data_query(db: &dyn DefDatabase, id: FunctagId) -> Arc<Self> {
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let functag = &item_tree[loc.value];
        let functag_data = FunctagData {
            name: functag.name.clone(),
            type_ref: functag.type_ref.clone(),
            deprecated: functag.deprecated,
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
    pub deprecated: bool,
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
            deprecated: funcenum.deprecated,
        };

        Arc::new(functag_data)
    }

    pub fn name(&self) -> Name {
        self.name.clone()
    }
}

/// A single field of a struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldData {
    pub name: Name,
    pub type_ref: TypeRef,
    pub const_: bool,
    pub deprecated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructData {
    pub name: Name,
    pub fields: Arc<Arena<StructFieldData>>,
    pub deprecated: bool,
}

impl StructData {
    pub(crate) fn struct_data_query(db: &dyn DefDatabase, id: StructId) -> Arc<Self> {
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let struct_ = &item_tree[loc.value];
        let mut fields = Arena::new();
        struct_.fields.clone().for_each(|field_idx| {
            let field = &item_tree[field_idx];
            let field_data = StructFieldData {
                name: field.name.clone(),
                type_ref: field.type_ref.clone(),
                const_: field.const_,
                deprecated: field.deprecated,
            };
            let _ = fields.alloc(field_data);
        });
        let struct_data = Self {
            name: struct_.name.clone(),
            fields: fields.into(),
            deprecated: struct_.deprecated,
        };

        Arc::new(struct_data)
    }

    pub fn name(&self) -> Name {
        self.name.clone()
    }

    pub fn field(&self, item: Idx<StructFieldData>) -> &StructFieldData {
        &self.fields[item]
    }

    pub fn field_by_name(&self, name: &str) -> Option<Idx<StructFieldData>> {
        let name = Name::from(name);
        self.fields
            .iter()
            .find(|(_, field)| field.name == name)
            .map(|(idx, _)| idx)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumData {
    pub name: Name,
    pub variants: Arc<Arena<VariantData>>,
    pub variants_map: Arc<FxHashMap<Name, Idx<VariantData>>>,
    pub deprecated: bool,
}

impl EnumData {
    pub(crate) fn enum_data_query(db: &dyn DefDatabase, id: EnumId) -> Arc<EnumData> {
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let enum_ = &item_tree[loc.value];
        let mut variants = Arena::new();
        let mut variants_map = FxHashMap::default();
        enum_.variants.clone().for_each(|variant_idx| {
            let variant = &item_tree[variant_idx];
            let variant_data = VariantData {
                name: variant.name.clone(),
                deprecated: variant.deprecated,
            };
            let variant_id = variants.alloc(variant_data);
            variants_map.insert(variant.name.clone(), variant_id);
        });
        let enum_data = EnumData {
            name: enum_.name.clone(),
            variants: Arc::new(variants),
            variants_map: Arc::new(variants_map),
            deprecated: enum_.deprecated,
        };

        Arc::new(enum_data)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantData {
    pub name: Name,
    pub deprecated: bool,
}

impl VariantData {
    pub(crate) fn variant_data_query(db: &dyn DefDatabase, id: VariantId) -> Arc<VariantData> {
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let variant = &item_tree[loc.value];

        VariantData {
            name: variant.name.clone(),
            deprecated: variant.deprecated,
        }
        .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumStructData {
    pub name: Name,
    pub items: Arc<Arena<EnumStructItemData>>,
    pub items_map: Arc<FxHashMap<Name, Idx<EnumStructItemData>>>,
    pub deprecated: bool,
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
    pub deprecated: bool,
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
                    deprecated: field.deprecated,
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
            deprecated: enum_struct.deprecated,
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

    pub fn field(&self, item: Idx<EnumStructItemData>) -> Option<&FieldData> {
        match &self.items[item] {
            EnumStructItemData::Field(field_data) => Some(field_data),
            EnumStructItemData::Method(_) => None,
        }
    }

    pub fn items(&self, name: &Name) -> Option<Idx<EnumStructItemData>> {
        self.items_map.get(name).cloned()
    }

    pub fn fields(&self) -> impl Iterator<Item = LocalFieldId> + '_ {
        self.items
            .iter()
            .filter_map(|(it, _)| self.field(it).map(|_| it))
    }

    pub fn methods(&self) -> impl Iterator<Item = FunctionId> + '_ {
        self.items
            .iter()
            .filter_map(|(it, _)| self.method(it).cloned())
    }

    pub fn field_type(&self, field: Idx<EnumStructItemData>) -> Option<&TypeRef> {
        match &self.items[field] {
            EnumStructItemData::Field(field_data) => Some(&field_data.type_ref),
            EnumStructItemData::Method(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalData {
    name: Name,
    type_ref: Option<TypeRef>,
    visibility: RawVisibilityId,
}

impl GlobalData {
    pub(crate) fn global_data_query(db: &dyn DefDatabase, id: GlobalId) -> Arc<GlobalData> {
        let global_id = id.lookup(db);
        let item_tree = global_id.item_tree(db);
        let global = &item_tree[global_id];
        let global_data = GlobalData {
            name: global.name.clone(),
            type_ref: global.type_ref.clone(),
            visibility: global.visibility,
        };

        Arc::new(global_data)
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn type_ref(&self) -> Option<&TypeRef> {
        self.type_ref.as_ref()
    }

    pub fn visibility(&self) -> RawVisibilityId {
        self.visibility
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
        let source = db.preprocessed_text(loc.file_id());
        for child in enum_struct_node
            .children(&mut enum_struct_node.walk())
            .filter(|c| TSKind::from(c) == TSKind::enum_struct_field)
        {
            let name_node = child.child_by_field_name("name").unwrap();
            let name = Name::from_node(&name_node, &source);
            let type_ref = TypeRef::from_returntype_node(&child, "type", &source).unwrap();
            let field = EnumStructItemData::Field(FieldData {
                name,
                type_ref,
                deprecated: Default::default(),
            });
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
        let source = db.preprocessed_text(loc.file_id());
        for child in typeset_node
            .children(&mut typeset_node.walk())
            .filter(|c| TSKind::from(c) == TSKind::typedef_expression)
        {
            if let Some(type_ref) = TypeRef::from_returntype_node(&child, "returnType", &source) {
                let typedef = TypedefData {
                    name: None,
                    type_ref,
                    deprecated: Default::default(),
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
        let source = db.preprocessed_text(loc.file_id());
        for child in typeset_node
            .children(&mut typeset_node.walk())
            .filter(|c| TSKind::from(c) == TSKind::typedef_expression)
        {
            let type_ref = TypeRef::from_returntype_node(&child, "returnType", &source);
            let functag = FunctagData {
                name: None,
                type_ref,
                deprecated: Default::default(),
            };
            map.insert(functags.alloc(functag), NodePtr::from(&child));
        }
        InFile::new(loc.file_id(), map)
    }
}

impl HasChildSource<LocalStructFieldId> for StructId {
    type Value = NodePtr;

    fn child_source(
        &self,
        db: &dyn DefDatabase,
    ) -> InFile<ArenaMap<LocalStructFieldId, Self::Value>> {
        let loc = self.lookup(db).id;
        let mut map = ArenaMap::default();
        let tree = db.parse(loc.file_id());
        let mut items = Arena::new();
        let struct_node = loc.source(db, &tree).value;
        let source = db.preprocessed_text(loc.file_id());
        for child in struct_node
            .children(&mut struct_node.walk())
            .filter(|c| TSKind::from(c) == TSKind::struct_field)
        {
            let name_node = child.child_by_field_name("name").unwrap();
            let name = Name::from_node(&name_node, &source);
            let type_ref = TypeRef::from_returntype_node(&child, "type", &source).unwrap();
            let field = StructFieldData {
                name,
                type_ref,
                const_: child
                    .children(&mut child.walk())
                    .any(|c| TSKind::from(c) == TSKind::anon_const),
                deprecated: Default::default(),
            };
            map.insert(items.alloc(field), NodePtr::from(&child));
        }
        InFile::new(loc.file_id(), map)
    }
}
