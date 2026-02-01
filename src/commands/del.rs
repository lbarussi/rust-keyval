use crate::db::storage::Db;

pub async fn execute(parts: Vec<&str>, db: &Db) -> String {
    if parts.len() < 2 {
        return "ERR usage DEL key\n".into();
    }

    let mut db = db.lock().await;

    let removed: Option<String> = db.remove(parts[1]);

    match removed {
        Some(_) => "1\n".into(),
        None => "0\n".into(),
    }
}
