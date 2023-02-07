use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use lsp_types::{
    CompletionItem, CompletionParams, DocumentSymbol, GotoDefinitionParams, Hover, HoverParams,
    LocationLink, Position, Range, SignatureInformation, Url,
};

use crate::{
    document::Document,
    providers::hover::description::Description,
    store::Store,
    utils::{range_contains_pos, range_equals_range},
};

use self::parameters::Parameter;

pub(crate) mod define_item;
pub(crate) mod enum_item;
pub(crate) mod enum_member_item;
pub(crate) mod enum_struct_item;
pub(crate) mod function_item;
pub(crate) mod include_item;
pub(crate) mod methodmap_item;
pub(crate) mod parameters;
pub(crate) mod property_item;
pub(crate) mod typedef_item;
pub(crate) mod variable_item;

/// Represents a location inside a resource, such as a line inside a text file.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Location {
    pub uri: Arc<Url>,
    pub range: Range,
}

impl Location {
    pub fn to_lsp_location(&self) -> lsp_types::Location {
        lsp_types::Location {
            uri: self.uri.as_ref().clone(),
            range: self.range,
        }
    }
}

#[derive(Debug, Clone)]
/// Generic representation of an item, which can be converted to a
/// [CompletionItem](lsp_types::CompletionItem), [Location](lsp_types::Location), etc.
pub enum SPItem {
    Function(function_item::FunctionItem),
    Variable(variable_item::VariableItem),
    Enum(enum_item::EnumItem),
    EnumMember(enum_member_item::EnumMemberItem),
    EnumStruct(enum_struct_item::EnumStructItem),
    Define(define_item::DefineItem),
    Methodmap(methodmap_item::MethodmapItem),
    Property(property_item::PropertyItem),
    Include(include_item::IncludeItem),
    Typedef(typedef_item::TypedefItem),
}

pub fn get_all_items(store: &Store, flat: bool) -> Vec<Arc<RwLock<SPItem>>> {
    let mut all_items = vec![];
    if let Some(main_path_uri) = store.environment.options.get_main_path_uri() {
        let mut includes: HashSet<Url> = HashSet::new();
        includes.insert(main_path_uri.clone());
        if let Some(document) = store.documents.get(&main_path_uri) {
            get_included_files(store, document, &mut includes);
            for include in includes.iter() {
                if let Some(document) = store.documents.get(include) {
                    if flat {
                        all_items.extend(document.sp_items_flat());
                    } else {
                        all_items.extend(document.sp_items())
                    }
                }
            }
        }
        return all_items;
    }
    for document in store.documents.values() {
        for item in document.sp_items.iter() {
            all_items.push(item.clone());
        }
    }

    all_items
}

fn get_included_files(store: &Store, document: &Document, includes: &mut HashSet<Url>) {
    for include_uri in document.includes.iter() {
        if includes.contains(include_uri) {
            continue;
        }
        includes.insert(include_uri.clone());
        if let Some(include_document) = store.get(include_uri) {
            get_included_files(store, &include_document, includes);
        }
    }
}

pub fn get_items_from_position(
    store: &Store,
    position: Position,
    uri: Url,
) -> Vec<Arc<RwLock<SPItem>>> {
    let uri = Arc::new(uri);
    let all_items = get_all_items(store, true);
    let mut res = vec![];
    for item in all_items.iter() {
        let item_lock = item.read().unwrap();
        match item_lock.range() {
            Some(range) => {
                if range_contains_pos(range, position) && item_lock.uri().as_ref().eq(&uri) {
                    res.push(item.clone());
                    continue;
                }
            }
            None => {
                continue;
            }
        }
        match item_lock.references() {
            Some(references) => {
                for reference in references.iter() {
                    if range_contains_pos(reference.range, position) && reference.uri.eq(&uri) {
                        res.push(item.clone());
                        break;
                    }
                }
            }
            None => {
                continue;
            }
        }
    }
    res
}

