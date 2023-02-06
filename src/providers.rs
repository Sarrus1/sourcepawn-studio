pub mod completion;
pub mod definition;
pub mod document_symbol;
pub mod hover;
pub mod reference;
pub mod semantic_tokens;
pub mod signature_help;

use std::sync::Arc;

use lsp_types::Url;

use crate::store::Store;

#[derive(Clone)]
pub struct FeatureRequest<P> {
    pub params: P,
    pub store: Store,
    pub uri: Arc<Url>,
}
