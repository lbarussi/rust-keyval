use crate::db::storage::Db;

pub async fn execute(parts: Vec<String>, db: &Db) -> String {
    if parts.len() < 2 {
        return "ERR usage DEL key\n".into();
    }

    let key = &parts[1];

    let mut db = db.lock().await;
    match db.remove(key) {
        Some(_) => "1\n".into(),
        None => "0\n".into(),
    }
}
