use crate::db::storage::Db;

pub async fn execute(parts: Vec<&str>, db: &Db) -> String {
    if parts.len() < 2 {
        return "ERR usage EXISTS key\n".into();
    }

    let db = db.lock().await;

    if db.contains_key(parts[1]) {
        "1\n".into()
    } else {
        "0\n".into()
    }
}
