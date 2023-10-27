
lazy_static::lazy_static! {
    /// Default requests timeout in seconds
    pub static ref REQUESTS_TIMEOUT: u64 = match option_env!("LAUNCHER_REQUESTS_TIMEOUT") {
        Some(timeout) => timeout.parse().unwrap_or(4),
        None => 8
    };
}