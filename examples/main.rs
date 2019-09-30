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
    let api_key = ApiKey::lookup().expect("ApiKey::lookup failed");
    DatastoreClient::new(api_key)
}


fn main() {
    let db = init_db();
    let item = TodoItem {
        name: random_id_string(),
        title: String::from("lorem ipsum")
    };
    let result = db.upsert(item.clone());
    println!("result: {:#?}", result);
}