impl SPItem {
    pub fn range(&self) -> Option<Range> {
        match self {
            SPItem::Variable(item) => Some(item.range),
            SPItem::Function(item) => Some(item.range),
            SPItem::Enum(item) => Some(item.range),
            SPItem::EnumMember(item) => Some(item.range),
            SPItem::EnumStruct(item) => Some(item.range),
            SPItem::Define(item) => Some(item.range),
            SPItem::Methodmap(item) => Some(item.range),
            SPItem::Property(item) => Some(item.range),
            SPItem::Typedef(item) => Some(item.range),
            SPItem::Include(item) => Some(item.range),
        }
    }

    pub fn full_range(&self) -> Option<Range> {
        match self {
            SPItem::Variable(item) => Some(item.range),
            SPItem::Function(item) => Some(item.full_range),
            SPItem::Enum(item) => Some(item.full_range),
            SPItem::EnumMember(item) => Some(item.range),
            SPItem::EnumStruct(item) => Some(item.full_range),
            SPItem::Define(item) => Some(item.full_range),
            SPItem::Methodmap(item) => Some(item.full_range),
            SPItem::Property(item) => Some(item.full_range),
            SPItem::Typedef(item) => Some(item.full_range),
            SPItem::Include(item) => Some(item.range),
        }
    }

    pub fn name(&self) -> String {
        match self {
            SPItem::Variable(item) => item.name.clone(),
            SPItem::Function(item) => item.name.clone(),
            SPItem::Enum(item) => item.name.clone(),
            SPItem::EnumMember(item) => item.name.clone(),
            SPItem::EnumStruct(item) => item.name.clone(),
            SPItem::Define(item) => item.name.clone(),
            SPItem::Methodmap(item) => item.name.clone(),
            SPItem::Property(item) => item.name.clone(),
            SPItem::Typedef(item) => item.name.clone(),
            SPItem::Include(item) => item.name.clone(),
        }
    }

    pub fn parent(&self) -> Option<Arc<RwLock<SPItem>>> {
        match self {
            SPItem::Variable(item) => item.parent.clone().map(|parent| parent.upgrade().unwrap()),
            SPItem::Function(item) => item.parent.clone().map(|parent| parent.upgrade().unwrap()),
            SPItem::Enum(_) => None,
            SPItem::EnumMember(item) => Some(item.parent.upgrade().unwrap()),
            SPItem::EnumStruct(_) => None,
            SPItem::Define(_) => None,
            SPItem::Methodmap(_) => None,
            SPItem::Property(item) => Some(item.parent.upgrade().unwrap()),
            SPItem::Typedef(_) => None,
            SPItem::Include(_) => None,
        }
    }

    pub fn type_(&self) -> String {
        match self {
            SPItem::Variable(item) => item.type_.clone(),
            SPItem::Function(item) => item.type_.clone(),
            SPItem::Enum(item) => item.name.clone(),
            SPItem::EnumMember(item) => item.parent.upgrade().unwrap().read().unwrap().name(),
            SPItem::EnumStruct(item) => item.name.clone(),
            SPItem::Define(_) => "".to_string(),
            SPItem::Methodmap(item) => item.name.clone(),
            SPItem::Property(item) => item.type_.clone(),
            SPItem::Typedef(item) => item.type_.clone(),
            SPItem::Include(_) => "".to_string(),
        }
    }

    pub fn description(&self) -> Option<Description> {
        match self {
            SPItem::Variable(item) => Some(item.description.clone()),
            SPItem::Function(item) => Some(item.description.clone()),
            SPItem::Enum(item) => Some(item.description.clone()),
            SPItem::EnumMember(item) => Some(item.description.clone()),
            SPItem::EnumStruct(item) => Some(item.description.clone()),
            SPItem::Define(item) => Some(item.description.clone()),
            SPItem::Methodmap(item) => Some(item.description.clone()),
            SPItem::Property(item) => Some(item.description.clone()),
            SPItem::Typedef(item) => Some(item.description.clone()),
            SPItem::Include(_) => None,
        }
    }

