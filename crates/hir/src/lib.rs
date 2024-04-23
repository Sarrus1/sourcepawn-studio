use base_db::Tree;
use db::HirDatabase;
use hir_def::{
    resolver::{HasResolver, ValueNs},
    DefDiagnostic, DefWithBodyId, EnumId, EnumStructId, ExprId, FuncenumId, FunctagId, FunctionId,
    FunctionKind, GlobalId, InFile, InferenceDiagnostic, ItemContainerId, LocalFieldId, Lookup,
    MacroId, MethodmapExtension, MethodmapId, Name, NodePtr, PropertyId, SpecialMethod, TypedefId,
    TypesetId, VariantId,
};
use preprocessor::PreprocessorError;
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
    Typeset(Typeset),
    Functag(Functag),
    Funcenum(Funcenum),
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
    Typedef,
    Typeset,
    Functag,
    Funcenum,
    File for DefResolution
);

impl DefResolution {
    fn try_from(value: ValueNs) -> Option<Self> {
        match value {
            ValueNs::FunctionId(ids) => {
                DefResolution::Function(Function::from(ids.first()?.value)).into()
            }
            ValueNs::LocalId(expr) => DefResolution::Local(Local::from(expr)).into(),
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
            DefResolution::Property(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Enum(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Variant(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Typedef(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Typeset(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Functag(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Funcenum(it) => it.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Field(it) => it.parent.id.lookup(db.upcast()).id.file_id(),
            DefResolution::Global(it) => it.id.lookup(db.upcast()).file_id(),
            DefResolution::Local(it) => it.parent.file_id(db.upcast()),
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
            DefResolution::Local(it) => it.name(db),
            DefResolution::File(_) => None,
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
                    for diag in db.methodmap_data_with_diagnostics(it.id).1.iter() {
                        match diag {
                            DefDiagnostic::UnresolvedInherit {
                                methodmap_ast_id,
                                inherit_name,
                                exists,
                            } => {
                                let file_id = it.id.lookup(db.upcast()).id.file_id();
                                let tree = db.parse(file_id);
                                let ast_id_map = db.ast_id_map(file_id);
                                let methodmap_node = ast_id_map[*methodmap_ast_id].to_node(&tree);
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
                FileDef::EnumStruct(_) => (),
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
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub fn type_def(self, db: &dyn HirDatabase) -> Option<DefResolution> {
        let ty = db.function_data(self.id).type_ref.clone()?;
        let ty_str = ty.type_as_string();
        self.id
            .resolver(db.upcast())
            .resolve_ident(&ty_str)
            .and_then(DefResolution::try_from)
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
        let node = self.source(db, &tree)?;
        let source = db.preprocessed_text(file_id);

        node.value
            .utf8_text(source.as_bytes())
            .ok()
            .map(String::from)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumStruct {
    pub(crate) id: EnumStructId,
}

impl EnumStruct {
    pub fn name(self, db: &dyn HirDatabase) -> Name {
        db.enum_struct_data(self.id).name.clone()
    }

    pub fn render(self, db: &dyn HirDatabase) -> Option<String> {
        format!("enum struct {}", self.name(db)).into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Methodmap {
    pub(crate) id: MethodmapId,
}

impl Methodmap {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Typedef {
    pub(crate) id: TypedefId,
}

impl Typedef {
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
            let ItemContainerId::TypesetId(parent_id) = self.id.lookup(db.upcast()).container
            else {
                panic!("expected a typedef to have a typeset as a parent");
            };
            let parent_name = db.typeset_data(parent_id).name.to_string();
            buf.push_str(&parent_name);
            buf.push('\n');
        }
        buf.push_str("function ");
        buf.push_str(&data.type_ref.to_string());
        buf.push(' ');

        if let Some(params) = node
            .value
            .children(&mut node.value.walk())
            .find(|n| TSKind::from(n) == TSKind::typedef_expression)
            .expect("expected a typedef to have a typedef_expression")
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Field {
    pub(crate) parent: EnumStruct,
    pub(crate) id: LocalFieldId,
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

    pub fn type_ref(self, db: &dyn HirDatabase) -> String {
        let parent_data = db.enum_struct_data(self.parent.id);
        parent_data
            .field(self.id)
            .expect("expected a field to have a type")
            .type_ref
            .to_string()
    }

    pub fn render(self, db: &dyn HirDatabase) -> Option<String> {
        let parent_data = db.enum_struct_data(self.parent.id);

        format!(
            "{} {}::{};",
            self.type_ref(db),
            parent_data.name,
            self.name(db)
        )
        .into()
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
            _ => None,
        }
    }
}

pub struct LocalSource<'tree> {
    pub local: Local,
    pub source: InFile<tree_sitter::Node<'tree>>,
}
