use crate::{spitem::SPItem, utils::range_contains_range};
use std::sync::{Arc, Mutex};

use lsp_types::Range;

#[derive(Debug, Default)]
pub struct Scope {
    pub func: Option<Arc<Mutex<SPItem>>>,
    pub mm_es: Option<Arc<Mutex<SPItem>>>,
}

impl Scope {
    fn func_full_range(&self) -> Range {
        self.func
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .full_range()
            .unwrap()
    }

    fn mm_es_full_range(&self) -> Range {
        self.mm_es
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .full_range()
            .unwrap()
    }

    pub fn update_func(
        &mut self,
        range: Range,
        func_idx: &mut usize,
        funcs_in_file: &Vec<Arc<Mutex<SPItem>>>,
    ) {
        // Do not update the function, we are still in its scope.
        if self.func.is_some() && range_contains_range(&self.func_full_range(), &range) {
            return;
        }

        if *func_idx >= funcs_in_file.len() {
            self.func = None;
            return;
        }

        let next_func_range = funcs_in_file[*func_idx]
            .lock()
            .unwrap()
            .full_range()
            .unwrap();

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
        mm_es_in_file: &Vec<Arc<Mutex<SPItem>>>,
    ) {
        // Do not update the function, we are still in its scope.
        if self.mm_es.is_some() && range_contains_range(&self.mm_es_full_range(), &range) {
            return;
        }

        if *mm_es_idx >= mm_es_in_file.len() {
            self.mm_es = None;
            return;
        }

        let next_mm_es_range = mm_es_in_file[*mm_es_idx]
            .lock()
            .unwrap()
            .full_range()
            .unwrap();

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
        self.func.clone().unwrap().lock().unwrap().name()
    }

    pub fn mm_es_key(&self) -> String {
        if self.mm_es.is_none() {
            return "".to_string();
        }
        self.mm_es.clone().unwrap().lock().unwrap().name()
    }
}
