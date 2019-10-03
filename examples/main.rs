use serde::{Serialize, Deserialize};
use ddb::*;


///////////////////////////////////////////////////////////////////////////////
// UTILS
///////////////////////////////////////////////////////////////////////////////

pub fn random_ascii_string(length: usize) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    std::iter::repeat(())
        .map(|()| rng.sample(rand::distributions::Alphanumeric))
        .take(length)
        .collect::<String>()
}

pub fn random_id_string() -> String {
    random_ascii_string(7)
}


///////////////////////////////////////////////////////////////////////////////
// MODEL
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TodoItem {
    pub name: String,
    pub title: String,
}

impl EntityKey for TodoItem {
    fn entity_kind_key() -> String {
        String::from("TodoItem")
    }
    fn entity_name_key(&self) -> String {
        self.name.clone()
    }
}


///////////////////////////////////////////////////////////////////////////////
// MAIN
///////////////////////////////////////////////////////////////////////////////

fn init_db() -> DatastoreClient {
    DatastoreClient::new()
        .expect("DatastoreClient init failed (probably could not find auth credentials)")
}

fn insert_new_todo(db: &DatastoreClient) -> String {
    let item = TodoItem {
        name: random_id_string(),
        title: String::from("lorem ipsum")
    };
    println!("new todo: {:#?}", item);
    let result = db.upsert(item.clone());
    println!("result [insert]: {:#?}", result);
    item.name
}

fn get_todo(db: &DatastoreClient, name: &str) {
    let result = db.get::<TodoItem, _>(name);
    //                    ^^^^^^^^ The type of the thing we are retrieving
    println!("result [get]: {:#?}", result);
}

fn main() {
    let db = init_db();
    let key = insert_new_todo(&db);
    get_todo(&db, &key);
}
