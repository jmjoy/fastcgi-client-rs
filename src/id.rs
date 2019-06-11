static COUNTER: AtomicU16 = AtomicU16::new(0);

pub(crate) struct RequestIdGenerator;

impl RequestIdGenerator {
    pub(crate) fn generate() -> u16 {
        COUNTER += 1;
        COUNTER.get()
    }
}

impl Drop for RequestIdGenerator {
    fn drop(&mut self) {
        COUNTER -= 1;
    }
}
