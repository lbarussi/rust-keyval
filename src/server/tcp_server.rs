use tokio::net::TcpListener;

use crate::db::storage::Db;
use crate::server::connection;

pub async fn start(addr: &str, db: Db) {
    let listener: TcpListener = TcpListener::bind(addr).await.unwrap();

    println!("Server running on {}", addr);

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let db_clone: Db = db.clone();

        tokio::spawn(async move {
            connection::handle(stream, db_clone).await;
        });
    }
}