    pub fn uri(&self) -> Arc<Url> {
        match self {
            SPItem::Variable(item) => item.uri.clone(),
            SPItem::Function(item) => item.uri.clone(),
            SPItem::Enum(item) => item.uri.clone(),
            SPItem::EnumMember(item) => item.uri.clone(),
            SPItem::EnumStruct(item) => item.uri.clone(),
            SPItem::Define(item) => item.uri.clone(),
            SPItem::Methodmap(item) => item.uri.clone(),
            SPItem::Property(item) => item.uri.clone(),
            SPItem::Typedef(item) => item.uri.clone(),
            SPItem::Include(item) => item.uri.clone(),
        }
    }

    pub fn references(&self) -> Option<&Vec<Location>> {
        match self {
            SPItem::Variable(item) => Some(&item.references),
            SPItem::Function(item) => Some(&item.references),
            SPItem::Enum(item) => Some(&item.references),
            SPItem::EnumMember(item) => Some(&item.references),
            SPItem::EnumStruct(item) => Some(&item.references),
            SPItem::Define(item) => Some(&item.references),
            SPItem::Methodmap(item) => Some(&item.references),
            SPItem::Property(item) => Some(&item.references),
            SPItem::Typedef(item) => Some(&item.references),
            SPItem::Include(_) => None,
        }
    }

    pub fn children(&self) -> Option<&Vec<Arc<RwLock<SPItem>>>> {
        match self {
            SPItem::Function(item) => Some(&item.children),
            SPItem::Enum(item) => Some(&item.children),
            SPItem::EnumStruct(item) => Some(&item.children),
            SPItem::Methodmap(item) => Some(&item.children),
            _ => None,
        }
    }

    pub fn push_reference(&mut self, reference: Location) {
        if range_equals_range(&self.range().unwrap(), &reference.range)
            && self.uri().eq(&reference.uri)
        {
            return;
        }
        match self {
            SPItem::Variable(item) => item.references.push(reference),
            SPItem::Function(item) => item.references.push(reference),
            SPItem::Enum(item) => item.references.push(reference),
            SPItem::EnumMember(item) => item.references.push(reference),
            SPItem::EnumStruct(item) => item.references.push(reference),
            SPItem::Define(item) => item.references.push(reference),
            SPItem::Methodmap(item) => item.references.push(reference),
            SPItem::Property(item) => item.references.push(reference),
            SPItem::Typedef(item) => item.references.push(reference),
            SPItem::Include(_) => {}
        }
    }

    pub fn push_child(&mut self, child: Arc<RwLock<SPItem>>) {
        match self {
            SPItem::Function(item) => item.children.push(child),
            SPItem::Enum(item) => item.children.push(child),
            SPItem::EnumStruct(item) => item.children.push(child),
            SPItem::Methodmap(item) => item.children.push(child),
            _ => {}
        }
    }

    pub fn set_new_references(&mut self, references: Vec<Location>) {
        match self {
            SPItem::Variable(item) => item.references = references,
            SPItem::Function(item) => item.references = references,
            SPItem::Enum(item) => item.references = references,
            SPItem::EnumMember(item) => item.references = references,
            SPItem::EnumStruct(item) => item.references = references,
            SPItem::Define(item) => item.references = references,
            SPItem::Methodmap(item) => item.references = references,
            SPItem::Property(item) => item.references = references,
            SPItem::Typedef(item) => item.references = references,
            SPItem::Include(_) => {}
        }
    }

    pub fn push_param(&mut self, param: Arc<RwLock<Parameter>>) {
        match self {
            SPItem::Typedef(item) => item.params.push(param),
            SPItem::Function(item) => item.params.push(param),
            _ => {
                eprintln!("Can only push type params to functions and typedefs.")
            }
        }
    }

    pub fn set_parent(&mut self, parent: Arc<RwLock<SPItem>>) {
        match self {
            SPItem::Methodmap(item) => {
                item.parent = Some(parent);
                item.tmp_parent = None
            }
            _ => {
                eprintln!("Cannot set the methodmap inherits of an item that is not a methodmap.")
            }
        }
    }

