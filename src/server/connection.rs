use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::db::storage::Db;
use crate::protocol::parser;
use crate::protocol::resp::parser::parse_resp_one;

pub async fn handle(mut stream: TcpStream, db: Db) {
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
                        let resp = parser::process_parts(parts, &db).await;
                        let bytes = resp.to_bytes();
                        let _ = stream.write_all(&bytes).await;

                        acc.drain(..consumed);
                        continue;
                    }
                    Ok(None) => {
                        break;
                    }
                    Err(e) => {
                        let _ = stream
                            .write_all(format!("ERR {}\n", e).as_bytes())
                            .await;
                        acc.clear();
                        break;
                    }
                }
            } else {
                if let Some(pos) = acc.iter().position(|&b| b == b'\n') {
                    let line_bytes = acc[..pos].to_vec();
                    acc.drain(..pos + 1);

                    let line = String::from_utf8_lossy(&line_bytes).trim().to_string();
                    if !line.is_empty() {
                        let response = parser::process(line, &db).await;
                        let _ = stream.write_all(response.as_bytes()).await;
                    }
                    continue;
                } else {
                    break;
                }
            }
        }
    }
}
