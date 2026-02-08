use std::time::Instant;

use crate::db::storage::Db;

pub async fn execute(parts: Vec<String>, db: &Db) -> String {
    if parts.len() < 2 {
        return "ERR usage GET key\n".into();
    }

    let key = &parts[1];
    let mut db = db.lock().await;

    if let Some(entry) = db.get(key) {
        if let Some(exp) = entry.expire_at {
            if Instant::now() > exp {
                db.remove(key);
                return "NULL\n".into();
            }
        }
        return format!("{}\n", entry.value);
    }

    "NULL\n".into()
}
