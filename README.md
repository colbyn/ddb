# Datastore DB
> Googles Cloud Firestore in <b>Datastore mode</b> - High Level Rust API (with serde support!)


## API Preview

```rust
// MODEL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TodoItem {
    pub name: String,
    pub title: String,
}
// MODEL METADATA
impl EntityKey for TodoItem {
    fn entity_kind_key() -> String {
        String::from("TodoItem")
    }
    fn entity_name_key(&self) -> String {
        self.name.clone()
    }
}
// INIT
let db = DatastoreClient::new().unwrap();
let item = TodoItem {
    name: String::from("test"),
    title: String::from("lorem ipsum")
};
// GO!
db.upsert(item);
```


