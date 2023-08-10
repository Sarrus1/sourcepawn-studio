use lsp_types::Range;
use parking_lot::RwLock;
use std::sync::Arc;
use syntax::SPItem;

use crate::range_contains_range;

#[derive(Debug, Default)]
pub struct Scope {
    pub func: Option<Arc<RwLock<SPItem>>>,
    pub mm_es: Option<Arc<RwLock<SPItem>>>,
}

impl Scope {
    fn func_full_range(&self) -> Range {
        self.func.as_ref().unwrap().read().full_range()
    }

    fn mm_es_full_range(&self) -> Range {
        self.mm_es.as_ref().unwrap().read().full_range()
    }

    pub fn update_func(
        &mut self,
        range: Range,
        func_idx: &mut usize,
        funcs_in_file: &Vec<Arc<RwLock<SPItem>>>,
    ) {
        // Do not update the function, we are still in its scope.
        if self.func.is_some() && range_contains_range(&self.func_full_range(), &range) {
            return;
        }

        if *func_idx >= funcs_in_file.len() {
            self.func = None;
            return;
        }

        let next_func_range = funcs_in_file[*func_idx].read().full_range();
        if range_contains_range(&next_func_range, &range) {
            self.func = Some(funcs_in_file[*func_idx].clone());
            *func_idx += 1;
        } else {
            self.func = None;
        }
    }

    pub fn update_mm_es(
        &mut self,
        range: Range,
        mm_es_idx: &mut usize,
        mm_es_in_file: &Vec<Arc<RwLock<SPItem>>>,
    ) {
        // Do not update the function, we are still in its scope.
        if self.mm_es.is_some() && range_contains_range(&self.mm_es_full_range(), &range) {
            return;
        }

        if *mm_es_idx >= mm_es_in_file.len() {
            self.mm_es = None;
            return;
        }

        let next_mm_es_range = mm_es_in_file[*mm_es_idx].read().full_range();
        if range_contains_range(&next_mm_es_range, &range) {
            self.mm_es = Some(mm_es_in_file[*mm_es_idx].clone());
            *mm_es_idx += 1;
        } else {
            self.mm_es = None;
        }
    }

    pub fn func_key(&self) -> String {
        if self.func.is_none() {
            return "".to_string();
        }
        self.func.clone().unwrap().read().name()
    }

    pub fn mm_es_key(&self) -> String {
        if self.mm_es.is_none() {
            return "".to_string();
        }
        self.mm_es.clone().unwrap().read().name()
    }
}
