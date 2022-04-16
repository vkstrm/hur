use std::{collections::HashMap, fmt::Display};

use serde::ser::SerializeMap;

/**
 * A sender MUST NOT generate multiple header fields with the same field
   name in a message unless either the entire field value for that
   header field is defined as a comma-separated list [i.e., #(values)]
   or the header field is a well-known exception (as noted below).
 */

#[derive(Debug)]
pub struct Headers {
    pub headers_map: std::collections::HashMap<String, Vec<String>>
}

impl serde::Serialize for Headers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
            S: serde::Serializer {
        let mut map = serializer.serialize_map(Some(self.headers_map.len()))?;
        for (k, v) in &self.headers_map {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl Headers {
    pub fn new() -> Headers {
        Headers {
            headers_map: HashMap::<String, Vec<String>>::new()
        }
    }

    pub fn add(&mut self, key: &str, value: &str) {
        if self.headers_map.contains_key(key) {
            match self.headers_map.get_mut(key) {
                Some(vec) => vec.push(value.to_string()),
                None => panic!("no vec for key"),
            }
        } else {
            self.headers_map.insert(key.to_string(), vec![value.to_string()]);
        }
    }

    pub fn append(&mut self, other: Headers) {
        for (key, val) in other.headers_map {
            let key = capitalize(&key);
            match key.as_str() {
                "Connection" | "Host" => {
                    self.headers_map.insert(key, val);
                },
                _ => {
                    for v in val {
                        self.add(&key, &v);
                    }
                }
            }
        }
    }

    pub fn has(&self, header: &str) -> Option<&Vec<String>> {
        self.headers_map.get(header)
    }
}

impl Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.headers_map)
    }
}

fn capitalize(string: &str) -> String {
    let mut chars = string.chars();
    return chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str();
}