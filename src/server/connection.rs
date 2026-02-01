use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::db::storage::Db;
use crate::protocol::parser;

pub async fn handle(mut stream: TcpStream, db: Db) {
    let mut buffer = [0; 1024];

    loop {
        let n = match stream.read(&mut buffer).await {
            Ok(0) => return,
            Ok(n) => n,
            Err(_) => return,
        };

        let input = String::from_utf8_lossy(&buffer[..n]).trim().to_string();

        let response = parser::process(input, &db).await;

        let _ = stream.write_all(response.as_bytes()).await;
    }
}
