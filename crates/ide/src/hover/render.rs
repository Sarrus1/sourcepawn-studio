use hir::DefResolution;
use ide_db::RootDatabase;
use vfs::FileId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Render {
    FileId(FileId),
    String(String),
}

impl From<FileId> for Render {
    fn from(file_id: FileId) -> Self {
        Render::FileId(file_id)
    }
}

impl From<String> for Render {
    fn from(string: String) -> Self {
        Render::String(string)
    }
}

pub fn render_def(db: &RootDatabase, def: DefResolution) -> Option<Render> {
    Some(match def {
        DefResolution::Function(it) => it.render(db)?.into(),
        DefResolution::Macro(it) => it.render(db)?.into(),
        DefResolution::EnumStruct(it) => it.render(db)?.into(),
        DefResolution::Methodmap(it) => it.render(db)?.into(),
        DefResolution::Property(it) => it.render(db)?.into(),
        DefResolution::Enum(it) => it.render(db)?.into(),
        DefResolution::Variant(it) => it.render(db)?.into(),
        DefResolution::Field(it) => it.render(db)?.into(),
        DefResolution::Typedef(it) => it.render(db)?.into(),
        DefResolution::Typeset(it) => it.render(db)?.into(),
        DefResolution::Functag(it) => it.render(db)?.into(),
        DefResolution::Funcenum(it) => it.render(db)?.into(),
        DefResolution::Global(it) => it.render(db)?.into(),
        DefResolution::Local(it) => it.render(db)?.into(),
        DefResolution::File(it) => it.file_id().into(),
    })
}
