use std::rc::Rc;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::PathBuf;
use std::string::ToString;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use crate::convert;

pub use crate::api_key::ApiKey;

///////////////////////////////////////////////////////////////////////////////
// HELPERS
///////////////////////////////////////////////////////////////////////////////

pub trait EntityKey {
    fn entity_kind_key() -> String;
    fn entity_name_key(&self) -> String;
}


#[derive(Debug)]
pub enum Error {
    Serialization {
        msg: String,
    },
    Deserialization {
        msg: String,
    },
    DatabaseResponse(google_datastore1::Error),
    NoPayload,
}


///////////////////////////////////////////////////////////////////////////////
// CLIENT
///////////////////////////////////////////////////////////////////////////////

type Handle = google_datastore1::Datastore<hyper::Client, yup_oauth2::ServiceAccountAccess<hyper::Client>>;

#[derive(Clone)]
pub struct DatastoreClient {
    handle: Rc<Handle>,
    project_id: String,
}

impl DatastoreClient {
    pub fn new(auth: crate::api_key::ApiKey) -> Self {
        let project_id = auth.project_id.clone();
        let key_file = auth.file_path
            .to_str()
            .expect("auth.file_path.to_str() failed");
        let client_secret = yup_oauth2::service_account_key_from_file(&key_file.to_owned())
            .expect("yup_oauth2::service_account_key_from_file failed");
        let client = hyper::Client::with_connector(
            hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new())
        );
        let access = yup_oauth2::ServiceAccountAccess::new(client_secret, client);
        let client = hyper::Client::with_connector(
            hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new())
        );
        let hub = google_datastore1::Datastore::new(client, access);
        let hub = Rc::new(hub);
        DatastoreClient {
            handle: hub,
            project_id
        }
    }
    pub fn insert<T: Serialize + EntityKey>(&self, value: T) -> Result<(), Error> {
        let kind_key = T::entity_kind_key();
        let name_key = value.entity_name_key();
        let properties = convert::to_datastore_value(value)
            .and_then(|value| {
                value.entity_value
            })
            .and_then(|x| x.properties)
            .ok_or(Error::Serialization {
                msg: String::from("expecting struct/map like input")
            })?;
        let entity = google_datastore1::Entity {
            properties: Some(properties),
            key: Some(google_datastore1::Key {
                path: Some(vec![
                    google_datastore1::PathElement {
                        kind: Some(kind_key.to_owned()),
                        name: Some(name_key.to_owned()),
                        id: None
                    }
                ]),
                partition_id: None
            })
        };
        let req = google_datastore1::CommitRequest {
            transaction: None,
            mutations: Some(vec![
                google_datastore1::Mutation {
                    insert: Some(entity),
                    delete: None,
                    update: None,
                    base_version: None,
                    upsert: None
                }
            ]),
            mode: Some(String::from("NON_TRANSACTIONAL"))
        };
        let result = self.handle
            .projects()
            .commit(req, &self.project_id)
            .doit();
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::DatabaseResponse(e))
        }
    }
    pub fn upsert<T: Serialize + EntityKey>(&self, value: T) -> Result<(), Error> {
        let kind_key = T::entity_kind_key();
        let name_key = value.entity_name_key();
        let properties = convert::to_datastore_value(value)
            .and_then(|value| {
                value.entity_value
            })
            .and_then(|x| x.properties)
            .ok_or(Error::Serialization {
                msg: String::from("expecting struct/map like input")
            })?;
        let entity = google_datastore1::Entity {
            properties: Some(properties),
            key: Some(google_datastore1::Key {
                path: Some(vec![
                    google_datastore1::PathElement {
                        kind: Some(kind_key.to_owned()),
                        name: Some(name_key.to_owned()),
                        id: None
                    }
                ]),
                partition_id: None
            })
        };
        let req = google_datastore1::CommitRequest {
            transaction: None,
            mutations: Some(vec![
                google_datastore1::Mutation {
                    insert: None,
                    delete: None,
                    update: None,
                    base_version: None,
                    upsert: Some(entity),
                }
            ]),
            mode: Some(String::from("NON_TRANSACTIONAL"))
        };
        let result = self.handle
            .projects()
            .commit(req, &self.project_id)
            .doit();
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::DatabaseResponse(e))
        }
    }
    pub fn update<T: Serialize + EntityKey>(&self, value: T) -> Result<(), Error> {
        let kind_key = T::entity_kind_key();
        let name_key = value.entity_name_key();
        let properties = convert::to_datastore_value(value)
            .and_then(|value| {
                value.entity_value
            })
            .and_then(|x| x.properties)
            .ok_or(Error::Serialization {
                msg: String::from("expecting struct/map like input")
            })?;
        let entity = google_datastore1::Entity {
            properties: Some(properties),
            key: Some(google_datastore1::Key {
                path: Some(vec![
                    google_datastore1::PathElement {
                        kind: Some(kind_key.to_owned()),
                        name: Some(name_key.to_owned()),
                        id: None
                    }
                ]),
                partition_id: None
            })
        };
        let req = google_datastore1::CommitRequest {
            transaction: None,
            mutations: Some(vec![
                google_datastore1::Mutation {
                    insert: None,
                    delete: None,
                    update: Some(entity),
                    base_version: None,
                    upsert: None,
                }
            ]),
            mode: Some(String::from("NON_TRANSACTIONAL"))
        };
        let result = self.handle
            .projects()
            .commit(req, &self.project_id)
            .doit();
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::DatabaseResponse(e))
        }
    }
    pub fn get<T: DeserializeOwned + EntityKey, K: ToString>(&self, name_key: K) -> Result<T, Error> {
        let kind_key = T::entity_kind_key();
        let req = google_datastore1::LookupRequest {
            keys: Some(vec![
                google_datastore1::Key {
                    path: Some(vec![
                        google_datastore1::PathElement {
                            kind: Some(kind_key),
                            name: Some(name_key.to_string()),
                            id: None
                        }
                    ]),
                    partition_id: None
                }]),
            read_options: None
        };
        let result = self.handle
            .projects()
            .lookup(req, &self.project_id)
            .doit();
        match result {
            Ok((_, lookup_response)) => {
                let payload = lookup_response.found
                    .and_then(|entities| {
                        entities.first().map(|x| x.clone())
                    })
                    .and_then(|x| x.entity)
                    .ok_or(Error::NoPayload)?;
                convert::from_datastore_entity(payload.clone())
                    .ok_or_else(|| {
                        eprintln!("conversion or parser error: {:#?}", payload);
                        Error::Deserialization {
                            msg: String::from("conversion or parser error")
                        }
                    })
            }
            Err(e) => Err(Error::DatabaseResponse(e)),
        }
    }
    pub fn delete<T: EntityKey, K: ToString>(&self, name_key: K) -> Result<(), Error> {
        let kind_key = T::entity_kind_key();
        let name_key = name_key.to_string();
        let entity_key = google_datastore1::Key {
            path: Some(vec![
                google_datastore1::PathElement {
                    kind: Some(kind_key.to_owned()),
                    name: Some(name_key.to_owned()),
                    id: None
                }
            ]),
            partition_id: None
        };
        let req = google_datastore1::CommitRequest {
            transaction: None,
            mutations: Some(vec![
                google_datastore1::Mutation {
                    insert: None,
                    delete: Some(entity_key),
                    update: None,
                    base_version: None,
                    upsert: None,
                }
            ]),
            mode: Some(String::from("NON_TRANSACTIONAL"))
        };
        let result = self.handle
            .projects()
            .commit(req, &self.project_id)
            .doit();
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::DatabaseResponse(e))
        }
    }
}
