use std::sync::atomic::{AtomicU16, Ordering};

static COUNTER: AtomicU16 = AtomicU16::new(1);

pub(crate) struct RequestIdGenerator;

impl RequestIdGenerator {
    pub(crate) fn generate(&self) -> u16 {
        COUNTER.fetch_add(Ordering::SeqCst)
    }
}

impl Drop for RequestIdGenerator {
    fn drop(&mut self) {
        COUNTER.fetch_sub(Ordering::SeqCst)
    }
}
