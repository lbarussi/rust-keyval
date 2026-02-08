use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use std::time::Instant;

use crate::db::storage::Db;
use crate::protocol::parser;
use crate::protocol::resp::encoder::RespValue;
use crate::protocol::resp::parser::parse_resp_one;
use crate::server::metrics_prom;

pub async fn handle(mut stream: TcpStream, db: Db) {
    metrics_prom::ACTIVE_CONNS.inc();

    let result = async {
        let mut buffer = [0u8; 4096];
        let mut acc: Vec<u8> = Vec::new();

        loop {
            let n = match stream.read(&mut buffer).await {
                Ok(0) => return,
                Ok(n) => n,
                Err(_) => return,
            };

            acc.extend_from_slice(&buffer[..n]);

            loop {
                if acc.is_empty() {
                    break;
                }

                if acc[0] == b'*' {
                    match parse_resp_one(&acc) {
                        Ok(Some((parts, consumed))) => {
                            let cmd = parts
                                .get(0)
                                .map(|s| s.to_uppercase())
                                .unwrap_or_else(|| "UNKNOWN".into());

                            metrics_prom::CMD_TOTAL
                                .with_label_values(&[cmd.as_str()])
                                .inc();

                            let t0 = Instant::now();
                            let resp = parser::process_parts(parts, &db).await;
                            metrics_prom::CMD_LATENCY
                                .with_label_values(&[cmd.as_str()])
                                .observe(t0.elapsed().as_secs_f64());

                            let bytes = resp.to_bytes();
                            let _ = stream.write_all(&bytes).await;

                            acc.drain(..consumed);
                            continue;
                        }
                        Ok(None) => {
                            break;
                        }
                        Err(e) => {
                            let bytes = RespValue::Error(format!("ERR {}", e)).to_bytes();
                            let _ = stream.write_all(&bytes).await;
                            acc.clear();
                            break;
                        }
                    }
                } else {
                    if let Some(pos) = acc.iter().position(|&b| b == b'\n') {
                        let line_bytes = acc[..pos].to_vec();
                        acc.drain(..pos + 1);

                        let line = String::from_utf8_lossy(&line_bytes).trim().to_string();
                        if line.is_empty() {
                            continue;
                        }

                        let parts: Vec<String> =
                            line.split_whitespace().map(|s| s.to_string()).collect();

                        let cmd = parts
                            .get(0)
                            .map(|s| s.to_uppercase())
                            .unwrap_or_else(|| "UNKNOWN".into());

                        metrics_prom::CMD_TOTAL
                            .with_label_values(&[cmd.as_str()])
                            .inc();

                        let t0 = Instant::now();
                        let resp = parser::process_parts(parts, &db).await;
                        metrics_prom::CMD_LATENCY
                            .with_label_values(&[cmd.as_str()])
                            .observe(t0.elapsed().as_secs_f64());

                        let bytes = resp.to_bytes();
                        let _ = stream.write_all(&bytes).await;

                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
    }
        .await;

    metrics_prom::ACTIVE_CONNS.dec();

    let _ = result;
}
