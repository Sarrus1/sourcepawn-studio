use core::fmt;

use base_db::Tree;
use db::HirDatabase;
use hir_def::{
    resolver::{HasResolver, ValueNs},
    type_string_from_node, DefDiagnostic, DefWithBodyId, EnumId, EnumStructId, ExprId, FuncenumId,
    FunctagId, FunctionId, FunctionKind, GlobalId, InFile, InferenceDiagnostic, ItemContainerId,
    LocalFieldId, Lookup, MacroId, MethodmapExtension, MethodmapId, Name, NodePtr, PropertyId,
    SpecialMethod, TypedefId, TypesetId, VariantId,
};
use itertools::Itertools;
use la_arena::RawIdx;
use preprocessor::PreprocessorError;
use serde::{
    de::{self, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use stdx::impl_from;
use syntax::TSKind;
use vfs::FileId;

pub mod db;
mod diagnostics;
mod from_id;
mod has_source;
mod semantics;
mod source_analyzer;
mod source_to_def;

pub use crate::{diagnostics::*, has_source::HasSource, semantics::Semantics};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefResolution {
    Function(Function),
    Macro(Macro),
    EnumStruct(EnumStruct),
    Methodmap(Methodmap),
    Property(Property),
    Enum(Enum),
    Variant(Variant),
    Typedef(Typedef),
    Typeset(Typeset),
    Functag(Functag),
    Funcenum(Funcenum),
    Field(Field),
    Global(Global),
    Local((Option<Name>, Local)),
    File(File),
}

impl_from!(
    Function,
    Macro,
    Methodmap,
    Property,
    EnumStruct,
    Field,
    Global,
    Typedef,
    Typeset,
    Functag,
    Funcenum,
    File for DefResolution
);

impl From<Local> for DefResolution {
    fn from(local: Local) -> Self {
        DefResolution::Local((None, local))
    }
}

impl DefResolution {
    fn try_from(value: ValueNs) -> Option<Self> {
        match value {
            ValueNs::FunctionId(ids) => {
                DefResolution::Function(Function::from(ids.first()?.value)).into()
            }
            ValueNs::LocalId((name, id, expr)) => {
                DefResolution::Local((name, Local::from((id, expr)))).into()
            }
            ValueNs::MacroId(id) => DefResolution::Macro(Macro::from(id.value)).into(),
            ValueNs::GlobalId(id) => DefResolution::Global(Global::from(id.value)).into(),
            ValueNs::EnumStructId(id) => {
                DefResolution::EnumStruct(EnumStruct::from(id.value)).into()
            }
            ValueNs::MethodmapId(id) => DefResolution::Methodmap(Methodmap::from(id.value)).into(),
            ValueNs::EnumId(id) => DefResolution::Enum(Enum::from(id.value)).into(),
            ValueNs::VariantId(id) => DefResolution::Variant(Variant::from(id.value)).into(),
            ValueNs::TypedefId(id) => DefResolution::Typedef(Typedef::from(id.value)).into(),
            ValueNs::TypesetId(id) => DefResolution::Typeset(Typeset::from(id.value)).into(),
            ValueNs::FunctagId(id) => DefResolution::Functag(Functag::from(id.value)).into(),
            ValueNs::FuncenumId(id) => DefResolution::Funcenum(Funcenum::from(id.value)).into(),
        }
    }
}

impl<'tree> HasSource<'tree> for DefResolution {
    fn source(
        self,
        db: &dyn HirDatabase,
        tree: &'tree Tree,
    ) -> Option<InFile<tree_sitter::Node<'tree>>> {
        match self {
            DefResolution::Function(func) => func.source(db, tree),
            DefResolution::Macro(macro_) => macro_.source(db, tree),
            DefResolution::EnumStruct(enum_struct) => enum_struct.source(db, tree),
            DefResolution::Methodmap(methodmap) => methodmap.source(db, tree),
            DefResolution::Property(property) => property.source(db, tree),
            DefResolution::Enum(enum_) => enum_.source(db, tree),
            DefResolution::Variant(variant) => variant.source(db, tree),
            DefResolution::Typedef(typedef) => typedef.source(db, tree),
            DefResolution::Typeset(typeset) => typeset.source(db, tree),
            DefResolution::Functag(functag) => functag.source(db, tree),
            DefResolution::Funcenum(funcenum) => funcenum.source(db, tree),
            DefResolution::Field(field) => field.source(db, tree),
            DefResolution::Global(global) => global.source(db, tree),
            DefResolution::Local(local) => local.1.source(db, tree)?.source(db, tree),
            DefResolution::File(file) => file.source(db, tree),
        }
    }
}

impl DefResolution {
    pub fn file_id(&self, db: &dyn HirDatabase) -> FileId {
        match self {
            DefResolution::Function(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Macro(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::EnumStruct(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Methodmap(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Property(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Enum(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Variant(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Typedef(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Typeset(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Functag(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Funcenum(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Field(it) => it.parent.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Global(it) => it.id.lookup(db.upcast()).file_id(),
            DefResolution::Local(it) => it.1.parent.file_id(db.upcast()),
            DefResolution::File(it) => it.id,
        }
    }

    pub fn name(&self, db: &dyn HirDatabase) -> Option<Name> {
        match self {
            DefResolution::Function(it) => Some(it.name(db)),
            DefResolution::Macro(it) => Some(it.name(db)),
            DefResolution::EnumStruct(it) => Some(it.name(db)),
            DefResolution::Methodmap(it) => Some(it.name(db)),
            DefResolution::Property(it) => Some(it.name(db)),
            DefResolution::Enum(it) => Some(it.name(db)),
            DefResolution::Variant(it) => Some(it.name(db)),
            DefResolution::Typedef(it) => it.name(db),
            DefResolution::Typeset(it) => Some(it.name(db)),
            DefResolution::Functag(it) => it.name(db),
            DefResolution::Funcenum(it) => Some(it.name(db)),
            DefResolution::Field(it) => Some(it.name(db)),
            DefResolution::Global(it) => Some(it.name(db)),
            DefResolution::Local(it) => {
                if let Some(name) = &it.0 {
                    Some(name.clone())
                } else {
                    it.1.name(db)
                }
            }
            DefResolution::File(_) => None,
        }
    }

    pub fn type_def(&self, db: &dyn HirDatabase) -> Option<DefResolution> {
        match self {
            DefResolution::Function(it) => it.return_type_def(db),
            DefResolution::Macro(_) => None,
            DefResolution::EnumStruct(_) => self.clone().into(),
            DefResolution::Methodmap(_) => self.clone().into(),
            DefResolution::Property(it) => it.type_(db),
            DefResolution::Enum(_) => self.clone().into(),
            DefResolution::Variant(_) => None,
            DefResolution::Typedef(_) => self.clone().into(),
            DefResolution::Typeset(_) => self.clone().into(),
            DefResolution::Functag(_) => self.clone().into(),
            DefResolution::Funcenum(_) => self.clone().into(),
            DefResolution::Field(it) => it.type_(db),
            DefResolution::Global(it) => it.type_(db),
            DefResolution::Local(it) => it.1.type_(db),
            DefResolution::File(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct File {
    pub(crate) id: FileId,
}

impl From<FileId> for File {
    fn from(file_id: FileId) -> Self {
        File { id: file_id }
    }
}

impl File {
    pub fn file_id(&self) -> FileId {
        self.id
    }

    pub fn declarations(self, db: &dyn HirDatabase) -> Vec<FileDef> {
        let db = db.upcast();
        let def_map = db.file_def_map(self.id);
        def_map
            .declarations()
            .iter()
            .map(|it| FileDef::from(*it))
            .collect::<Vec<_>>()
    }

    pub fn diagnostics(self, db: &dyn HirDatabase, acc: &mut Vec<AnyDiagnostic>) {
        let result = db.preprocess_file(self.id);
        let errors = result.errors();
        acc.extend(errors.evaluation_errors.iter().map(|it| {
            AnyDiagnostic::PreprocessorEvaluationError(
                PreprocessorEvaluationError {
                    range: *it.range(),
                    text: it.text().to_owned(),
                }
                .into(),
            )
        }));
        acc.extend(errors.unresolved_include_errors.iter().map(|it| {
            AnyDiagnostic::UnresolvedInclude(
                UnresolvedInclude {
                    range: *it.range(),
                    path: it.text().to_owned(),
                }
                .into(),
            )
        }));
        acc.extend(errors.macro_not_found_errors.iter().map(|it| {
            AnyDiagnostic::UnresolvedMacro(
                UnresolvedMacro {
                    range: *it.range(),
                    name: it.text().to_owned(),
                }
                .into(),
            )
        }));
        acc.extend(
            result
                .inactive_ranges()
                .iter()
                .map(|range| AnyDiagnostic::InactiveCode(InactiveCode { range: *range }.into())),
        );
        self.declarations(db)
            .iter()
            .for_each(|it| acc.extend(it.diagnostics(db)));
    }
}

impl<'tree> File {
    fn source(
        self,
        _db: &dyn HirDatabase,
        tree: &'tree Tree,
    ) -> Option<InFile<tree_sitter::Node<'tree>>> {
        InFile::new(self.id, tree.root_node()).into()
    }
}

/// The defs which can be visible in the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileDef {
    Function(Function),
    Macro(Macro),
    EnumStruct(EnumStruct),
    Methodmap(Methodmap),
    Global(Global),
    Enum(Enum),
    Variant(Variant),
    Typedef(Typedef),
    Typeset(Typeset),
    Functag(Functag),
    Funcenum(Funcenum),
}

impl_from!(Function, Macro, EnumStruct, Methodmap, Global, Enum, Variant, Typedef for FileDef);

impl FileDef {
    pub fn diagnostics(self, db: &dyn HirDatabase) -> Vec<AnyDiagnostic> {
        let mut acc = Vec::new();
        if let Some(def) = self.as_def_with_body() {
            def.diagnostics(db, &mut acc);
        } else {
            match self {
                FileDef::Methodmap(it) => {
                    let data = db.methodmap_data_with_diagnostics(it.id);
                    data.0
                        .methods()
                        .chain(data.0.getters_setters())
                        .flat_map(|id| FileDef::from(Function::from(id)).as_def_with_body())
                        .for_each(|def| def.diagnostics(db, &mut acc));
                    for diag in data.1.iter() {
                        match diag {
                            DefDiagnostic::UnresolvedInherit {
                                methodmap_ast_id,
                                inherit_name,
                                exists,
                            } => {
                                let file_id = it.id.lookup(db.upcast()).id.file_id();
                                let tree = db.parse(file_id);
                                let ast_id_map = db.ast_id_map(file_id);
                                let Some(methodmap_node) =
                                    ast_id_map[*methodmap_ast_id].to_node(&tree)
                                else {
                                    continue;
                                };
                                // FIXME: This is not ideal, we have to find a better way to get the node.
                                if let Some(inherit_node) =
                                    methodmap_node.child_by_field_name("inherits")
                                {
                                    acc.push(AnyDiagnostic::UnresolvedInherit(
                                        UnresolvedInherit {
                                            expr: InFile::new(
                                                file_id,
                                                NodePtr::from(&inherit_node),
                                            ),
                                            inherit: inherit_name.clone(),
                                            exists: *exists,
                                        }
                                        .into(),
                                    ));
                                }
                            }
                        }
                    }
                }
                FileDef::EnumStruct(it) => {
                    let data = db.enum_struct_data(it.id);
                    data.methods()
                        .flat_map(|id| FileDef::from(Function::from(id)).as_def_with_body())
                        .for_each(|def| def.diagnostics(db, &mut acc));
                }
                _ => (),
            }
        }

        acc
    }

    pub fn as_def_with_body(self) -> Option<DefWithBody> {
        match self {
            FileDef::Function(it) => Some(it.into()),
            FileDef::Typedef(it) => Some(it.into()),
            FileDef::Functag(it) => Some(it.into()),
            FileDef::EnumStruct(_)
            | FileDef::Methodmap(_)
            | FileDef::Global(_)
            | FileDef::Macro(_)
            | FileDef::Enum(_)
            | FileDef::Variant(_)
            | FileDef::Typeset(_)
            | FileDef::Funcenum(_) => None,
        }
    }
}

/// The defs which have a body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefWithBody {
    Function(Function),
    Typedef(Typedef),
    Functag(Functag),
}
impl_from!(Function, Typedef, Functag for DefWithBody);

impl DefWithBody {
    pub fn name(self, db: &dyn HirDatabase) -> Option<Name> {
        match self {
            DefWithBody::Function(f) => Some(f.name(db)),
            DefWithBody::Typedef(t) => t.name(db),
            DefWithBody::Functag(f) => f.name(db),
        }
    }

    pub fn diagnostics(self, db: &dyn HirDatabase, acc: &mut Vec<AnyDiagnostic>) {
        db.unwind_if_cancelled();

        let (_, source_map) = db.body_with_source_map(self.into());
        let infer = db.infer(self.into());
        let expr_syntax = |expr| source_map.expr_source(expr).expect("no matching source");
        for d in infer.diagnostics.iter() {
            match d {
                InferenceDiagnostic::UnresolvedField {
                    expr,
                    receiver,
                    name,
                    method_with_same_name_exists,
                } => acc.push(
                    UnresolvedField {
                        expr: expr_syntax(*expr),
                        name: name.clone(),
                        receiver: receiver.clone(),
                        method_with_same_name_exists: *method_with_same_name_exists,
                    }
                    .into(),
                ),
                InferenceDiagnostic::UnresolvedMethodCall {
                    expr,
                    receiver,
                    name,
                    field_with_same_name_exists,
                } => acc.push(
                    UnresolvedMethodCall {
                        expr: expr_syntax(*expr),
                        name: name.clone(),
                        receiver: receiver.clone(),
                        field_with_same_name_exists: *field_with_same_name_exists,
                    }
                    .into(),
                ),
                InferenceDiagnostic::UnresolvedConstructor {
                    expr,
                    methodmap,
                    exists,
                } => {
                    let exists = match exists {
                        Some(hir_def::ConstructorDiagnosticKind::Methodmap) => {
                            Some(ConstructorDiagnosticKind::Methodmap)
                        }
                        Some(hir_def::ConstructorDiagnosticKind::EnumStruct) => {
                            Some(ConstructorDiagnosticKind::EnumStruct)
                        }
                        _ => None,
                    };
                    acc.push(
                        UnresolvedConstructor {
                            expr: expr_syntax(*expr),
                            methodmap: methodmap.clone(),
                            exists,
                        }
                        .into(),
                    )
                }
                InferenceDiagnostic::UnresolvedNamedArg { expr, name, callee } => acc.push(
                    UnresolvedNamedArg {
                        expr: expr_syntax(*expr),
                        name: name.clone(),
                        callee: callee.clone(),
                    }
                    .into(),
                ),
                InferenceDiagnostic::IncorrectNumberOfArguments {
                    expr,
                    name,
                    expected,
                    actual,
                } => acc.push(
                    IncorrectNumberOfArguments {
                        expr: expr_syntax(*expr),
                        name: name.clone(),
                        expected: *expected,
                        actual: *actual,
                    }
                    .into(),
                ),
                InferenceDiagnostic::InvalidUseOfThis { expr } => acc.push(
                    InvalidUseOfThis {
                        expr: expr_syntax(*expr),
                    }
                    .into(),
                ),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctionType {
    Function,
    Method,
    Getter,
    Setter,
    Constructor,
    Destructor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Function {
    pub(crate) id: FunctionId,
}

impl Function {
    pub fn id(self) -> FunctionId {
        self.id
    }

    pub fn name(self, db: &dyn HirDatabase) -> Name {
        db.function_data(self.id).name.clone()
    }

    pub fn type_ref(self, db: &dyn HirDatabase) -> Option<String> {
        db.function_data(self.id)
            .type_ref
            .as_ref()
            .map(|it| it.to_string())
    }

    pub fn return_type_def(self, db: &dyn HirDatabase) -> Option<DefResolution> {
        let type_ref = db.function_data(self.id).type_ref.clone()?;
        let ty_str = type_ref.type_as_string();
        self.id
            .resolver(db.upcast())
            .resolve_ident(&ty_str)
            .and_then(DefResolution::try_from)
    }

    pub fn type_def(self, db: &dyn HirDatabase) -> Vec<DefResolution> {
        let mut res = Vec::new();
        if let Some(return_type_def) = self.return_type_def(db) {
            res.push(return_type_def);
        };

        match self.id.lookup(db.upcast()).container {
            ItemContainerId::MethodmapId(it) => {
                res.push(Methodmap::from(it).into());
            }
            ItemContainerId::EnumStructId(it) => {
                res.push(EnumStruct::from(it).into());
            }
            ItemContainerId::FileId(_)
            | ItemContainerId::EnumId(_)
            | ItemContainerId::TypesetId(_)
            | ItemContainerId::FuncenumId(_) => (),
        }

        res
    }

    pub fn render(self, db: &dyn HirDatabase) -> Option<String> {
        let data = db.function_data(self.id);

        let mut buf = String::new();
        buf.push_str(&data.visibility.to_string());
        if !buf.is_empty() {
            buf.push(' ');
        }
        match data.kind {
            FunctionKind::Def => (),
            FunctionKind::Forward => buf.push_str("forward "),
            FunctionKind::Native => buf.push_str("native "),
        }
        if let Some(type_ref) = &data.type_ref {
            buf.push_str(&type_ref.to_string());
            buf.push(' ');
        }
        if data.special == Some(SpecialMethod::Destructor) {
            buf.push('~');
        }
        buf.push_str(&self.name(db).to_string());

        let file_id = self.id.lookup(db.upcast()).id.file_id();
        let tree = db.parse(file_id);
        let node = self.source(db, &tree)?;
        let source = db.preprocessed_text(file_id);

        if let Some(params) = node
            .value
            .child_by_field_name("parameters")
            .and_then(|params_node| {
                params_node
                    .utf8_text(source.as_bytes())
                    .ok()
                    .map(String::from)
            })
        {
            buf.push_str(&params);
        }
        let Some(parent) = node.value.parent() else {
            return buf.to_string().into();
        };

        if let Some(parent_name) = parent.child_by_field_name("name").and_then(|name_node| {
            name_node
                .utf8_text(source.as_bytes())
                .ok()
                .map(String::from)
        }) {
            buf = format!("{}\n{}", parent_name, buf);
        }

        buf.to_string().into()
    }

    pub fn kind(self, db: &dyn HirDatabase) -> FunctionType {
        let item = self.id.lookup(db.upcast());
        match item.container {
            ItemContainerId::MethodmapId(container) => {
                let method_map = db.methodmap_data(container);
                if let Some(constructor_id) = method_map.constructor() {
                    if *constructor_id == self.id {
                        return FunctionType::Constructor;
                    }
                }
                if let Some(destructor_id) = method_map.destructor() {
                    if *destructor_id == self.id {
                        return FunctionType::Destructor;
                    }
                }
                FunctionType::Method
            }

            ItemContainerId::EnumStructId(_) => FunctionType::Method,
            ItemContainerId::FileId(_) => FunctionType::Function,
            _ => unreachable!("unexpected container for a function"),
        }
    }

    pub fn as_snippet(self, db: &dyn HirDatabase) -> Option<String> {
        let loc = self.id.lookup(db.upcast());
        let source = db.preprocessed_text(loc.id.file_id());
        let file_id = loc.id.file_id();
        let tree = db.parse(file_id);
        let node = self.source(db, &tree)?.value;
        let type_ = node
            .child_by_field_name("returnType")?
            .utf8_text(source.as_bytes())
            .ok()?;
        let name = node
            .child_by_field_name("name")?
            .utf8_text(source.as_bytes())
            .ok()?;
        let params = node.child_by_field_name("parameters")?;
        let mut buf = format!("public {} {}(", type_, name);
        let mut at_least_one_param = false;
        for (i, param) in params
            .children(&mut params.walk())
            .filter(|n| {
                matches!(
                    TSKind::from(n),
                    TSKind::parameter_declaration | TSKind::rest_parameter
                )
            })
            .enumerate()
        {
            at_least_one_param = true;
            let Some(name_node) = param.child_by_field_name("name") else {
                continue;
            };
            let name = name_node.utf8_text(source.as_bytes()).ok()?;
            let cur = param.utf8_text(source.as_bytes()).ok()?;
            buf.push_str(&cur.replace(name, &format!("${{{}:{}}}", i + 2, name)));
            buf.push_str(", ");
        }
        if at_least_one_param {
            buf = buf.trim_end().to_string();
            buf.pop();
        }
        buf.push_str(")\n{\n\t$0\n}");

        buf.into()
    }

    /// Returns whether the function is deprecated.
    ///
    /// This method is "fast" as it does not do a lookup of the node in the tree.
    pub fn is_deprecated(self, db: &dyn HirDatabase) -> bool {
        db.function_data(self.id).deprecated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Macro {
    pub(crate) id: MacroId,
}

impl Macro {
    pub fn name(self, db: &dyn HirDatabase) -> Name {
        db.macro_data(self.id).name.clone()
    }

    pub fn render(self, db: &dyn HirDatabase) -> Option<String> {
        let file_id = self.id.lookup(db.upcast()).id.file_id();
        let tree = db.parse(file_id);
        let node = self.source(db, &tree)?.value;
        let source = db.preprocessed_text(file_id);

        let mut buf = "#define ".to_string();
        buf.push_str(
            node.child_by_field_name("name")?
                .utf8_text(source.as_bytes())
                .ok()?
                .trim(),
        );

        if TSKind::from(node) == TSKind::preproc_macro {
            buf.push_str(
                node.child_by_field_name("parameters")?
                    .utf8_text(source.as_bytes())
                    .ok()?
                    .trim(),
            );
        }
        buf.push(' ');
        buf.push_str(
            node.child_by_field_name("value")?
                .utf8_text(source.as_bytes())
                .ok()?
                .trim(),
        );

        buf.trim().to_string().into()
    }

    /// Returns whether the macro is deprecated.
    ///
    /// This method is "fast" as it does not do a lookup of the node in the tree.
    pub fn is_deprecated(self, db: &dyn HirDatabase) -> bool {
        db.macro_data(self.id).deprecated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnumStruct {
    pub(crate) id: EnumStructId,
}

impl EnumStruct {
    pub fn id(self) -> EnumStructId {
        self.id
    }

    pub fn name(self, db: &dyn HirDatabase) -> Name {
        db.enum_struct_data(self.id).name.clone()
    }

    pub fn render(self, db: &dyn HirDatabase) -> Option<String> {
        format!("enum struct {}", self.name(db)).into()
    }

    /// Returns whether the enum struct is deprecated.
    ///
    /// This method is "fast" as it does not do a lookup of the node in the tree.
    pub fn is_deprecated(self, db: &dyn HirDatabase) -> bool {
        db.enum_struct_data(self.id).deprecated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Methodmap {
    pub(crate) id: MethodmapId,
}

impl Methodmap {
    pub fn id(self) -> MethodmapId {
        self.id
    }

    pub fn name(self, db: &dyn HirDatabase) -> Name {
        db.methodmap_data(self.id).name.clone()
    }

    pub fn extension(self, db: &dyn HirDatabase) -> Option<MethodmapExtension> {
        db.methodmap_data(self.id).extension.clone()
    }

    pub fn render(self, db: &dyn HirDatabase) -> Option<String> {
        let mut buf = format!("methodmap {}", self.name(db));
        match self.extension(db) {
            Some(MethodmapExtension::Inherits(inherits)) => {
                buf.push_str(&format!(" < {}", inherits))
            }
            Some(MethodmapExtension::Nullable) => buf.push_str(" __nullable__"),
            None => (),
        }

        buf.into()
    }

    pub fn type_def(self, db: &dyn HirDatabase) -> Vec<DefResolution> {
        let mut res = Vec::new();
        if let Some(inherits) = db.methodmap_data(self.id).inherits {
            res.push(Methodmap::from(inherits).into());
        }

        res
    }

    /// Returns whether the methodmap is deprecated.
    ///
    /// This method is "fast" as it does not do a lookup of the node in the tree.
    pub fn is_deprecated(self, db: &dyn HirDatabase) -> bool {
        db.methodmap_data(self.id).deprecated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Property {
    pub(crate) id: PropertyId,
}

impl Property {
    pub fn name(self, db: &dyn HirDatabase) -> Name {
        db.property_data(self.id).name.clone()
    }

    pub fn render(self, db: &dyn HirDatabase) -> Option<String> {
        let data = db.property_data(self.id);
        let ItemContainerId::MethodmapId(parent_id) = self.id.lookup(db.upcast()).container else {
            panic!("expected a property to have a methodmap as a parent");
        };
        let parent_name = db.methodmap_data(parent_id).name.to_string();
        let mut buf = format!("{}::", parent_name);
        buf.push_str("property ");
        buf.push_str(&data.type_ref.to_string());
        buf.push(' ');
        buf.push_str(&self.name(db).to_string());

        buf.into()
    }

    pub fn type_(self, db: &dyn HirDatabase) -> Option<DefResolution> {
        let ty = db.property_data(self.id).type_ref.clone();
        let ty_str = ty.type_as_string();
        self.id
            .resolver(db.upcast())
            .resolve_ident(&ty_str)
            .and_then(DefResolution::try_from)
    }

    pub fn type_def(self, db: &dyn HirDatabase) -> Vec<DefResolution> {
        let mut res = Vec::new();
        if let Some(return_type_def) = self.type_(db) {
            res.push(return_type_def);
            return res;
        }

        match self.id.lookup(db.upcast()).container {
            ItemContainerId::MethodmapId(it) => {
                res.push(Methodmap::from(it).into());
            }
            ItemContainerId::FileId(_)
            | ItemContainerId::EnumStructId(_)
            | ItemContainerId::EnumId(_)
            | ItemContainerId::TypesetId(_)
            | ItemContainerId::FuncenumId(_) => (),
        }

        res
    }

    /// Returns whether the property is deprecated.
    ///
    /// This method is "fast" as it does not do a lookup of the node in the tree.
    pub fn is_deprecated(self, db: &dyn HirDatabase) -> bool {
        db.property_data(self.id).deprecated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Enum {
    pub(crate) id: EnumId,
}

impl Enum {
    pub fn name(self, db: &dyn HirDatabase) -> Name {
        db.enum_data(self.id).name.clone()
    }

    pub fn render(self, db: &dyn HirDatabase) -> Option<String> {
        format!("enum {}", self.name(db)).into()
    }

    /// Returns whether the enum is deprecated.
    ///
    /// This method is "fast" as it does not do a lookup of the node in the tree.
    pub fn is_deprecated(self, db: &dyn HirDatabase) -> bool {
        db.enum_data(self.id).deprecated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Variant {
    pub(crate) id: VariantId,
}

impl Variant {
    pub fn name(self, db: &dyn HirDatabase) -> Name {
        db.variant_data(self.id).name.clone()
    }

    pub fn render(self, db: &dyn HirDatabase) -> Option<String> {
        let ItemContainerId::EnumId(parent_id) = self.id.lookup(db.upcast()).container else {
            panic!("expected a variant to have an enum as a parent");
        };
        let parent_name = db.enum_data(parent_id).name.to_string();
        let name = self.name(db).to_string();
        if parent_name.is_empty() {
            name.into()
        } else {
            format!("{}::{}", parent_name, name).into()
        }
    }

    pub fn type_def(self, db: &dyn HirDatabase) -> Vec<DefResolution> {
        let ItemContainerId::EnumId(parent_id) = self.id.lookup(db.upcast()).container else {
            panic!("expected a variant to have an enum as a parent");
        };

        vec![DefResolution::Enum(parent_id.into())]
    }

    /// Returns whether the variant is deprecated.
    ///
    /// This method is "fast" as it does not do a lookup of the node in the tree.
    pub fn is_deprecated(self, db: &dyn HirDatabase) -> bool {
        db.variant_data(self.id).deprecated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Typedef {
    pub(crate) id: TypedefId,
}

impl Typedef {
    pub fn id(self) -> TypedefId {
        self.id
    }

    pub fn name(self, db: &dyn HirDatabase) -> Option<Name> {
        db.typedef_data(self.id).name.clone()
    }

    pub fn render(self, db: &dyn HirDatabase) -> Option<String> {
        let file_id = self.id.lookup(db.upcast()).id.file_id();
        let tree = db.parse(file_id);
        let node = self.source(db, &tree)?;
        let source = db.preprocessed_text(file_id);

        let data = db.typedef_data(self.id);
        let mut buf = String::new();
        if let Some(name) = self.name(db) {
            buf.push_str("typedef ");
            buf.push_str(&name.to_string());
            buf.push_str(" = ");
        } else {
            // let ItemContainerId::TypesetId(parent_id) = self.id.lookup(db.upcast()).container
            // else {
            //     panic!("expected a typedef to have a typeset as a parent");
            // };
            // let parent_name = db.typeset_data(parent_id).name.to_string();
            // buf.push_str(&parent_name);
            // buf.push_str("::");
            // buf.push('\n');
        }
        buf.push_str("function ");
        buf.push_str(&data.type_ref.to_string());
        buf.push(' ');

        let typedef_expr = if TSKind::from(&node.value) == TSKind::typedef_expression {
            node.value
        } else {
            node.value
                .children(&mut node.value.walk())
                .find(|n| TSKind::from(n) == TSKind::typedef_expression)?
        };

        if let Some(params) =
            typedef_expr
                .child_by_field_name("parameters")
                .and_then(|params_node| {
                    params_node
                        .utf8_text(source.as_bytes())
                        .ok()
                        .map(String::from)
                })
        {
            buf.push_str(&params);
        }

        buf.push(';');

        buf.into()
    }

    pub fn return_type(self, db: &dyn HirDatabase) -> String {
        db.typedef_data(self.id).type_ref.to_string()
    }

    pub fn type_def(self, db: &dyn HirDatabase) -> Vec<DefResolution> {
        let mut res = Vec::new();
        let ty = db.typedef_data(self.id).type_ref.clone();
        let ty_str = ty.type_as_string();
        if let Some(def) = self
            .id
            .resolver(db.upcast())
            .resolve_ident(&ty_str)
            .and_then(DefResolution::try_from)
        {
            res.push(def);
        }

        res
    }

    pub fn as_snippet(self, db: &dyn HirDatabase) -> Option<String> {
        let loc = self.id.lookup(db.upcast());
        let source = db.preprocessed_text(loc.id.file_id());
        let file_id = loc.id.file_id();
        let tree = db.parse(file_id);
        let node = self.source(db, &tree)?.value;
        let typedef_expr = if TSKind::from(&node) == TSKind::typedef_expression {
            node
        } else {
            node.children(&mut node.walk())
                .find(|n| TSKind::from(n) == TSKind::typedef_expression)?
        };
        let type_ = typedef_expr
            .child_by_field_name("returnType")?
            .utf8_text(source.as_bytes())
            .ok()?;
        let params = typedef_expr.child_by_field_name("parameters")?;
        let mut buf = format!("{} ${{1:name}}(", type_);
        let mut at_least_one_param = false;
        for (i, param) in params
            .children(&mut params.walk())
            .filter(|n| {
                matches!(
                    TSKind::from(n),
                    TSKind::parameter_declaration | TSKind::rest_parameter
                )
            })
            .enumerate()
        {
            at_least_one_param = true;
            let Some(name_node) = param.child_by_field_name("name") else {
                continue;
            };
            let name = name_node.utf8_text(source.as_bytes()).ok()?;
            let cur = param.utf8_text(source.as_bytes()).ok()?;
            buf.push_str(&cur.replace(name, &format!("${{{}:{}}}", i + 2, name)));
            buf.push_str(", ");
        }
        if at_least_one_param {
            buf = buf.trim_end().to_string();
            buf.pop();
        }
        buf.push_str(")\n{\n\t$0\n}");

        buf.into()
    }

    /// Returns whether the typedef is deprecated.
    ///
    /// This method is "fast" as it does not do a lookup of the node in the tree.
    pub fn is_deprecated(self, db: &dyn HirDatabase) -> bool {
        db.typedef_data(self.id).deprecated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Typeset {
    pub(crate) id: TypesetId,
}

impl Typeset {
    pub fn name(self, db: &dyn HirDatabase) -> Name {
        db.typeset_data(self.id).name.clone()
    }

    pub fn render(self, db: &dyn HirDatabase) -> Option<String> {
        format!("typeset {}", self.name(db)).into()
    }

    pub fn children(self, db: &dyn HirDatabase) -> Vec<Typedef> {
        let data = db.typeset_data(self.id);
        data.typedefs
            .iter()
            .map(|it| it.1)
            .cloned()
            .map(|it| it.into())
            .collect_vec()
    }

    /// Returns whether the typeset is deprecated.
    ///
    /// This method is "fast" as it does not do a lookup of the node in the tree.
    pub fn is_deprecated(self, db: &dyn HirDatabase) -> bool {
        db.typeset_data(self.id).deprecated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Functag {
    pub(crate) id: FunctagId,
}

impl Functag {
    pub fn name(self, db: &dyn HirDatabase) -> Option<Name> {
        db.functag_data(self.id).name.clone()
    }

    pub fn render(self, db: &dyn HirDatabase) -> Option<String> {
        let file_id = self.id.lookup(db.upcast()).id.file_id();
        let tree = db.parse(file_id);
        let node = self.source(db, &tree)?;
        let source = db.preprocessed_text(file_id);

        let data = db.functag_data(self.id);
        let mut buf = String::new();
        let type_ = data
            .type_ref
            .as_ref()
            .map(|it| it.to_string())
            .unwrap_or_default();
        if let Some(name) = self.name(db) {
            buf.push_str("functag public ");
            buf.push_str(&type_);
            buf.push_str(&name.to_string());
        } else {
            let ItemContainerId::FuncenumId(parent_id) = self.id.lookup(db.upcast()).container
            else {
                panic!("expected a typedef to have a typeset as a parent");
            };
            let parent_name = db.funcenum_data(parent_id).name.to_string();
            buf.push_str(&parent_name);
            buf.push('\n');
            buf.push_str(&type_);
            buf.push_str(":public");
        }

        if let Some(params) = node
            .value
            .child_by_field_name("parameters")
            .and_then(|params_node| {
                params_node
                    .utf8_text(source.as_bytes())
                    .ok()
                    .map(String::from)
            })
        {
            buf.push_str(&params);
        }

        buf.push(';');

        buf.into()
    }

    pub fn type_def(self, db: &dyn HirDatabase) -> Vec<DefResolution> {
        let mut res = Vec::new();
        let Some(ty) = db.functag_data(self.id).type_ref.clone() else {
            return res;
        };
        let ty_str = ty.type_as_string();
        if let Some(def) = self
            .id
            .resolver(db.upcast())
            .resolve_ident(&ty_str)
            .and_then(DefResolution::try_from)
        {
            res.push(def);
        }

        res
    }

    /// Returns whether the functag is deprecated.
    ///
    /// This method is "fast" as it does not do a lookup of the node in the tree.
    pub fn is_deprecated(self, db: &dyn HirDatabase) -> bool {
        db.functag_data(self.id).deprecated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Funcenum {
    pub(crate) id: FuncenumId,
}

impl Funcenum {
    pub fn name(self, db: &dyn HirDatabase) -> Name {
        db.funcenum_data(self.id).name.clone()
    }

    pub fn render(self, db: &dyn HirDatabase) -> Option<String> {
        format!("funcenum {}", self.name(db)).into()
    }

    /// Returns whether the funcenum is deprecated.
    ///
    /// This method is "fast" as it does not do a lookup of the node in the tree.
    pub fn is_deprecated(self, db: &dyn HirDatabase) -> bool {
        db.funcenum_data(self.id).deprecated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Global {
    pub(crate) id: GlobalId,
}

impl Global {
    pub fn name(self, db: &dyn HirDatabase) -> Name {
        db.global_data(self.id).name().clone()
    }

    pub fn render(self, db: &dyn HirDatabase) -> Option<String> {
        let data = db.global_data(self.id);

        let mut buf = String::new();
        buf.push_str(&data.visibility().to_string());
        if !buf.is_empty() {
            buf.push(' ');
        }
        if let Some(type_ref) = data.type_ref() {
            buf.push_str(&type_ref.to_string());
            if !buf.ends_with(':') {
                buf.push(' ');
            }
        }
        buf.push_str(&self.name(db).to_string());
        buf.push(';');

        buf.into()
    }

    pub fn type_(self, db: &dyn HirDatabase) -> Option<DefResolution> {
        let ty = db.global_data(self.id).type_ref().cloned()?;
        let ty_str = ty.type_as_string();
        self.id
            .resolver(db.upcast())
            .resolve_ident(&ty_str)
            .and_then(DefResolution::try_from)
    }

    pub fn type_def(self, db: &dyn HirDatabase) -> Vec<DefResolution> {
        let mut res = Vec::new();
        if let Some(def) = self.type_(db) {
            res.push(def);
        }

        res
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Field {
    pub(crate) parent: EnumStruct,
    pub(crate) id: LocalFieldId,
}

impl Serialize for Field {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Field", 2)?;
        state.serialize_field("parent", &self.parent)?;
        state.serialize_field("id", &self.id.into_raw().into_u32())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Field {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum FieldKey {
            Parent,
            Id,
        }

        impl<'de> Deserialize<'de> for FieldKey {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct KeyVisitor;

                impl<'de> Visitor<'de> for KeyVisitor {
                    type Value = FieldKey;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`parent` or `id`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<FieldKey, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "parent" => Ok(FieldKey::Parent),
                            "id" => Ok(FieldKey::Id),
                            _ => Err(E::custom(format!("unexpected field: {}", value))),
                        }
                    }
                }

                deserializer.deserialize_identifier(KeyVisitor)
            }
        }

        struct FieldVisitor;

        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Field")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Field, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut parent = None;
                let mut id = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        FieldKey::Parent => {
                            if parent.is_some() {
                                return Err(de::Error::duplicate_field("parent"));
                            }
                            parent = Some(map.next_value()?);
                        }
                        FieldKey::Id => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            let id_u32: u32 = map.next_value()?;
                            id = Some(LocalFieldId::from_raw(RawIdx::from_u32(id_u32)));
                        }
                    }
                }

                let parent = parent.ok_or_else(|| de::Error::missing_field("parent"))?;
                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;

                Ok(Field { parent, id })
            }
        }

        const FIELDS: &[&str] = &["parent", "id"];
        deserializer.deserialize_struct("Field", FIELDS, FieldVisitor)
    }
}

impl Field {
    pub fn name(self, db: &dyn HirDatabase) -> Name {
        let parent_data = db.enum_struct_data(self.parent.id);
        parent_data
            .field(self.id)
            .expect("expected a field to have a name")
            .name
            .clone()
    }

    pub fn render(self, db: &dyn HirDatabase) -> Option<String> {
        let parent_data = db.enum_struct_data(self.parent.id);
        let type_str = parent_data
            .field(self.id)
            .expect("expected a field to have a type")
            .type_ref
            .to_string();

        format!("{} {}::{};", type_str, parent_data.name, self.name(db)).into()
    }

    pub fn type_(self, db: &dyn HirDatabase) -> Option<DefResolution> {
        let parent_data = db.enum_struct_data(self.parent.id);
        let ty_str = parent_data
            .field(self.id)
            .expect("expected a field to have a type")
            .type_ref
            .type_as_string();
        self.parent
            .id
            .resolver(db.upcast())
            .resolve_ident(&ty_str)
            .and_then(DefResolution::try_from)
    }

    pub fn type_def(self, db: &dyn HirDatabase) -> Vec<DefResolution> {
        let mut res = Vec::new();

        if let Some(type_) = self.type_(db) {
            res.push(type_);
        }

        res.push(DefResolution::EnumStruct(self.parent));

        res
    }

    /// Returns whether the field is deprecated.
    ///
    /// This method is "fast" as it does not do a lookup of the node in the tree.
    pub fn is_deprecated(self, db: &dyn HirDatabase) -> bool {
        db.enum_struct_data(self.parent.id)
            .field(self.id)
            .expect("expected a field to exist")
            .deprecated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Attribute {
    Field(Field),
    Property(Property),
}

impl_from!(Field, Property for Attribute);

/// A single local variable definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Local {
    pub(crate) parent: DefWithBodyId,
    pub(crate) expr_id: ExprId,
}

impl Serialize for Local {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Local", 2)?;
        state.serialize_field("parent", &self.parent)?;
        state.serialize_field("expr_id", &self.expr_id.into_raw().into_u32())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Local {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Parent,
            ExprId,
        }

        struct LocalVisitor;

        impl<'de> Visitor<'de> for LocalVisitor {
            type Value = Local;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Local")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Local, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut parent = None;
                let mut expr_id = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Parent => {
                            if parent.is_some() {
                                return Err(de::Error::duplicate_field("parent"));
                            }
                            parent = Some(map.next_value()?);
                        }
                        Field::ExprId => {
                            if expr_id.is_some() {
                                return Err(de::Error::duplicate_field("expr_id"));
                            }
                            let expr_id_u32: u32 = map.next_value()?;
                            expr_id = Some(ExprId::from_raw(expr_id_u32.into()));
                        }
                    }
                }
                let parent = parent.ok_or_else(|| de::Error::missing_field("parent"))?;
                let expr_id = expr_id.ok_or_else(|| de::Error::missing_field("expr_id"))?;
                Ok(Local { parent, expr_id })
            }
        }

        const FIELDS: &[&str] = &["parent", "expr_id"];
        deserializer.deserialize_struct("Local", FIELDS, LocalVisitor)
    }
}

impl<'tree> Local {
    fn source(self, db: &dyn HirDatabase, tree: &'tree Tree) -> Option<LocalSource<'tree>> {
        let (_, source_map) = db.body_with_source_map(self.parent);
        let node_ptr = source_map.expr_source(self.expr_id)?;
        Some(LocalSource {
            local: self,
            source: InFile::new(
                self.parent.file_id(db.upcast()),
                node_ptr.value.to_node(tree).expect("failed to find a node"),
            ),
        })
    }

    pub fn name(self, db: &dyn HirDatabase) -> Option<Name> {
        let file_id = self.parent.file_id(db.upcast());
        let tree = db.parse(file_id);
        let source = db.preprocessed_text(file_id);
        let node = self.source(db, &tree)?.source.value;
        node.utf8_text(source.as_bytes()).ok().map(Name::from)
    }

    pub fn render(self, db: &dyn HirDatabase) -> Option<String> {
        let file_id = self.parent.file_id(db.upcast());
        let tree = db.parse(file_id);
        let source = db.preprocessed_text(file_id);
        let node = self.source(db, &tree)?.source.value;
        let name = node.utf8_text(source.as_bytes()).ok().map(String::from)?;
        let Some(parent) = node.parent() else {
            return name.into();
        };
        match TSKind::from(parent) {
            TSKind::variable_declaration_statement => {
                let mut buf = String::new();
                let type_node = parent.child_by_field_name("type")?;
                buf.push_str(type_node.utf8_text(source.as_bytes()).ok()?);
                buf.push(' ');
                buf.push_str(&name);
                buf.push(';');
                Some(buf)
            }
            TSKind::old_variable_declaration_statement => {
                let mut buf = String::new();
                let type_ = parent
                    .child_by_field_name("type")
                    .and_then(|it| it.utf8_text(source.as_bytes()).ok())
                    .unwrap_or_default();
                buf.push_str(type_);
                buf.push_str(parent.utf8_text(source.as_bytes()).ok()?);
                if !buf.ends_with(';') {
                    buf.push(';');
                }
                Some(buf)
            }
            TSKind::parameter_declarations => {
                let mut buf = node.utf8_text(source.as_bytes()).ok()?.to_string();
                if !buf.ends_with(';') {
                    buf.push(';');
                }
                Some(buf)
            }
            _ => None,
        }
    }

    pub fn type_(self, db: &dyn HirDatabase) -> Option<DefResolution> {
        let file_id = self.parent.file_id(db.upcast());
        let tree = db.parse(file_id);
        let source = db.preprocessed_text(file_id);
        let node = self.source(db, &tree).map(|s| s.source.value)?;
        let type_node = node
            .parent()
            .and_then(|it| it.child_by_field_name("type"))?;
        let ty_str = type_string_from_node(&type_node, &source);
        self.parent
            .resolver(db.upcast())
            .resolve_ident(&ty_str)
            .and_then(DefResolution::try_from)
    }

    pub fn type_def(self, db: &dyn HirDatabase) -> Vec<DefResolution> {
        let mut res = Vec::new();
        if let Some(def) = self.type_(db) {
            res.push(def);
        }

        res
    }
}

pub struct LocalSource<'tree> {
    pub local: Local,
    pub source: InFile<tree_sitter::Node<'tree>>,
}
