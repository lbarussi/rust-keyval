mod server;
mod db;
mod protocol;
mod commands;

use db::storage::new_db;
use crate::db::storage::Db;

#[tokio::main]
async fn main() {
    let db: Db = new_db();

    server::tcp_server::start("127.0.0.1:6374", db).await;
}
