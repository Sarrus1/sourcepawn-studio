mod item;

use base_db::FilePosition;
use hir::{db, DefResolution, Semantics};
use hir_def::{resolver::HasResolver, Name};
use ide_db::{RootDatabase, SymbolKind};
pub use item::CompletionItem;
use itertools::Itertools;
use smol_str::{SmolStr, ToSmolStr};

pub fn completions(
    db: &RootDatabase,
    pos: FilePosition,
    trigger_character: Option<char>,
) -> Option<Vec<CompletionItem>> {
    let sema = &Semantics::new(db);

    let res = sema
        .defs_in_scope(pos.file_id)
        .into_iter()
        .flat_map(|it| {
            let out: (Option<Name>, SymbolKind) = match it {
                DefResolution::Function(it) => (it.name(db).into(), it.kind(db).into()),
                DefResolution::Macro(it) => (it.name(db).into(), SymbolKind::Macro),
                DefResolution::EnumStruct(it) => (it.name(db).into(), SymbolKind::Struct),
                DefResolution::Methodmap(it) => (it.name(db).into(), SymbolKind::Methodmap),
                DefResolution::Property(it) => (it.name(db).into(), SymbolKind::Property),
                DefResolution::Enum(it) => (it.name(db).into(), SymbolKind::Enum),
                DefResolution::Variant(it) => (it.name(db).into(), SymbolKind::Variant),
                DefResolution::Typedef(it) => (it.name(db), SymbolKind::Typedef),
                DefResolution::Typeset(it) => (it.name(db).into(), SymbolKind::Typeset),
                DefResolution::Functag(it) => (it.name(db), SymbolKind::Functag),
                DefResolution::Funcenum(it) => (it.name(db).into(), SymbolKind::Funcenum),
                DefResolution::Field(it) => (it.name(db).into(), SymbolKind::Field),
                DefResolution::Global(it) => (it.name(db).into(), SymbolKind::Global),
                DefResolution::Local(it) => (it.name(db), SymbolKind::Local),
                DefResolution::File(_) => unreachable!(),
            };
            let label = out.0?.to_smolstr();
            Some(CompletionItem {
                label,
                kind: out.1,
                detail: None,
                documentation: None,
                deprecated: false,
                trigger_call_info: false,
            })
        })
        .collect_vec();

    res.into()
}
