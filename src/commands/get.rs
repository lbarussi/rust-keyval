use crate::db::storage::Db;

pub async fn execute(parts: Vec<&str>, db: &Db) -> String {
    if parts.len() < 2 {
        return "Err usage GET key/n".into();
    }

    let mut db = db.lock().await;
    
    match db.get(parts[1]) {
        Some(value) => format!("{}\n", value),
        None => "NULL\n".into()
    }
}