    pub fn to_completions(
        &self,
        params: &CompletionParams,
        request_method: bool,
    ) -> Vec<CompletionItem> {
        match self {
            SPItem::Variable(item) => {
                let mut res = vec![];
                if let Some(completion) = item.to_completion(params, request_method) {
                    res.push(completion)
                }
                res
            }
            SPItem::Function(item) => item.to_completions(params, request_method),
            SPItem::Enum(item) => item.to_completions(params, request_method),
            SPItem::EnumMember(item) => {
                let mut res = vec![];
                if let Some(completion) = item.to_completion(params) {
                    res.push(completion)
                }
                res
            }
            SPItem::EnumStruct(item) => item.to_completions(params, request_method),
            SPItem::Define(item) => {
                let mut res = vec![];
                if let Some(completion) = item.to_completion(params) {
                    res.push(completion)
                }
                res
            }
            SPItem::Methodmap(item) => item.to_completions(params, request_method),
            SPItem::Property(item) => {
                let mut res = vec![];
                if let Some(completion) = item.to_completion(params, request_method) {
                    res.push(completion)
                }
                res
            }
            SPItem::Typedef(item) => {
                let mut res = vec![];
                if let Some(completion) = item.to_completion(params) {
                    res.push(completion)
                }
                res
            }
            SPItem::Include(item) => {
                let mut res = vec![];
                if let Some(completion) = item.to_completion(params) {
                    res.push(completion)
                }
                res
            }
        }
    }

    pub fn to_hover(&self, params: &HoverParams) -> Option<Hover> {
        match self {
            SPItem::Variable(item) => item.to_hover(params),
            SPItem::Function(item) => item.to_hover(params),
            SPItem::Enum(item) => item.to_hover(params),
            SPItem::EnumMember(item) => item.to_hover(params),
            SPItem::EnumStruct(item) => item.to_hover(params),
            SPItem::Define(item) => item.to_hover(params),
            SPItem::Methodmap(item) => item.to_hover(params),
            SPItem::Property(item) => item.to_hover(params),
            SPItem::Typedef(item) => item.to_hover(params),
            SPItem::Include(item) => item.to_hover(params),
        }
    }

    pub fn to_definition(&self, params: &GotoDefinitionParams) -> Option<LocationLink> {
        match self {
            SPItem::Variable(item) => item.to_definition(params),
            SPItem::Function(item) => item.to_definition(params),
            SPItem::Enum(item) => item.to_definition(params),
            SPItem::EnumMember(item) => item.to_definition(params),
            SPItem::EnumStruct(item) => item.to_definition(params),
            SPItem::Define(item) => item.to_definition(params),
            SPItem::Methodmap(item) => item.to_definition(params),
            SPItem::Property(item) => item.to_definition(params),
            SPItem::Typedef(item) => item.to_definition(params),
            SPItem::Include(item) => item.to_definition(params),
        }
    }

    pub fn to_document_symbol(&self) -> Option<DocumentSymbol> {
        match self {
            SPItem::Variable(item) => item.to_document_symbol(),
            SPItem::Function(item) => item.to_document_symbol(),
            SPItem::Define(item) => item.to_document_symbol(),
            SPItem::Enum(item) => item.to_document_symbol(),
            SPItem::EnumMember(item) => item.to_document_symbol(),
            SPItem::EnumStruct(item) => item.to_document_symbol(),
            SPItem::Methodmap(item) => item.to_document_symbol(),
            SPItem::Typedef(item) => item.to_document_symbol(),
            SPItem::Property(item) => item.to_document_symbol(),
            SPItem::Include(_) => None,
        }
    }

    pub fn to_signature_help(&self, parameter_count: u32) -> Option<SignatureInformation> {
        match self {
            SPItem::Function(item) => item.to_signature_help(parameter_count),
            _ => None,
        }
    }
}
