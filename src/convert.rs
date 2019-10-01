use std::str::FromStr;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub fn to_datastore_value<T: Serialize>(x: T) -> Option<google_datastore1::Value> {
    match serde_json::to_value(x).ok()? {
        serde_json::Value::Null => {
            Some(google_datastore1::Value {
                null_value: Some(String::from("NULL_VALUE")),
                ..google_datastore1::Value::default()
            })
        }
        serde_json::Value::Bool(x) => {
            Some(google_datastore1::Value {
                boolean_value: Some(x),
                ..google_datastore1::Value::default()
            })
        }
        serde_json::Value::Number(x) => {
            if x.is_f64() {
                Some(google_datastore1::Value {
                    double_value: Some(x.as_f64().expect("should be f64")),
                    ..google_datastore1::Value::default()
                })
            } else if x.is_i64() {
                Some(google_datastore1::Value {
                    integer_value: Some(format!("{}", x.as_i64().expect("should be i64"))),
                    ..google_datastore1::Value::default()
                })
            } else if x.is_u64() {
                Some(google_datastore1::Value {
                    integer_value: Some(format!("{}", x.as_u64().expect("should be u64"))),
                    ..google_datastore1::Value::default()
                })
            } else {
                panic!("ddb from_datastore_value unreachable");
            }
        }
        serde_json::Value::String(x) => {
            Some(google_datastore1::Value {
                string_value: Some(x),
                ..google_datastore1::Value::default()
            })
        }
        serde_json::Value::Array(xs) => {
            let mut any_invalid = false;
            let xs = xs
                .into_iter()
                .filter_map(|x| {
                    let x = to_datastore_value(x);
                    if x.is_none() {
                        any_invalid = true;
                    }
                    x
                })
                .collect::<Vec<_>>();
            if !any_invalid {
                Some(google_datastore1::Value {
                    array_value: Some(google_datastore1::ArrayValue {
                        values: Some(xs)
                    }),
                    ..google_datastore1::Value::default()
                })
            } else {
                None
            }
        }
        serde_json::Value::Object(xs) => {
            let mut any_invalid = false;
            let xs = xs
                .into_iter()
                .filter_map(|(k, v)| {
                    let v = to_datastore_value(v);
                    if v.is_none() {
                        any_invalid = true;
                    }
                    v.map(|v| (k, v))
                })
                .collect::<HashMap<_, _>>();
            Some(google_datastore1::Value {
                entity_value: Some(google_datastore1::Entity {
                    properties: Some(xs),
                    key: None,
                }),
                ..google_datastore1::Value::default()
            })
        }
    }
}

pub fn from_datastore_value<T: serde::de::DeserializeOwned>(value: google_datastore1::Value) -> Option<T> {
    let mut serde_value: serde_json::Value;
    if let Some(xs) = value.entity_value {
        let mut any_invalid = false;
        let xs = xs
            .properties
            .unwrap_or(HashMap::default())
            .into_iter()
            .filter_map(|(k, v)| {
                let v = from_datastore_value(v);
                if v.is_none() {
                    any_invalid = true;
                }
                v.map(|v| (k, v))
            })
            .collect::<serde_json::Map<_, _>>();
        if !any_invalid {
            serde_value = serde_json::Value::Object(xs);
        } else {
            return None;
        }
    } else if let Some(xs) = value.timestamp_value {
        serde_value = serde_json::Value::String(xs);
    } else if let Some(xs) = value.geo_point_value {
        serde_value = unimplemented!();
    } else if let Some(xs) = value.blob_value {
        serde_value = unimplemented!();
    } else if let Some(xs) = value.double_value {
        serde_value = serde_json::Value::Number(serde_json::Number::from_f64(xs)?);
    } else if let Some(xs) = value.meaning {
        serde_value = unimplemented!();
    } else if let Some(xs) = value.exclude_from_indexes {
        serde_value = unimplemented!();
    } else if let Some(xs) = value.string_value {
        serde_value = serde_json::Value::String(xs);
    } else if let Some(xs) = value.key_value {
        serde_value = unimplemented!();
    } else if let Some(xs) = value.boolean_value {
        serde_value = serde_json::Value::Bool(xs);
    } else if let Some(xs) = value.array_value {
        let mut any_invalid = false;
        let xs = xs
            .values
            .unwrap_or(Vec::default())
            .into_iter()
            .filter_map(|x| {
                let x = from_datastore_value(x);
                any_invalid = true;
                x
            })
            .collect::<Vec<_>>();
        if !any_invalid {
            serde_value = serde_json::Value::Array(xs);
        } else {
            return None;
        }
    } else if let Some(xs) = value.integer_value {
        let as_u64: Option<u64> = FromStr::from_str(&xs).ok();
        let as_i64: Option<i64> = FromStr::from_str(&xs).ok();
        let number: serde_json::Number = as_u64
            .map(|x| From::from(x))
            .or(as_i64.map(|x| From::from(x)))?;
        serde_value = serde_json::Value::Number(number);
    } else if let Some(xs) = value.null_value {
        serde_value = serde_json::Value::Null;
    } else {
        serde_value = serde_json::Value::Null;
    }
    serde_json::from_value(serde_value)
        .map_err(|e| {
            // eprintln!("error serde_json::from_value: {:#?}", e);
            e
        })
        .ok()
}


pub fn from_datastore_entity<T: serde::de::DeserializeOwned>(value: google_datastore1::Entity) -> Option<T> {
    let value = google_datastore1::Value {
        entity_value: Some(value),
        ..Default::default()
    };
    from_datastore_value(value)
}



