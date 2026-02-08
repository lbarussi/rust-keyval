use once_cell::sync::Lazy;
use prometheus::{Counter, Encoder, HistogramOpts, HistogramVec, IntCounterVec, IntGauge, Opts, Registry, TextEncoder};

pub static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

pub static ACTIVE_CONNS: Lazy<IntGauge> = Lazy::new(|| {
    let g = IntGauge::with_opts(Opts::new("keyval_active_connections", "Active TCP connections"))
        .unwrap();
    REGISTRY.register(Box::new(g.clone())).unwrap();
    g
});

pub static KEYS_COUNT: Lazy<IntGauge> = Lazy::new(|| {
    let g = IntGauge::with_opts(Opts::new("keyval_keys_count", "Number of keys in DB")).unwrap();
    REGISTRY.register(Box::new(g.clone())).unwrap();
    g
});

pub static CMD_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    let c = IntCounterVec::new(
        Opts::new("keyval_cmd_total", "Total commands processed"),
        &["cmd"],
    )
        .unwrap();
    REGISTRY.register(Box::new(c.clone())).unwrap();
    c
});

pub static CMD_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    let h = HistogramVec::new(
        HistogramOpts::new("keyval_cmd_latency_seconds", "Command latency in seconds"),
        &["cmd"],
    )
        .unwrap();
    REGISTRY.register(Box::new(h.clone())).unwrap();
    h
});

pub static BYTES_IN: Lazy<prometheus::IntCounter> = Lazy::new(|| {
    let c = prometheus::IntCounter::new("keyval_bytes_in_total", "Total bytes read").unwrap();
    REGISTRY.register(Box::new(c.clone())).unwrap();
    c
});

pub static BYTES_OUT: Lazy<prometheus::IntCounter> = Lazy::new(|| {
    let c = prometheus::IntCounter::new("keyval_bytes_out_total", "Total bytes written").unwrap();
    REGISTRY.register(Box::new(c.clone())).unwrap();
    c
});

pub static PROCESS_RSS_BYTES: Lazy<prometheus::IntGauge> = Lazy::new(|| {
    let g = prometheus::IntGauge::new(
        "keyval_process_resident_memory_bytes",
        "Resident set size (RSS) of the keyval process in bytes",
    )
        .unwrap();
    REGISTRY.register(Box::new(g.clone())).unwrap();
    g
});

pub static PROCESS_CPU_SECONDS_TOTAL: Lazy<Counter> = Lazy::new(|| {
    let c = Counter::with_opts(Opts::new(
        "process_cpu_seconds_total",
        "Total user and system CPU time spent in seconds",
    )).unwrap();
    REGISTRY.register(Box::new(c.clone())).unwrap();
    c
});

pub fn gather() -> Vec<u8> {
    let metric_families = REGISTRY.gather();
    let mut out = Vec::new();
    let encoder = TextEncoder::new();
    encoder.encode(&metric_families, &mut out).unwrap();
    out
}
