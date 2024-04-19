use hir::DefResolution;
use ide_db::RootDatabase;

use super::HoverAction;

pub fn goto_type_action_for_def(db: &RootDatabase, def: DefResolution) -> Option<HoverAction> {
    let ty = match def {
        DefResolution::Function(it) => it.type_def(db),
        _ => todo!(),
    }?;

    Some(HoverAction::goto_type_from_targets(db, vec![ty]))
}
