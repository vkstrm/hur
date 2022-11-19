use std::collections::{hash_map, HashMap};
use std::fmt::Display;

use serde::ser::SerializeMap;

#[derive(Debug)]
pub struct Headers {
    internal_headers: HashMap<String, Vec<String>>,
}

impl serde::Serialize for Headers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.internal_headers.len()))?;
        for (k, v) in &self.internal_headers {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl Headers {
    pub fn new() -> Headers {
        Headers {
            internal_headers: HashMap::<String, Vec<String>>::new(),
        }
    }

    pub fn add(&mut self, key: &str, value: &str) {
        if self.internal_headers.contains_key(key) {
            match self.internal_headers.get_mut(key) {
                Some(vec) => vec.push(value.to_string()),
                None => panic!("no vec for key"),
            }
        } else {
            self.internal_headers
                .insert(key.to_string(), vec![value.to_string()]);
        }
    }

    pub fn append(&mut self, other: Headers) {
        for (key, val) in other.internal_headers {
            let key = capitalize(&key);
            match key.as_str() {
                "Connection" | "Host" => {
                    self.internal_headers.insert(key, val);
                }
                _ => {
                    for v in val {
                        self.add(&key, &v);
                    }
                }
            }
        }
    }

    pub fn get(&self, header: &str) -> Option<&Vec<String>> {
        match self.internal_headers.get(header) {
            Some(vec) => {
                if !vec.is_empty() {
                    Some(vec)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn iter(&self) -> HeaderIterator {
        HeaderIterator {
            iterator: self.internal_headers.iter(),
        }
    }
}

impl Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.internal_headers)
    }
}

impl From<HashMap<String, String>> for Headers {
    fn from(map: HashMap<String, String>) -> Self {
        let mut headers = Headers::new();
        for (key, value) in map {
            headers.add(&key, &value)
        }
        headers
    }
}

pub struct HeaderIterator<'a> {
    iterator: hash_map::Iter<'a, String, Vec<String>>,
}

impl<'a> Iterator for HeaderIterator<'a> {
    type Item = (&'a String, &'a Vec<String>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

impl std::iter::IntoIterator for Headers {
    type Item = (String, Vec<String>);
    type IntoIter = hash_map::IntoIter<String, Vec<String>>;

    fn into_iter(self) -> Self::IntoIter {
        self.internal_headers.into_iter()
    }
}

fn capitalize(string: &str) -> String {
    let mut chars = string.chars();
    return chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str();
}
