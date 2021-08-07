use std::collections::HashMap;

#[derive(Debug)]
pub struct Headers {
    pub headers_map: std::collections::HashMap<String, Vec<String>>
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
                "Connection" | "connection" | "Host" | "host" => {
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
}

fn capitalize(string: &String) -> String {
    let mut chars = string.chars();
    return chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str();
}