use prometheus::{
    register_histogram_vec, register_int_counter_vec, HistogramVec,
    IntCounterVec,
};

pub const SOURCE_SYSTEM: &str = "SYSTEM";

lazy_static::lazy_static! {
    pub static ref HTTP_REQUESTS_TOTAL: IntCounterVec=register_int_counter_vec!(
        "http_requests_total",
        "Total number of HTTP requests",
        &["method", "path"]).unwrap();

    pub static ref HTTP_REQUESTS_DURATION_SECONDS: HistogramVec=register_histogram_vec!(
        "http_requests_duration_seconds",
        "Duration of HTTP requests",
        &["method", "path"],
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]).unwrap();

}
