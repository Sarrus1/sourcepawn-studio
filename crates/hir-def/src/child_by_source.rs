use vfs::FileId;

use crate::{
    db::DefMap, dyn_map::DynMap, hir::Expr, DefDatabase, DefWithBodyId, FieldId, FileItem,
};

pub trait ChildBySource {
    fn child_by_source(&self, db: &dyn DefDatabase, file_id: FileId) -> DynMap {
        let mut res = DynMap::default();
        self.child_by_source_to(db, &mut res, file_id);
        res
    }
    fn child_by_source_to(&self, db: &dyn DefDatabase, map: &mut DynMap, file_id: FileId);
}

// impl ChildBySource for FileId {
//     fn child_by_source_to(&self, db: &dyn DefDatabase, res: &mut DynMap, file_id: FileId) {
//         let def_map = db.file_def_map(file_id);
//         let item_tree = db.file_item_tree(file_id);
//         for item in item_tree.top_level_items() {
//             let FileItem::Function(id) = item else {
//                 continue;
//             };

//         }
//         let module_data = &def_map[self.local_id];
//         module_data.scope.child_by_source_to(db, res, file_id);
//     }
// }
