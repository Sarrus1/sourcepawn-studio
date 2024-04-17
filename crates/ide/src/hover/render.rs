use hir::DefResolution;
use ide_db::RootDatabase;

pub fn render_def(db: &RootDatabase, def: DefResolution) -> String {
    match def {
        DefResolution::Function(it) => it.render(db),
        DefResolution::EnumStruct(it) => it.render(db),
        DefResolution::Field(it) => it.render(db),
        _ => todo!(),
    }
}
