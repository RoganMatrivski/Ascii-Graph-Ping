use std::time::Duration;

#[derive(Debug)]
pub struct PingResult {
    pub ip_id: usize,
    pub seq: usize,
    pub rtt: Option<Duration>,
}

#[derive(Debug)]
pub struct PingResultNoIP {
    pub seq: usize,
    pub rtt_arr: Vec<Duration>,
}

#[derive(Debug)]
pub struct AppConfig {
    pub term_width: u32,
    pub term_height: u32,
}
