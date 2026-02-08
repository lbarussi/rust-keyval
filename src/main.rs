mod server;
mod db;
mod protocol;
mod commands;

use db::storage::new_db;
use crate::db::ttl_cleaner::start_cleaner;

#[tokio::main]
async fn main() {
    /**

    ⭐ TTL real (com background task) ⭐ Persistência em arquivo ⭐ RESP protocol real ⭐ Pub/Sub

    **/

    let db = new_db();

    start_cleaner(db.clone()).await;

    server::tcp_server::start("127.0.0.1:6374", db).await;
}