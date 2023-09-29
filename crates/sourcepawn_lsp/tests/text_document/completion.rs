use std::time::Duration;

use insta::assert_json_snapshot;
use itertools::Itertools;
use lsp_types::{
    request::{Completion, ResolveCompletionItem},
    CompletionItem, CompletionParams, CompletionResponse,
};

use crate::fixture::TestBed;

fn complete(fixture: &str) -> Vec<CompletionItem> {
    let test_bed = TestBed::new(fixture).unwrap();
    test_bed
        .initialize(
            serde_json::from_value(serde_json::json!({
                "textDocument": {
                    "completion": {
                        "completionItem": {
                            "documentationFormat": ["plaintext", "markdown"]
                        }
                    }
                },
                "workspace": {
                    "configuration": true,
                    "workspace_folders": true
                }
            }))
            .unwrap(),
        )
        .unwrap();
    let text_document_position = test_bed.cursor().unwrap();
    test_bed
        .internal_rx
        .recv_timeout(Duration::from_secs(10))
        .unwrap();
    let items = match test_bed
        .client()
        .send_request::<Completion>(CompletionParams {
            text_document_position,
            partial_result_params: Default::default(),
            work_done_progress_params: Default::default(),
            context: None,
        })
        .unwrap()
    {
        Some(CompletionResponse::Array(items)) => items,
        Some(CompletionResponse::List(list)) => list.items,
        None => Vec::new(),
    };

    items
        .into_iter()
        .map(|item| match item.data {
            Some(_) => {
                let mut item = test_bed
                    .client()
                    .send_request::<ResolveCompletionItem>(item)
                    .unwrap();

                item.data = None;
                item
            }
            None => item,
        })
        .sorted_by(|item1, item2| item1.label.cmp(&item2.label))
        .collect()
}

#[test]
fn global_variable_1() {
    assert_json_snapshot!(complete(
        r#"
%! main.sp
int foo;

|
^"#
    ));
}

#[test]
fn global_variable_include_1() {
    assert_json_snapshot!(complete(
        r#"
%! include/bar.sp
int foo;
%! main.sp
#include "bar.sp"

|
^"#
    ));
}

#[test]
fn global_variable_circular_include_1() {
    assert_json_snapshot!(complete(
        r#"
%! foo.sp
#include "bar.sp"
int foo;
%! bar.sp
#include "foo.sp"
int bar;

|
^"#
    ));
}
