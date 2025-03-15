use std::collections::{HashMap, hash_map};
use std::fmt::Display;

use serde::ser::SerializeMap;

use crate::error;
use crate::error::Error;

#[derive(Clone)]
pub struct Header {
    key: String,
    value: String,
}

impl Header {
    pub fn try_from(header: &str) -> Result<Self, Error> {
        let splits: Vec<&str> = header.splitn(2, ':').collect();
        if splits.len() < 2 {
            error!(&format!("invalid header \"{}\"", header))
        }
        Ok(Header {
            key: splits[0].trim().to_string(),
            value: splits[1].trim().to_string(),
        })
    }
}

#[test]
fn test_valid_header() {
    let headers = [
        "key:value".to_string(),
        "key2: value".to_string(),
        "key3 :value".to_string(),
        "key4 : value".to_string(),
    ];
    headers.iter().for_each(|s| {
        let h = Header::try_from(s.as_str());
        assert!(h.is_ok());
    });
}

#[test]
fn test_invalid_headers() {
    let headers = vec![
        "key".to_string(),
        "key=value".to_string(),
        "key value".to_string(),
    ];
    for header in headers {
        let h = Header::try_from(header.as_str());
        assert!(h.is_err());
    }
}

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
    pub fn new() -> Self {
        Headers {
            internal_headers: HashMap::<String, Vec<String>>::new(),
        }
    }

    pub fn add(&mut self, key: &str, value: &str) {
        let key = key.to_lowercase();
        if self.internal_headers.contains_key(key.as_str()) {
            match self.internal_headers.get_mut(key.as_str()) {
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
            let key = key.to_lowercase();
            match key.as_str() {
                "connection" | "host" => {
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
        self.internal_headers
            .get(header.to_lowercase().as_str())
            .filter(|&vec| !vec.is_empty())
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

impl From<&Vec<Header>> for Headers {
    fn from(headers: &Vec<Header>) -> Self {
        let mut h = Headers::new();
        headers
            .iter()
            .for_each(|header| h.add(&header.key, &header.value));
        h
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
