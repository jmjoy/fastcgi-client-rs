use std::sync::Once;

static START: Once = Once::new();

/// Setup function that is only run once, even if called multiple times.
pub fn setup() {
    START.call_once(|| {
        env_logger::init();
    });
}
