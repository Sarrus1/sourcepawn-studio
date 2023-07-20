use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use fxhash::FxHashSet;
use lsp_types::{
    CompletionItem, CompletionList, CompletionParams, DocumentSymbol, Documentation,
    GotoDefinitionParams, Hover, HoverParams, LocationLink, MarkupContent, Position, Range,
    SignatureInformation, Url,
};

use crate::{
    document::Document,
    providers::hover::description::Description,
    store::Store,
    utils::{range_contains_pos, range_equals_range},
};

use self::parameter::Parameter;

pub(crate) mod define_item;
pub(crate) mod enum_item;
pub(crate) mod enum_member_item;
pub(crate) mod enum_struct_item;
pub(crate) mod function_item;
pub(crate) mod include_item;
pub(crate) mod methodmap_item;
pub(crate) mod parameter;
pub(crate) mod property_item;
pub(crate) mod typedef_item;
pub(crate) mod typeset_item;
pub(crate) mod variable_item;

/// Represents a location inside a resource, such as a line inside a text file.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Location {
    // Uri of the location.
    pub uri: Arc<Url>,

    // Range of the location.
    pub range: Range,

    // User visible range of the location.
    pub v_range: Range,
}

impl Location {
    pub fn to_lsp_location(&self) -> lsp_types::Location {
        lsp_types::Location {
            uri: self.uri.as_ref().clone(),
            range: self.v_range,
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
    Typeset(typeset_item::TypesetItem),
}

impl Store {
    pub fn get_all_items(&self, flat: bool) -> (Vec<Arc<RwLock<SPItem>>>, Duration) {
        log::debug!("Getting all items from store. flat: {}", flat);
        let mut include_duration = Instant::now().elapsed();
        let mut all_items = vec![];
        if let Ok(Some(main_path_uri)) = self.environment.options.get_main_path_uri() {
            let mut includes = FxHashSet::default();
            includes.insert(main_path_uri.clone());
            if let Some(document) = self.documents.get(&main_path_uri) {
                let now = Instant::now();
                self.get_included_files(document, &mut includes);
                include_duration = now.elapsed();
                for include in includes.iter() {
                    if let Some(document) = self.documents.get(include) {
                        if flat {
                            all_items.extend(document.get_sp_items_flat());
                        } else {
                            all_items.extend(document.get_sp_items())
                        }
                    }
                }
            }
            log::trace!("Done getting {} item(s)", all_items.len());
            return (all_items, include_duration);
        }
        for document in self.documents.values() {
            for item in document.sp_items.iter() {
                all_items.push(item.clone());
            }
        }

        log::trace!(
            "Done getting {} item(s) without the main path.",
            all_items.len()
        );
        (all_items, include_duration)
    }

    pub(crate) fn get_included_files(&self, document: &Document, includes: &mut FxHashSet<Url>) {
        for include_uri in document.includes.keys() {
            if includes.contains(include_uri) {
                continue;
            }
            includes.insert(include_uri.clone());
            if let Some(include_document) = self.get(include_uri) {
                self.get_included_files(&include_document, includes);
            }
        }
    }

    pub(crate) fn get_items_from_position(
        &self,
        position: Position,
        uri: Url,
    ) -> Vec<Arc<RwLock<SPItem>>> {
        log::debug!(
            "Getting all items from position {:#?} in file {:#?}.",
            position,
            uri
        );
        let uri = Arc::new(uri);
        let all_items = self.get_all_items(true);
        let mut res = vec![];
        for item in all_items.0.iter() {
            let item_lock = item.read().unwrap();
            if range_contains_pos(item_lock.v_range(), position)
                && item_lock.uri().as_ref().eq(&uri)
            {
                res.push(item.clone());
                continue;
            }
            match item_lock.references() {
                Some(references) => {
                    for reference in references.iter() {
                        if range_contains_pos(reference.v_range, position) && reference.uri.eq(&uri)
                        {
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
        log::trace!("Got {} item(s) from position", res.len());

        res
    }

    pub(crate) fn get_item_from_key(&self, key: String) -> Option<Arc<RwLock<SPItem>>> {
        log::debug!("Getting item from key {:?}.", key);
        let all_items = self.get_all_items(false);
        let sub_keys: Vec<&str> = key.split('-').collect();
        if sub_keys.is_empty() {
            return None;
        }
        let mut current_item: Option<Arc<RwLock<SPItem>>> = None;
        for key in sub_keys {
            current_item = match current_item {
                Some(item) => item.read().unwrap().children().and_then(|children| {
                    children
                        .iter()
                        .find(|child| child.read().unwrap().name() == key)
                        .cloned()
                }),
                None => all_items
                    .0
                    .iter()
                    .find(|item| item.read().unwrap().name() == key)
                    .cloned(),
            };

            if current_item.is_none() {
                log::trace!("Did not find a match from key.");
                return None;
            }
        }
        log::debug!("Got {:#?} from key.", current_item);

        current_item
    }
}

impl SPItem {
    pub fn range(&self) -> Range {
        match self {
            SPItem::Variable(item) => item.range,
            SPItem::Function(item) => item.range,
            SPItem::Enum(item) => item.range,
            SPItem::EnumMember(item) => item.range,
            SPItem::EnumStruct(item) => item.range,
            SPItem::Define(item) => item.range,
            SPItem::Methodmap(item) => item.range,
            SPItem::Property(item) => item.range,
            SPItem::Typedef(item) => item.range,
            SPItem::Typeset(item) => item.range,
            SPItem::Include(item) => item.range,
        }
    }

    pub fn v_range(&self) -> Range {
        match self {
            SPItem::Variable(item) => item.v_range,
            SPItem::Function(item) => item.v_range,
            SPItem::Enum(item) => item.v_range,
            SPItem::EnumMember(item) => item.v_range,
            SPItem::EnumStruct(item) => item.v_range,
            SPItem::Define(item) => item.v_range,
            SPItem::Methodmap(item) => item.v_range,
            SPItem::Property(item) => item.v_range,
            SPItem::Typedef(item) => item.v_range,
            SPItem::Typeset(item) => item.v_range,
            SPItem::Include(item) => item.v_range,
        }
    }

    pub fn full_range(&self) -> Range {
        match self {
            SPItem::Variable(item) => item.range,
            SPItem::Function(item) => item.full_range,
            SPItem::Enum(item) => item.full_range,
            SPItem::EnumMember(item) => item.range,
            SPItem::EnumStruct(item) => item.full_range,
            SPItem::Define(item) => item.full_range,
            SPItem::Methodmap(item) => item.full_range,
            SPItem::Property(item) => item.full_range,
            SPItem::Typedef(item) => item.full_range,
            SPItem::Typeset(item) => item.full_range,
            SPItem::Include(item) => item.range,
        }
    }

    pub fn v_full_range(&self) -> Range {
        match self {
            SPItem::Variable(item) => item.v_range,
            SPItem::Function(item) => item.v_full_range,
            SPItem::Enum(item) => item.v_full_range,
            SPItem::EnumMember(item) => item.v_range,
            SPItem::EnumStruct(item) => item.v_full_range,
            SPItem::Define(item) => item.v_full_range,
            SPItem::Methodmap(item) => item.v_full_range,
            SPItem::Property(item) => item.v_full_range,
            SPItem::Typedef(item) => item.v_full_range,
            SPItem::Typeset(item) => item.v_full_range,
            SPItem::Include(item) => item.v_range,
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
            SPItem::Typeset(item) => item.name.clone(),
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
            SPItem::Typeset(_) => None,
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
            SPItem::Typeset(item) => item.name.clone(),
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
            SPItem::Typeset(item) => Some(item.description.clone()),
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
            SPItem::Typeset(item) => item.uri.clone(),
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
            SPItem::Typeset(item) => Some(&item.references),
            SPItem::Include(_) => None,
        }
    }

    pub fn children(&self) -> Option<&Vec<Arc<RwLock<SPItem>>>> {
        match self {
            SPItem::Function(item) => Some(&item.children),
            SPItem::Enum(item) => Some(&item.children),
            SPItem::EnumStruct(item) => Some(&item.children),
            SPItem::Methodmap(item) => Some(&item.children),
            SPItem::Typeset(item) => Some(&item.children),
            _ => None,
        }
    }

    pub fn push_reference(&mut self, reference: Location) {
        if range_equals_range(&self.range(), &reference.range) && self.uri().eq(&reference.uri) {
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
            SPItem::Typeset(item) => item.references.push(reference),
            SPItem::Include(_) => {}
        }
    }

    pub fn push_child(&mut self, child: Arc<RwLock<SPItem>>) {
        match self {
            SPItem::Function(item) => item.children.push(child),
            SPItem::Enum(item) => item.children.push(child),
            SPItem::EnumStruct(item) => item.children.push(child),
            SPItem::Methodmap(item) => item.children.push(child),
            SPItem::Typeset(item) => item.children.push(child),
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
            SPItem::Typeset(item) => item.references = references,
            SPItem::Include(_) => {}
        }
    }

    pub(crate) fn key(&self) -> String {
        match self {
            SPItem::Variable(item) => item.key(),
            SPItem::Function(item) => item.key(),
            SPItem::Enum(item) => item.key(),
            SPItem::EnumMember(item) => item.key(),
            SPItem::EnumStruct(item) => item.key(),
            SPItem::Define(item) => item.key(),
            SPItem::Methodmap(item) => item.key(),
            SPItem::Property(item) => item.key(),
            SPItem::Typedef(item) => item.key(),
            SPItem::Typeset(item) => item.key(),
            SPItem::Include(_) => todo!(),
        }
    }

    pub fn push_param(&mut self, param: Arc<RwLock<Parameter>>) {
        match self {
            SPItem::Typedef(item) => item.params.push(param),
            SPItem::Function(item) => item.params.push(param),
            _ => {
                log::warn!("Can only push type params to functions and typedefs.")
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
                log::warn!("Cannot set the methodmap inherits of an item that is not a methodmap.")
            }
        }
    }

    pub fn ctor(&self) -> Option<Arc<RwLock<SPItem>>> {
        match self {
            SPItem::Variable(_) => None,
            SPItem::Function(_) => None,
            SPItem::Enum(_) => None,
            SPItem::EnumMember(_) => None,
            SPItem::EnumStruct(_) => None,
            SPItem::Define(_) => None,
            SPItem::Methodmap(item) => item.ctor(),
            SPItem::Property(_) => None,
            SPItem::Typedef(_) => None,
            SPItem::Typeset(_) => None,
            SPItem::Include(_) => None,
        }
    }

    pub fn doc_completion(&self, line: &str) -> Option<CompletionList> {
        match self {
            SPItem::Variable(_) => None,
            SPItem::Function(item) => item.doc_completion(line),
            SPItem::Enum(_) => None,
            SPItem::EnumMember(_) => None,
            SPItem::EnumStruct(_) => None,
            SPItem::Define(_) => None,
            SPItem::Methodmap(_) => None,
            SPItem::Property(_) => None,
            SPItem::Typedef(_) => None,
            SPItem::Typeset(_) => None,
            SPItem::Include(_) => None,
        }
    }

    pub(crate) fn documentation(&self) -> Documentation {
        Documentation::MarkupContent(MarkupContent {
            kind: lsp_types::MarkupKind::Markdown,
            value: match self {
                SPItem::Variable(item) => item.description.to_md(),
                SPItem::Function(item) => item.description.to_md(),
                SPItem::Enum(item) => item.description.to_md(),
                SPItem::EnumMember(item) => item.description.to_md(),
                SPItem::EnumStruct(item) => item.description.to_md(),
                SPItem::Define(item) => item.description.to_md(),
                SPItem::Methodmap(item) => item.description.to_md(),
                SPItem::Property(item) => item.description.to_md(),
                SPItem::Typedef(item) => item.description.to_md(),
                SPItem::Typeset(item) => item.description.to_md(),
                SPItem::Include(_) => "".to_string(),
            },
        })
    }

    pub(crate) fn formatted_text(&self) -> String {
        match self {
            SPItem::Variable(item) => item.formatted_text(),
            SPItem::Function(item) => item.formatted_text(),
            SPItem::Enum(item) => item.formatted_text(),
            SPItem::EnumMember(item) => item.formatted_text(),
            SPItem::EnumStruct(item) => item.formatted_text(),
            SPItem::Define(item) => item.formatted_text(),
            SPItem::Methodmap(item) => item.formatted_text(),
            SPItem::Property(item) => item.formatted_text(),
            SPItem::Typedef(item) => item.formatted_text(),
            SPItem::Typeset(item) => item.formatted_text(),
            SPItem::Include(item) => item.formatted_text(),
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
            SPItem::Typeset(item) => {
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
            SPItem::Typeset(item) => item.to_hover(params),
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
            SPItem::Typeset(item) => item.to_definition(params),
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
            SPItem::Property(item) => item.to_document_symbol(),
            SPItem::Typedef(item) => item.to_document_symbol(),
            SPItem::Typeset(item) => item.to_document_symbol(),
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
