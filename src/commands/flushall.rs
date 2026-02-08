use crate::db::storage::Db;

pub async fn execute(_parts: Vec<String>, db: &Db) -> String {
    let mut db = db.lock().await;
    db.clear();
    "OK\n".into()
}
