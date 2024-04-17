use hir::DefResolution;
use ide_db::RootDatabase;

pub fn render_def(db: &RootDatabase, def: DefResolution) -> Option<String> {
    match def {
        DefResolution::Function(it) => it.render(db),
        DefResolution::Macro(it) => it.render(db),
        DefResolution::EnumStruct(it) => it.render(db),
        DefResolution::Methodmap(it) => it.render(db),
        DefResolution::Property(it) => it.render(db),
        DefResolution::Enum(it) => it.render(db),
        DefResolution::Variant(it) => it.render(db),
        DefResolution::Field(it) => it.render(db),
        _ => todo!(),
    }
}
