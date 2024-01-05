use base_db::Tree;
use db::HirDatabase;
use hir_def::{
    DefWithBodyId, EnumStructId, ExprId, FileDefId, FunctionId, GlobalId, InFile,
    InferenceDiagnostic, LocalFieldId, Name,
};
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
    EnumStruct(EnumStruct),
    Field(Field),
    Global(Global),
    Local(Local),
}

impl<'tree> HasSource<'tree> for DefResolution {
    fn source(
        self,
        db: &dyn HirDatabase,
        tree: &'tree Tree,
    ) -> Option<InFile<tree_sitter::Node<'tree>>> {
        match self {
            DefResolution::Function(func) => func.source(db, tree),
            DefResolution::EnumStruct(enum_struct) => enum_struct.source(db, tree),
            DefResolution::Field(field) => field.source(db, tree),
            DefResolution::Global(global) => global.source(db, tree),
            DefResolution::Local(local) => local.source(db, tree)?.source(db, tree),
        }
    }
}

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
        self.declarations(db)
            .iter()
            .for_each(|it| acc.extend(it.diagnostics(db)));
    }
}

/// The defs which can be visible in the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileDef {
    Function(Function),
    EnumStruct(EnumStruct),
    Global(Global),
}

impl_from!(Function, EnumStruct, Global for FileDef);

impl FileDef {
    pub fn diagnostics(self, db: &dyn HirDatabase) -> Vec<AnyDiagnostic> {
        let id: FileDefId = match self {
            FileDef::Function(it) => it.id.into(),
            FileDef::EnumStruct(it) => it.id.into(),
            FileDef::Global(it) => it.id.into(),
        };

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
            FileDef::EnumStruct(_) | FileDef::Global(_) => None,
        }
    }
}

/// The defs which have a body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefWithBody {
    Function(Function),
}
impl_from!(Function for DefWithBody);

impl DefWithBody {
    pub fn name(self, db: &dyn HirDatabase) -> Option<Name> {
        match self {
            DefWithBody::Function(f) => Some(f.name(db)),
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
                } => {
                    let expr = expr_syntax(*expr);

                    acc.push(
                        UnresolvedField {
                            expr,
                            name: name.clone(),
                            receiver: receiver.clone(),
                            method_with_same_name_exists: *method_with_same_name_exists,
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
pub struct EnumStruct {
    pub(crate) id: EnumStructId,
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
