use line_index::TextRange;
use nohash_hasher::IntMap;
use vfs::FileId;

#[derive(Default, Debug, Clone)]
pub struct SourceChange {
    pub source_file_edits: IntMap<FileId, Vec<TextEdit>>,
}

impl SourceChange {
    pub fn insert(&mut self, file_id: FileId, edit: TextEdit) {
        self.source_file_edits
            .entry(file_id)
            .or_default()
            .push(edit);
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextEdit {
    range: TextRange,
    replacement_text: String,
}

impl TextEdit {
    pub fn new(range: TextRange, replacement_text: String) -> Self {
        Self {
            range,
            replacement_text,
        }
    }
}
