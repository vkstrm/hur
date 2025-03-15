use super::headers::Headers;
use crate::error;
use crate::error::Error;
use std::convert::TryInto;
use std::io::BufRead;

const CRLF_LEN: usize = "\r\n".len();

#[derive(serde::Serialize, Debug)]
pub struct Response {
    pub protocol: String,
    #[serde(rename = "statusCode")]
    pub status_code: u32,
    #[serde(rename = "reasonPhrase")]
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

        if (100..=199).contains(&status_code) || status_code == 204 || status_code == 304 {
            return Ok(Response {
                protocol,
                status_code,
                reason_phrase,
                headers,
                body: None,
            });
        }

        let body = if let Some(encoding) = headers.get("Transfer-Encoding") {
            let encoding = encoding[0].as_str();
            // TODO Check if header contains commas example "chunked, gzip"
            match encoding {
                "chunked" => chunked_body(bottom)?,
                _ => None,
            }
        } else if let Some(encoding) = headers.get("Content-Length") {
            let length = encoding[0].parse::<usize>().unwrap();
            Some(content_length_body(length, bottom)?)
        } else {
            Some(String::from_utf8(bottom.to_vec()).unwrap())
        };

        Ok(Response {
            protocol,
            status_code,
            reason_phrase,
            headers,
            body,
        })
    }
}

fn get_status_line(head: &[u8]) -> Result<(String, &[u8]), Error> {
    if let Some(line) = head.iter().as_slice().lines().next() {
        match line {
            Ok(l) => {
                let len = l.len();
                Ok((l, &head[len + CRLF_LEN..]))
            }
            Err(why) => error!(&why.to_string()),
        }
    } else {
        error!("no status line in response")
    }
}

fn parse_status_line(status_line: &str) -> Result<(String, u32, String), Error> {
    let mut splits: Vec<String> = status_line.splitn(3, ' ').map(String::from).collect();
    if splits.len() != 3 {
        error!("improper status line")
    }
    let reason_phrase = splits.pop().unwrap();
    let status_code = splits.pop().unwrap().parse::<u32>().unwrap();
    let protocol = splits.pop().unwrap();
    Ok((protocol, status_code, reason_phrase))
}

fn collect_head(buf: &[u8]) -> &[u8] {
    let mut taken = 0;
    for line in buf.iter().as_slice().lines().map_while(Result::ok) {
        if line.is_empty() {
            break;
        }
        taken += line.len() + CRLF_LEN;
    }

    &buf[..taken]
}

fn collect_body(from_index: usize, buffer: &[u8]) -> &[u8] {
    &buffer[from_index..]
}

fn collect_headers(head: &[u8]) -> Result<Headers, Error> {
    let mut headers = Headers::new();
    let lines = head.iter().as_slice().lines();
    for line_res in lines {
        match line_res {
            Ok(line) => {
                let splits: Vec<&str> = line.splitn(2, ':').collect();
                if splits.len() != 2 {
                    error!("incorrect header format")
                }
                headers.add(splits[0].trim(), splits[1].trim());
            }
            Err(error) => error!(&error.to_string()),
        }
    }
    Ok(headers)
}

fn chunked_body(buf: &[u8]) -> Result<Option<String>, Error> {
    let mut lines = buf.iter().as_slice().lines();
    lines.next(); // Advance past empty line
    let (chunk_size, chunk_line_size) = if let Some(line_res) = lines.next() {
        match line_res {
            Ok(line) => (hexstr_to_dec(&line), line.len() + CRLF_LEN),
            Err(why) => error!(&why.to_string()),
        }
    } else {
        (0, 0)
    };

    if chunk_size < buf[CRLF_LEN + chunk_line_size..].len() {
        // Why does times 3 work?
        let body =
            String::from_utf8_lossy(&buf[CRLF_LEN + chunk_line_size..(CRLF_LEN * 3) + chunk_size]);

        return Ok(Some(body.to_string()));
    }

    Ok(None)
}

fn content_length_body(content_length: usize, buf: &[u8]) -> Result<String, Error> {
    let mut lines = buf.iter().as_slice().lines();
    lines.next();
    let body_vec = buf[CRLF_LEN..content_length + CRLF_LEN].to_vec();
    match String::from_utf8(body_vec) {
        Ok(body) => Ok(body),
        Err(why) => error!(&why.to_string()),
    }
}

fn hexstr_to_dec(s: &str) -> usize {
    let mut sum = 0;
    let mut n: u32 = s.len().try_into().unwrap();
    for x in s.to_lowercase().chars() {
        n = n.saturating_sub(1);
        let with_n = u32::pow(16, n);
        if let Some(digit) = x.to_digit(10) {
            sum += digit * with_n;
        } else {
            sum += match x {
                'a' => 10 * with_n,
                'b' => 11 * with_n,
                'c' => 12 * with_n,
                'd' => 13 * with_n,
                'e' => 14 * with_n,
                'f' => 15 * with_n,
                _ => 0,
            };
        }
    }
    sum as usize
}

#[test]
fn hex_test() {
    assert_eq!(hexstr_to_dec("3B"), 59);
    assert_eq!(hexstr_to_dec("E7A9"), 59305);
}

#[test]
fn test_parse_status_line() {
    let status_line = "HTTP/1.1 400 Bad Request";
    let (protocol, status_code, reason_phrase) = parse_status_line(status_line).unwrap();
    assert_eq!(protocol, "HTTP/1.1");
    assert_eq!(status_code, 400);
    assert_eq!(reason_phrase, "Bad Request");
}

#[test]
fn test_collect_headers() {
    let head = r#"Content-Type:application/json
    Content-Length:5000"#;
    let headers = collect_headers(head.as_bytes()).unwrap();
    assert_eq!(
        headers.get("Content-Type"),
        Some(&vec!["application/json".to_string()])
    )
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
