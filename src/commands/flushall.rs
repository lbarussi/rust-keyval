use crate::db::storage::Db;

pub async fn execute(_parts: Vec<&str>, db: &Db) -> String {
    let mut db = db.lock().await;
    db.clear();
    "OK\n".into()
}
