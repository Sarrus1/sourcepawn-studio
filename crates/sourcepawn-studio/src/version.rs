/// Returns information about cargo's version.
pub fn version() -> String {
    const V: &str = env!("CARGO_PKG_VERSION");
    V.to_string()
}
