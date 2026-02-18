use crate::system::SystemContext;
use std::sync::OnceLock;

static SYSTEM_CONTEXT: OnceLock<SystemContext> = OnceLock::new();

pub fn initialize() {
    SYSTEM_CONTEXT.get_or_init(|| SystemContext::collect());
}

pub fn get_context() -> &'static SystemContext {
    SYSTEM_CONTEXT
        .get()
        .expect("SystemContext must be initialized before use!")
}
