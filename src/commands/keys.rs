use crate::db::storage::Db;

pub async fn execute(_parts: Vec<String>, db: &Db) -> String {
    let db = db.lock().await;
    let keys = db.keys().cloned().collect::<Vec<_>>().join(", ");
    format!("{}\n", keys)
}