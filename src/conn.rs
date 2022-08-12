/// Connection mode, indicate is keep alive or not.
pub trait Mode {
    fn is_keep_alive() -> bool;
}

/// Short connection mode.
pub struct Short;

impl Mode for Short {
    fn is_keep_alive() -> bool {
        false
    }
}

/// Keep alive connection mode.
pub struct KeepAlive {}

impl Mode for KeepAlive {
    fn is_keep_alive() -> bool {
        true
    }
}