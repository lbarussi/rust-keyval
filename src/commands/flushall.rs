use crate::db::storage::Db;
use crate::protocol::resp::encoder::RespValue;

pub async fn execute(_parts: Vec<String>, db: &Db) -> RespValue {
    let mut db = db.lock().await;
    db.clear();
    RespValue::SimpleString("OK".into())
}
