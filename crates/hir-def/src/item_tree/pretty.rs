use crate::db::DefDatabase;

use super::{EnumStruct, Field, FileItem, Function, ItemTree, Variable};

pub(super) fn print_item_tree(_db: &dyn DefDatabase, tree: &ItemTree) -> String {
    let mut buf: Vec<String> = vec![];
    for item in tree.top_level_items() {
        match item {
            FileItem::Function(idx) => {
                let Function { name, .. } = &tree[*idx];
                buf.push(name.0.to_string());
                buf.push("\n".to_string())
            }
            FileItem::Variable(idx) => {
                let Variable { name, .. } = &tree[*idx];
                buf.push(name.0.to_string());
                buf.push("\n".to_string())
            }
            FileItem::EnumStruct(idx) => {
                let EnumStruct { name, fields, .. } = &tree[*idx];
                buf.push(format!("{} {{", name.0.to_string()));
                for field_idx in fields.clone() {
                    let Field { name, type_ref, .. } = &tree[field_idx];
                    buf.push(format!("  {} {};", type_ref.to_str(), name.0));
                }
                buf.push("}\n".to_string());
            }
        }
    }

    buf.join("\n")
}
