use base_db::Tree;
use db::HirDatabase;
use hir_def::{
    DefWithBodyId, EnumId, EnumStructId, ExprId, FunctionId, GlobalId, InFile, InferenceDiagnostic,
    LocalFieldId, LocalPropertyId, Lookup, MacroId, MethodmapId, Name, TypedefId, VariantId,
};
use preprocessor::PreprocessorError;
use stdx::impl_from;
use vfs::FileId;

pub mod db;
mod diagnostics;
mod from_id;
mod has_source;
mod semantics;
mod source_analyzer;
mod source_to_def;

pub use crate::{diagnostics::*, has_source::HasSource, semantics::Semantics};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefResolution {
    Function(Function),
    Macro(Macro),
    EnumStruct(EnumStruct),
    Methodmap(Methodmap),
    Property(Property),
    Enum(Enum),
    Variant(Variant),
    Typedef(Typedef),
    Field(Field),
    Global(Global),
    Local(Local),
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
    Local,
    File for DefResolution
);

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
            DefResolution::Field(field) => field.source(db, tree),
            DefResolution::Global(global) => global.source(db, tree),
            DefResolution::Local(local) => local.source(db, tree)?.source(db, tree),
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
            DefResolution::Property(it) => it.parent.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Enum(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Variant(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Typedef(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Field(it) => it.parent.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Global(it) => it.id.lookup(db.upcast()).file_id(),
            DefResolution::Local(it) => it.parent.file_id(db.upcast()),
            DefResolution::File(it) => it.id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub(crate) id: FileId,
}

impl From<FileId> for File {
    fn from(file_id: FileId) -> Self {
        File { id: file_id }
    }
}

impl File {
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
}

impl_from!(Function, Macro, EnumStruct, Global, Enum, Variant, Typedef for FileDef);

impl FileDef {
    pub fn diagnostics(self, db: &dyn HirDatabase) -> Vec<AnyDiagnostic> {
        // let id: FileDefId = match self {
        //     FileDef::Function(it) => it.id.into(),
        //     FileDef::Macro(it) => it.id.into(),
        //     FileDef::EnumStruct(it) => it.id.into(),
        //     FileDef::Global(it) => it.id.into(),
        // };

        let mut acc = Vec::new();

        match self.as_def_with_body() {
            Some(def) => {
                def.diagnostics(db, &mut acc);
            }
            None => {
                // for diag in hir_ty::diagnostics::incorrect_case(db, id) {
                //     acc.push(diag.into())
                // }
            }
        }

        acc
    }

    pub fn as_def_with_body(self) -> Option<DefWithBody> {
        match self {
            FileDef::Function(it) => Some(it.into()),
            FileDef::EnumStruct(_)
            | FileDef::Methodmap(_)
            | FileDef::Global(_)
            | FileDef::Macro(_)
            | FileDef::Enum(_)
            | FileDef::Variant(_)
            | FileDef::Typedef(_) => None,
        }
    }
}

/// The defs which have a body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefWithBody {
    Function(Function),
    Typedef(Typedef),
}
impl_from!(Function for DefWithBody);

impl DefWithBody {
    pub fn name(self, db: &dyn HirDatabase) -> Option<Name> {
        match self {
            DefWithBody::Function(f) => Some(f.name(db)),
            DefWithBody::Typedef(t) => t.name(db),
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
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Function {
    pub(crate) id: FunctionId,
}

impl Function {
    pub fn name(self, db: &dyn HirDatabase) -> Name {
        db.function_data(self.id).name.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Macro {
    pub(crate) id: MacroId,
}

impl Macro {
    pub fn name(self, db: &dyn HirDatabase) -> Name {
        db.macro_data(self.id).name.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumStruct {
    pub(crate) id: EnumStructId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Methodmap {
    pub(crate) id: MethodmapId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Property {
    pub(crate) parent: Methodmap,
    pub(crate) id: LocalPropertyId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Enum {
    pub(crate) id: EnumId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Variant {
    pub(crate) id: VariantId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Typedef {
    pub(crate) id: TypedefId,
}

impl Typedef {
    pub fn name(self, db: &dyn HirDatabase) -> Option<Name> {
        db.typedef_data(self.id).name.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Global {
    pub(crate) id: GlobalId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Field {
    pub(crate) parent: EnumStruct,
    pub(crate) id: LocalFieldId,
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

impl<'tree> Local {
    fn source(self, db: &dyn HirDatabase, tree: &'tree Tree) -> Option<LocalSource<'tree>> {
        let (_, source_map) = db.body_with_source_map(self.parent);
        let node_ptr = source_map.expr_source(self.expr_id)?;
        Some(LocalSource {
            local: self,
            source: InFile::new(
                self.parent.file_id(db.upcast()),
                node_ptr.value.to_node(tree),
            ),
        })
    }
}

pub struct LocalSource<'tree> {
    pub local: Local,
    pub source: InFile<tree_sitter::Node<'tree>>,
}
