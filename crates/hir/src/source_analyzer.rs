use std::iter;
use std::sync::Arc;

use hir_def::{
    body::{
        scope::{ExprScopes, ScopeId},
        Body, BodySourceMap,
    },
    resolver::resolver_for_scope,
    resolver::Resolver,
    DefWithBodyId, InFile,
};
use syntax::TSKind;
use tree_sitter::Point;
use vfs::FileId;

use crate::db::HirDatabase;

/// `SourceAnalyzer` is a convenience wrapper which exposes HIR API in terms of
/// original source files. It should not be used inside the HIR itself.
#[derive(Debug)]
pub(crate) struct SourceAnalyzer {
    pub(crate) file_id: FileId,
    pub(crate) resolver: Resolver,
    def: Option<(DefWithBodyId, Arc<Body>, Arc<BodySourceMap>)>,
}

impl SourceAnalyzer {
    pub(crate) fn new_for_body(
        db: &dyn HirDatabase,
        def: DefWithBodyId,
        node @ InFile { file_id, .. }: InFile<&tree_sitter::Node>,
        offset: Option<Point>,
    ) -> SourceAnalyzer {
        let (body, source_map) = db.body_with_source_map(def);
        let scopes = db.expr_scopes(def, file_id);
        let scope = match offset {
            None => scope_for(&scopes, &source_map, node),
            Some(offset) => scope_for_offset(db, &scopes, &source_map, file_id, offset),
        };
        let resolver = resolver_for_scope(db.upcast(), def, scope);
        SourceAnalyzer {
            resolver,
            def: Some((def, body, source_map)),
            file_id,
        }
    }
}

fn scope_for(
    scopes: &ExprScopes,
    source_map: &BodySourceMap,
    node: InFile<&tree_sitter::Node>,
) -> Option<ScopeId> {
    let node_ancestors = iter::successors(Some(*node.value), |it| it.parent());
    node_ancestors
        .filter(|it| matches!(TSKind::from(*it), TSKind::sym_block))
        .filter_map(|it| source_map.node_expr(&it))
        .find_map(|it| scopes.scope_for(it))
}

fn scope_for_offset(
    db: &dyn HirDatabase,
    scopes: &ExprScopes,
    source_map: &BodySourceMap,
    file_id: FileId,
    point: tree_sitter::Point,
) -> Option<ScopeId> {
    let tree = db.parse(file_id);
    scopes
        .scope_by_expr()
        .iter()
        .filter_map(|(id, scope)| {
            let ptr = source_map.expr_source(id)?;
            Some((ptr.to_node(&tree), scope))
        })
        .filter(|(node, _scope)| node.start_position() <= point && point <= node.end_position())
        .min_by_key(|(node, _)| node.end_byte() - node.start_byte())
        .map(|(_, id)| *id)
}
