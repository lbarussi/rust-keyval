use crate::db::storage::Db;

pub async fn execute(parts: Vec<String>, db: &Db) -> String {
    if parts.len() < 2 {
        return "ERR usage EXISTS key\n".into();
    }

    let key = &parts[1];
    let db = db.lock().await;

    if db.contains_key(key) {
        "1\n".into()
    } else {
        "0\n".into()
    }
}
