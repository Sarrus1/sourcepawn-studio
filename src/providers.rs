mod completion;
mod hover;

use std::sync::Arc;

use lsp_types::Url;

use crate::store::Store;

pub use self::completion::provide_completions;
pub use self::hover::provide_hover;

#[derive(Clone)]
pub struct FeatureRequest<P> {
    pub params: P,
    pub store: Store,
    pub uri: Arc<Url>,
}
