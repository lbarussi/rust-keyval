use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::value::ValueEntry;

pub type Db = Arc<Mutex<HashMap<String, ValueEntry>>>;

pub fn new_db() -> Db {
    Arc::new(Mutex::new(HashMap::new()))
}
