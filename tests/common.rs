use std::sync::Once;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

static START: Once = Once::new();

/// Setup function that is only run once, even if called multiple times.
pub fn setup() {
    START.call_once(|| {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    });
}
