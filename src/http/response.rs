use std::io::BufRead;

use super::headers::Headers;
use crate::error::Error;
use crate::error;

const CRLF_LEN: usize = "\r\n".as_bytes().len();

#[derive(serde::Serialize, Debug)]
pub struct Response {
    pub protocol: String,
    pub status_code: u32,
    pub reason_phrase: String,
    pub headers: Headers,
    pub body: Option<String>,
}

impl Response {
    pub fn from_buffer(buf: &[u8]) -> Result<Response, Error> {
        let head = collect_head(buf);
        let bottom = collect_body(head.len(), buf);
        let (status_line, head) = get_status_line(head)?;
        let (protocol, status_code, reason_phrase) = parse_status_line(&status_line)?;
        let headers = collect_headers(head)?;

        // Collect body
        // let mut body_string = String::new();
        // for line in lines {
        //     body_string.push_str(&format!("{line}\n",line = line));
        // }
        // let body = match body_string.is_empty() {
        //     true => None,
        //     false => Some(body_string),
        // };

        // https://www.rfc-editor.org/rfc/rfc7230#section-3.3.3
        

        // let body = match headers.has("Transfer-Encoding") {
        //     Some(header_vec) => {
        //         match header_vec[0].as_str() {
        //             "chunked" => collect_chunked_body(&mut lines),
        //             _ => panic!("unsupported transfer-encoding")
        //         }
        //     },
        //     None => None
        // };

        Ok(Response {
            protocol,
            status_code,
            reason_phrase,
            headers,
            body: None,
        })
    }
}

fn get_status_line(head: &[u8]) -> Result<(String, &[u8]), Error> {
    if let Some(line) = head.iter().as_slice().lines().next() {
        match line {
            Ok(l) => {
                let len = l.len();
                Ok((l, &head[len + CRLF_LEN..]))
            },
            Err(why) => error!(&why.to_string())
        }
    } else {
        error!("no status line in response")
    }
}

fn parse_status_line(status_line: &str) -> Result<(String, u32, String), Error> {
    let mut splits: Vec<String> = status_line.splitn(3, ' ').map(|s| String::from(s)).collect();
    if splits.len() != 3 {
        error!("improper status line")
    }
    let reason_phrase = splits.pop().unwrap();
    let status_code = splits.pop().unwrap().parse::<u32>().unwrap();
    let protocol = splits.pop().unwrap();
    Ok((protocol, status_code, reason_phrase))
}

#[test]
fn test_parse_status_line() {
    let status_line = "HTTP/1.1 400 Bad Request";
    let (protocol, status_code, reason_phrase) = parse_status_line(status_line).unwrap();
    assert_eq!(protocol, "HTTP/1.1");
    assert_eq!(status_code, 400);
    assert_eq!(reason_phrase, "Bad Request");
}

fn collect_head(buf: &[u8]) -> &[u8] {
    let mut taken = 0;
    let mut iter = buf.iter().as_slice().lines();
    while let Some(line) = iter.next() {
        match line {
            Ok(l) => {
                taken += l.len();
                if l.is_empty() {
                    break;
                }
            },
            _ => {}
        }
    }
    
    return &buf[..taken];
}

fn collect_body(from_index: usize, buffer: &[u8]) -> &[u8] {
    &buffer[from_index..]
}

#[test]
fn test_collect_top() {
    let input = "LINE 1\r\nLINE 2\r\n\r\nLINE 3\r\n\r\n";
    let b = collect_head(input.as_bytes());
    println!("{}", String::from_utf8(b.to_vec()).unwrap());
}

#[test]
fn test_empty_collect_top() {
    let input = "";
    let b = collect_head(input.as_bytes());
    assert_eq!(b.len(), 0);
}

fn collect_headers(head: &[u8]) -> Result<Headers, Error> {
    let mut headers = Headers::new();
    let mut lines = head.iter().as_slice().lines();
    while let Some(line_res) = lines.next() {
        match line_res {
            Ok(line) => {
                let splits: Vec<&str> = line.splitn(2, ':').collect();
                if splits.len() != 2 {
                    error!("incorrect header format")
                }
                headers.add(splits[0].trim(), splits[1].trim());        
            },
            Err(why) => error!(&why.to_string())
        }
    }
    Ok(headers)
}

#[test]
fn test_collect_headers() {
    let head = r#"Content-Type:application/json
    Content-Length:5000"#;
    let headers = collect_headers(head.as_bytes()).unwrap();
    assert_eq!(headers.has("Content-Type"), Some(&vec!["application/json".to_string()]))
}

// fn collect_chunked_body(lines: &mut Lines) -> Option<String> {
//     // let chunk_size = match lines.next() {
//     //     Some(size) => {
//     //         hexstr_to_dec(size)
//     //     },
//     //     None => return None,
//     // };

//     let mut body: Vec<String> = Vec::new();
//     let mut previous_empty = false;
//     while let Some(line) = lines.next() {
//         if line.is_empty() { previous_empty = true; }
//         if line == "0" && previous_empty { 
//             break; 
//         }
//         body.push(format!("{l}\n", l = line));
//     }
//     if body.is_empty() { return None; }
//     match body.last() {
//         Some(line) => {
//             if line == "\n" {
//                 body.pop();
//             }
//         },
//         None => unreachable!()
//     };
//     Some(String::from_iter(body.into_iter()))
// }

// fn hexstr_to_dec(s: &str) -> usize {
//     let mut sum = 0;
//     let mut n: u32 = s.len().try_into().unwrap();
//     for x in s.to_lowercase().chars() {
//         if n != 0 {
//             n -= 1;
//         }
//         let with_n = u32::pow(16, n);
//         if let Some(digit) = x.to_digit(10) {
//             sum += digit * with_n;
//         } else {
//             sum += match x {
//                 'a' => 10 * with_n,
//                 'b' => 11 * with_n,
//                 'c' => 12 * with_n,
//                 'd' => 13 * with_n,
//                 'e' => 14 * with_n,
//                 'f' => 15 * with_n,
//                 _ => 0, 
//             };
//         }
//     }
//     sum as usize
// }

// #[test]
// fn hex_test() {
//     assert_eq!(hexstr_to_dec("3B"), 59);
//     assert_eq!(hexstr_to_dec("E7A9"), 59305);
// }

// fn collect_body(lines: &mut Lines) -> Option<String> {
//     let mut body_string = String::new();
//     while let Some(line) = lines.next() {
//         body_string.push_str(&format!("{line}\n",line = line));
//     }

//     // let body_string = lines.collect::<String>();

//     // for line in lines {
//     //     body_string.push_str(&format!("{line}\n",line = line));
//     // }
//     match body_string.is_empty() {
//         true => None,
//         false => Some(body_string),
//     }
// }