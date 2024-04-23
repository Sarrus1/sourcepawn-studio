use hir::DefResolution;
use ide_db::RootDatabase;

use super::HoverAction;

pub fn goto_type_action_for_def(db: &RootDatabase, def: DefResolution) -> Option<HoverAction> {
    let res = match def {
        DefResolution::Function(it) => it.type_def(db),
        DefResolution::Methodmap(it) => it.type_def(db),
        DefResolution::Property(it) => it.type_def(db),
        DefResolution::Typedef(it) => it.type_def(db),
        DefResolution::Functag(it) => it.type_def(db),
        DefResolution::Field(it) => it.type_def(db),
        DefResolution::Global(it) => it.type_def(db),
        DefResolution::Local(it) => it.type_def(db),
        DefResolution::Variant(it) => it.type_def(db),
        DefResolution::Macro(_)
        | DefResolution::Enum(_)
        | DefResolution::Typeset(_)
        | DefResolution::EnumStruct(_)
        | DefResolution::Funcenum(_)
        | DefResolution::File(_) => vec![],
    };

    Some(HoverAction::goto_type_from_targets(db, res))
}
