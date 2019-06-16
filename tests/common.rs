use std::sync::{Once, ONCE_INIT};

#[allow(dead_code)]
static INIT: Once = ONCE_INIT;

/// Setup function that is only run once, even if called multiple times.
#[allow(dead_code)]
pub(crate) fn setup() {
    INIT.call_once(|| {
        env_logger::init();
    });
}
