use crate::db::storage::Db;

pub async fn execute(parts: Vec<&str>, db: &Db) -> String {
    if parts.len() < 2 {
        return "ERR usage DEL key\n".into();
    }

    let mut db = db.lock().await;

    match db.remove(parts[1]) {
        Some(_) => "1\n".into(),
        None => "0\n".into(),
    }
}
