use la_arena::Idx;

use crate::db::DefDatabase;

use super::{
    Enum, EnumStruct, EnumStructItemId, Field, FileItem, Funcenum, Functag, Function, FunctionKind,
    ItemTree, Macro, Methodmap, MethodmapItemId, Property, RawVisibilityId, Typedef, Typeset,
    Variable, Variant,
};

pub fn print_item_tree(_db: &dyn DefDatabase, tree: &ItemTree) -> String {
    let mut printer = Printer::new(tree);
    for item in tree.top_level_items() {
        match item {
            FileItem::Function(idx) => printer.print_function(idx),
            FileItem::Variable(idx) => printer.print_variable(idx),
            FileItem::EnumStruct(idx) => printer.print_enum_struct(idx),
            FileItem::Enum(idx) => printer.print_enum(idx),
            FileItem::Macro(idx) => printer.print_macro(idx),
            FileItem::Methodmap(idx) => printer.print_methodmap(idx),
            FileItem::Typedef(idx) => printer.print_typedef(idx),
            FileItem::Typeset(idx) => printer.print_typeset(idx),
            FileItem::Functag(idx) => printer.print_functag(idx),
            FileItem::Funcenum(idx) => printer.print_funcenum(idx),
            FileItem::Variant(_) => (),
        }
        printer.newline();
    }

    printer.into_string()
}

struct Printer<'a> {
    buf: String,
    indent_level: usize,
    tree: &'a ItemTree,
}

