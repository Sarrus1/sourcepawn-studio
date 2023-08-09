use lsp_types::{
    CallHierarchyIncomingCall, CallHierarchyIncomingCallsParams, CallHierarchyItem,
    CallHierarchyOutgoingCall, CallHierarchyOutgoingCallsParams, CallHierarchyPrepareParams,
};

use crate::{
    spitem::SPItem,
    store::Store,
    utils::{range_contains_range, range_to_position_average},
};

pub fn prepare(
    store: &Store,
    params: CallHierarchyPrepareParams,
) -> Option<Vec<CallHierarchyItem>> {
    let items = &store.get_items_from_position(
        params.text_document_position_params.position,
        &params.text_document_position_params.text_document.uri,
    );
    if items.is_empty() {
        return None;
    }

    let item = items[0].read();
    if let SPItem::Function(function_item) = &*item {
        Some(vec![function_item.to_call_hierarchy()])
    } else {
        None
    }
}

pub fn outgoing(
    store: &Store,
    params: CallHierarchyOutgoingCallsParams,
) -> Option<Vec<CallHierarchyOutgoingCall>> {
    let items = &store.get_items_from_position(
        range_to_position_average(&params.item.selection_range),
        &params.item.uri,
    );
    if items.is_empty() {
        return None;
    }

    let mut outgoing_calls = vec![];
    let origin_item = &*items[0].read();
    if let SPItem::Function(function_origin_item) = origin_item {
        for item in store.get_all_items(true).0.iter() {
            if let SPItem::Function(function_item) = &*item.read() {
                let mut from_ranges = vec![];
                for reference in function_item.references.iter() {
                    if range_contains_range(&function_origin_item.full_range, &reference.range)
                        && function_origin_item.uri == reference.uri
                    {
                        from_ranges.push(reference.range);
                    }
                }
                if from_ranges.is_empty() {
                    continue;
                }
                outgoing_calls.push(CallHierarchyOutgoingCall {
                    to: function_item.to_call_hierarchy(),
                    from_ranges,
                })
            }
        }
    }

    Some(outgoing_calls)
}

pub fn incoming(
    store: &Store,
    params: CallHierarchyIncomingCallsParams,
) -> Option<Vec<CallHierarchyIncomingCall>> {
    let items = &store.get_items_from_position(
        range_to_position_average(&params.item.selection_range),
        &params.item.uri,
    );

    if items.is_empty() {
        return None;
    }

    let mut incoming_calls = vec![];
    let origin_item = &*items[0].read();
    if let SPItem::Function(function_origin_item) = origin_item {
        for item in store.get_all_items(true).0.iter() {
            if let SPItem::Function(function_item) = &*item.read() {
                let mut from_ranges = vec![];
                for reference in function_origin_item.references.iter() {
                    if range_contains_range(&function_item.full_range, &reference.range)
                        && function_item.uri == reference.uri
                    {
                        from_ranges.push(reference.range);
                    }
                }
                if from_ranges.is_empty() {
                    continue;
                }
                incoming_calls.push(CallHierarchyIncomingCall {
                    from: function_item.to_call_hierarchy(),
                    from_ranges,
                })
            }
        }
    }

    Some(incoming_calls)
}
