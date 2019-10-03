#![allow(unused)]
//! ```
//! use serde::{Serialize, Deserialize};
//! 
//! // MODEL
//! #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
//! pub struct TodoItem {
//!     pub name: String,
//!     pub title: String,
//! }
//! // MODEL METADATA
//! impl ddb::EntityKey for TodoItem {
//!     fn entity_kind_key() -> String {
//!         String::from("TodoItem")
//!     }
//!     fn entity_name_key(&self) -> String {
//!         self.name.clone()
//!     }
//! }
//! // INIT
//! let db = ddb::DatastoreClient::new().unwrap();
//! let item = TodoItem {
//!     name: String::from("test"),
//!     title: String::from("lorem ipsum")
//! };
//! // GO!
//! db.upsert(item);
//! ```

mod convert;
mod db;
mod auth;

pub use db::*;