impl<'a> Printer<'a> {
    fn new(tree: &'a ItemTree) -> Self {
        Self {
            buf: String::new(),
            indent_level: 0,
            tree,
        }
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn dedent(&mut self) {
        self.indent_level -= 1;
    }

    fn newline(&mut self) {
        self.buf.push('\n');
        self.buf.push_str("  ".repeat(self.indent_level).as_str());
    }

    fn push(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    fn into_string(self) -> String {
        self.buf
    }

    pub fn print_macro(&mut self, idx: &Idx<Macro>) {
        let Macro { name, ast_id, .. } = &self.tree[*idx];
        self.push(format!("// {}", ast_id).as_str());
        self.newline();
        self.push(&format!("#define {}", name.0.to_string()));
        self.newline();
    }

    pub fn print_variable(&mut self, idx: &Idx<Variable>) {
        let Variable {
            name,
            visibility,
            type_ref,
            ast_id,
        } = &self.tree[*idx];
        self.push(format!("// {}", ast_id).as_str());
        self.newline();
        if visibility != &RawVisibilityId::NONE {
            self.push(&visibility.to_string());
            self.push(" ");
        }
        if let Some(type_ref) = type_ref {
            self.push(&type_ref.to_str());
            self.push(" ");
        }
        self.push(name.0.as_str());
        self.newline();
    }

    pub fn print_enum(&mut self, idx: &Idx<Enum>) {
        let Enum {
            name,
            variants,
            ast_id,
            ..
        } = &self.tree[*idx];
        self.push(format!("// {}", ast_id).as_str());
        self.newline();
        self.push(&format!("enum {} {{", name.0));
        self.indent();
        self.newline();
        for variant in self.tree.data().variants[variants.clone()].iter() {
            let Variant { name, ast_id, .. } = variant;
            self.push(format!("// {}", ast_id).as_str());
            self.newline();
            self.push(&format!("{},", name.0));
            self.newline();
        }
        self.dedent();
        self.push("};");
        self.newline();
    }

    pub fn print_enum_struct(&mut self, idx: &Idx<EnumStruct>) {
        let EnumStruct {
            name,
            items,
            ast_id,
            ..
        } = &self.tree[*idx];
        self.push(format!("// {}", ast_id).as_str());
        self.newline();
        self.push(&format!("enum struct {} {{", name.0));
        self.indent();
        self.newline();
        for item_idx in items.iter() {
            match item_idx {
                EnumStructItemId::Field(field_idx) => {
                    let Field {
                        name,
                        type_ref,
                        ast_id,
                    } = &self.tree[*field_idx];
                    self.push(format!("// {}", ast_id).as_str());
                    self.newline();
                    self.push(&format!("{} {};", type_ref.to_str(), name.0));
                    self.newline();
                }
                EnumStructItemId::Method(method_idx) => self.print_function(method_idx),
            }
        }
        self.dedent();
        self.newline();
        self.push("}");
        self.newline();
    }

    pub fn print_methodmap(&mut self, idx: &Idx<Methodmap>) {
        let Methodmap {
            name,
            items,
            inherits,
            ast_id,
            ..
        } = &self.tree[*idx];
        self.push(format!("// {}", ast_id).as_str());
        self.newline();
        self.push(&format!("methodmap {}", name.0));
        if let Some(inherits) = inherits {
            self.push(&format!(" < {}", inherits.0));
        }
        self.push(" {");
        self.indent();
        self.newline();
        for item_idx in items.iter() {
            match item_idx {
                &MethodmapItemId::Property(property_idx) => {
                    let Property {
                        name,
                        type_ref,
                        getters_setters,
                        ast_id,
                    } = &self.tree[property_idx];
                    self.push(format!("// {}", ast_id).as_str());
                    self.newline();
                    self.push(&format!("property {} {} {{", type_ref.to_str(), name.0));
                    self.indent();
                    self.newline();
                    for fn_idx in getters_setters.clone() {
                        self.print_function(&fn_idx);
                    }
                    self.dedent();
                    self.push("}");
                    self.newline();
                }
                MethodmapItemId::Method(method_idx) => self.print_function(method_idx),
            }
        }
        self.dedent();
        self.newline();
        self.push("}");
        self.newline();
    }

    pub fn print_function(&mut self, idx: &Idx<Function>) {
        let Function {
            name,
            kind,
            visibility,
            ret_type,
            params,
            ast_id,
            ..
        } = &self.tree[*idx];
        self.push(format!("// {}", ast_id).as_str());
        self.newline();
        if visibility != &RawVisibilityId::NONE {
            self.push(&visibility.to_string());
            self.push(" ");
        }
        match kind {
            FunctionKind::Forward => self.push("forward "),
            FunctionKind::Native => self.push("native "),
            _ => (),
        }
        if let Some(ret_type) = ret_type {
            self.push(&ret_type.to_str());
            self.push(" ");
        }
        self.push(&name.0);
        self.push("(");
        self.indent();
        for param in self.tree.data().params[params.clone()].iter() {
            if let Some(type_ref) = &param.type_ref {
                self.newline();
                self.push(format!("// {}", param.ast_id).as_str());
                self.newline();
                self.push(&type_ref.to_str());
                self.push(",");
            }
        }
        if kind == &FunctionKind::Def {
            self.push(") {");
            self.newline();
            self.push("/* body */");
            self.dedent();
            self.newline();
            self.push("}");
        } else {
            self.push(");");
            self.dedent();
        }
        self.newline();
    }

    pub fn print_typedef(&mut self, idx: &Idx<Typedef>) {
        let Typedef {
            name,
            type_ref,
            params,
            ast_id,
        } = &self.tree[*idx];
        self.push(format!("// {}", ast_id).as_str());
        self.newline();
        if let Some(name) = name {
            self.push(format!("typedef {} = ", name).as_str());
        }
        self.push(format!("function {}", type_ref.to_str()).as_str());
        self.push("(");
        self.indent();
        for param in self.tree.data().params[params.clone()].iter() {
            if let Some(type_ref) = &param.type_ref {
                self.newline();
                self.push(format!("// {}", param.ast_id).as_str());
                self.newline();
                self.push(&type_ref.to_str());
                self.push(",");
            }
        }
        self.push(");");
        self.dedent();
        self.newline();
    }

    pub fn print_typeset(&mut self, idx: &Idx<Typeset>) {
        let Typeset {
            name,
            typedefs,
            ast_id,
        } = &self.tree[*idx];
        self.push(format!("// {}", ast_id).as_str());
        self.newline();
        self.push(format!("typeset {}", name).as_str());
        self.newline();
        self.indent();
        for typedef in typedefs.clone() {
            self.print_typedef(&typedef);
        }
        self.dedent();
        self.push("};");
        self.newline();
    }

    pub fn print_functag(&mut self, idx: &Idx<Functag>) {
        let Functag {
            name,
            type_ref,
            params,
            ast_id,
        } = &self.tree[*idx];
        self.push(format!("// {}", ast_id).as_str());
        self.newline();
        if let Some(name) = name {
            self.push(
                format!(
                    "functag public {}:{}",
                    type_ref.as_ref().map(|e| e.to_str()).unwrap_or_default(),
                    name
                )
                .as_str(),
            );
        } else {
            self.push(
                format!(
                    "{}:public",
                    type_ref.as_ref().map(|e| e.to_str()).unwrap_or_default(),
                )
                .as_str(),
            );
        }
        self.push("(");
        self.indent();
        for param in self.tree.data().params[params.clone()].iter() {
            if let Some(type_ref) = &param.type_ref {
                self.newline();
                self.push(format!("// {}", param.ast_id).as_str());
                self.newline();
                self.push(&type_ref.to_str());
                self.push(",");
            }
        }
        self.push(");");
        self.dedent();
        self.newline();
    }

    pub fn print_funcenum(&mut self, idx: &Idx<Funcenum>) {
        let Funcenum {
            name,
            functags,
            ast_id,
        } = &self.tree[*idx];
        self.push(format!("// {}", ast_id).as_str());
        self.newline();
        self.push(format!("funcenum {}", name).as_str());
        self.newline();
        self.indent();
        for functag in functags.clone() {
            self.print_functag(&functag);
        }
        self.dedent();
        self.push("};");
        self.newline();
    }
}
