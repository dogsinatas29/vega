use crate::system::scanner;
use crate::system::SystemContext;
use std::sync::OnceLock;

static SYSTEM_CONTEXT: OnceLock<SystemContext> = OnceLock::new();

pub fn initialize() {
    SYSTEM_CONTEXT.get_or_init(|| scanner::scan_system());
}

pub fn get_context() -> &'static SystemContext {
    SYSTEM_CONTEXT
        .get()
        .expect("SystemContext must be initialized before use!")
}
