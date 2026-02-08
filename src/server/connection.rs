use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::db::storage::Db;
use crate::protocol::parser;
use crate::protocol::resp::parser::parse_resp;

pub async fn handle(mut stream: TcpStream, db: Db) {

    let mut buffer = [0; 1024];
    let mut acc = String::new();

    loop {

        let n = match stream.read(&mut buffer).await {
            Ok(0) => return,
            Ok(n) => n,
            Err(_) => return,
        };

        let incoming = String::from_utf8_lossy(&buffer[..n]);
        println!("READ >>> {:?}", incoming);

        acc.push_str(&incoming);
        println!("ACC >>> {:?}", acc);

        loop {

            if acc.starts_with('*') {

                println!("PROTO >>> RESP detected");

                if !acc.ends_with("\r\n") {
                    println!("WAIT >>> RESP not complete yet");
                    break;
                }

                println!("RESP >>> Trying parse");

                let parts: Vec<String> = parse_resp(&acc.trim());
                println!("PARTS >>> {:?}", parts);

                if parts.is_empty() {
                    println!("WARN >>> Empty parts, waiting more data");
                    break;
                }

                let normalized_input = parts.join(" ");
                println!("PROCESS >>> {}", normalized_input);

                let response = parser::process(normalized_input, &db).await;
                println!("RESPONSE >>> {:?}", response);

                let _ = stream.write_all(response.as_bytes()).await;

                println!("ACC >>> cleared");
                acc = String::new();

                break;
            }

            if let Some(pos) = acc.find('\n') {

                println!("PROTO >>> Plain text detected");

                let line = acc[..pos].trim().to_string();
                acc = acc[pos + 1..].to_string();

                println!("LINE >>> {:?}", line);
                println!("ACC REMAIN >>> {:?}", acc);

                if !line.is_empty() {

                    println!("PROCESS >>> {}", line);

                    let response = parser::process(line, &db).await;
                    println!("RESPONSE >>> {:?}", response);

                    let _ = stream.write_all(response.as_bytes()).await;
                }

            } else {
                println!("WAIT >>> Need more data");
                break;
            }
        }
    }
}
