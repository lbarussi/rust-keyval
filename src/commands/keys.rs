use crate::db::storage::Db;
use crate::protocol::resp::encoder::RespValue;

pub async fn execute(_parts: Vec<String>, db: &Db) -> RespValue {
    let db = db.lock().await;

    let mut items = Vec::new();
    for k in db.keys() {
        items.push(RespValue::Bulk(Some(k.clone().into_bytes())));
    }

    RespValue::Array(items)
}