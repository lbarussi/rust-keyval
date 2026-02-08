mod server;
mod db;
mod protocol;
mod commands;

use std::time::Duration;
use crate::db::storage::{new_db, Db};
use crate::db::ttl_cleaner::start_cleaner;
use crate::server::http_metrics;
use tokio::time::{interval, Interval};

#[tokio::main]
async fn main() {
    let db: Db = new_db();

    tokio::spawn({
        let db = db.clone();
        async move {
            let mut tick = interval(Duration::from_secs(1));
            loop {
                tick.tick().await;
                let n = db.lock().await.len() as i64;
                server::metrics_prom::KEYS_COUNT.set(n);
            }
        }
    });

    tokio::spawn(async move {
        let mut tick: Interval = interval(Duration::from_secs(1));
        let mut last_cpu: Option<f64> = None;

        loop {
            tick.tick().await;

            if let Some(rss) = server::linux_proc::rss_bytes() {
                server::metrics_prom::PROCESS_RSS_BYTES.set(rss as i64);
            }

            if let Some(cpu_now) = crate::server::linux_proc::cpu_seconds_total() {
                match last_cpu {
                    Some(prev) if cpu_now >= prev => {
                        server::metrics_prom::PROCESS_CPU_SECONDS_TOTAL.inc_by(cpu_now - prev);
                    }
                    None => {
                        // First reading: it doesn't add anything.
                    }
                    _ => {
                        // rare case (overflow/error): ignore
                    }
                }
                last_cpu = Some(cpu_now);
            }
        }
    });

    start_cleaner(db.clone()).await;

    let keyval_bind: String = std::env::var("KEYVAL_BIND")
        .unwrap_or_else(|_| "127.0.0.1:6374".into());

    let metrics_bind: String = std::env::var("METRICS_BIND")
        .unwrap_or_else(|_| "127.0.0.1:9100".into());


    let _ = &*server::metrics_prom::ACTIVE_CONNS;
    let _ = &*server::metrics_prom::KEYS_COUNT;
    let _ = &*server::metrics_prom::CMD_TOTAL;
    let _ = &*server::metrics_prom::CMD_LATENCY;
    let _ = &*server::metrics_prom::PROCESS_RSS_BYTES;
    let _ = &*server::metrics_prom::PROCESS_CPU_SECONDS_TOTAL;

    println!("Metrics running on {}", metrics_bind);
    {
        let mb = metrics_bind.clone();
        tokio::spawn(async move {
            http_metrics::serve(&mb).await;
        });
    }

    server::tcp_server::start(&keyval_bind, db).await;
}