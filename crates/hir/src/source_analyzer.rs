use std::iter;
use std::sync::Arc;

use hir_def::{
    body::{
        scope::{ExprScopes, ScopeId},
        Body, BodySourceMap,
    },
    resolver::resolver_for_scope,
    resolver::Resolver,
    DefWithBodyId, ExprId, InFile, InferenceResult,
};
use syntax::TSKind;
use tree_sitter::Point;
use vfs::FileId;

use crate::{db::HirDatabase, Field};

/// `SourceAnalyzer` is a convenience wrapper which exposes HIR API in terms of
/// original source files. It should not be used inside the HIR itself.
#[derive(Debug)]
pub(crate) struct SourceAnalyzer {
    pub(crate) file_id: FileId,
    pub(crate) resolver: Resolver,
    def: Option<(DefWithBodyId, Arc<Body>, Arc<BodySourceMap>)>,
    infer: Option<Arc<InferenceResult>>,
}

impl SourceAnalyzer {
    // TODO: Add a no infer method for non field/method references.
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
        let infer = db.infer(def);
        SourceAnalyzer {
            resolver,
            def: Some((def, body, source_map)),
            file_id,
            infer: Some(infer),
        }
    }

    pub(crate) fn new_for_body_no_infer(
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
            infer: None,
        }
    }

    fn body_source_map(&self) -> Option<&BodySourceMap> {
        self.def.as_ref().map(|(.., source_map)| &**source_map)
    }

    fn body(&self) -> Option<&Body> {
        self.def.as_ref().map(|(_, body, _)| &**body)
    }

    fn expr_id(&self, _db: &dyn HirDatabase, src: &tree_sitter::Node) -> Option<ExprId> {
        let sm = self.body_source_map()?;
        sm.node_expr(src)
    }

    pub(crate) fn resolve_field(
        &self,
        db: &dyn HirDatabase,
        node: &tree_sitter::Node,
    ) -> Option<Field> {
        assert!(matches!(TSKind::from(*node), TSKind::field_access));
        let expr_id = self.expr_id(db, node)?;
        self.infer
            .as_ref()?
            .field_resolution(expr_id)
            .map(|it| it.into())
    }
}

fn scope_for(
    scopes: &ExprScopes,
    source_map: &BodySourceMap,
    node: InFile<&tree_sitter::Node>,
) -> Option<ScopeId> {
    let node_ancestors = iter::successors(Some(*node.value), |it| it.parent());
    node_ancestors
        .filter(|it| matches!(TSKind::from(*it), TSKind::block))
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
