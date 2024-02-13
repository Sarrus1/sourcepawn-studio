use base_db::Upcast;
use hir_def::DefDatabase;

#[salsa::query_group(HirDatabaseStorage)]
pub trait HirDatabase:
    DefDatabase + Upcast<dyn DefDatabase> + preprocessor::db::PreprocDatabase
{
}
