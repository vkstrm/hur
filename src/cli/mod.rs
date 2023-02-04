use std::io::Read;

use super::http::{headers::Headers, Method};
use super::logs;

use crate::error;
use crate::error::Error;
use crate::http::request::Request;

use clap::{crate_authors, crate_name, crate_version, Arg, ArgMatches};

pub mod output;
use output::Output;
use url::Url;

struct Input {
    pub url: String,
    pub method: Method,
    pub headers: Headers,
    pub body: Option<String>,
    pub allow_proxy: bool,
    pub timeout: u64,
}

pub fn create_request(args: Vec<String>) -> Result<(Request, Output), Error> {
    let (input, output) = parse_args(args)?;
    let parsed_url = parse_url(&input.url)?;

    let mut request = match input.body {
        Some(body) => Request::with_body(parsed_url, input.method, input.headers, &body),
        None => Request::new(parsed_url, input.method, input.headers),
    }?;

    if !input.allow_proxy {
        request.disable_proxy();
    }

    request.set_timeout(input.timeout);

    Ok((request, output))
}

fn parse_url(url: &str) -> Result<Url, Error> {
    let parsed_url = match Url::parse(url) {
        Ok(url) => url,
        Err(why) => error!(&why.to_string()),
    };
    if !parsed_url.has_host() {
        error!("no host in input");
    }
    Ok(parsed_url)
}

fn parse_args(args: Vec<String>) -> Result<(Input, Output), Error> {
    let command = use_clap();
    let matches = command.get_matches_from(args);
    let input = parse_input(&matches)?;
    let output = parse_output(&matches);
    
    if matches.get_flag("info") {
        enable_logging()?;
    }

    Ok((input, output))
}

fn parse_input(matches: &ArgMatches) -> Result<Input, Error> {
    let mut headers = headers(matches)?;
    let body = parse_body(matches, &mut headers)?;

    let timeout = match matches.get_one::<u64>("timeout") {
        Some(timeout) => timeout.to_owned(),
        None => 10,
    };

    Ok(Input {
        url: matches.get_one::<String>("url").unwrap().to_owned(),
        method: get_method(matches.get_one::<String>("method").unwrap()),
        headers,
        body,
        allow_proxy: !(matches.get_flag("no-proxy")),
        timeout,
    })
}

fn parse_body(matches: &ArgMatches, headers: &mut Headers) -> Result<Option<String>, Error> {
    if let Some(body) = matches.get_one::<String>("body") {
        return Ok(Some(body.to_string()));
    }
    if let Some(body) = matches.get_one::<String>("json") {
        match serde_json::from_str::<serde_json::Value>(body) {
            Ok(_) => {
                headers.add("Content-Type", "application/json");
                return Ok(Some(body.to_string()));
            }
            Err(why) => error!(&why.to_string()),
        };
    }
    if let Some(path) = matches.get_one::<String>("body-file") {
        let file_str = read_file(path)?;
        if path.ends_with(".json") {
            headers.add("Content-Type", "application/json");
        }
        return Ok(Some(file_str));
    }

    Ok(None)
}

fn parse_output(matches: &ArgMatches) -> Output {
    Output {
        verbose: matches.get_flag("verbose"),
        no_body: matches.get_flag("no-body"),
    }
}

fn headers(matches: &ArgMatches) -> Result<Headers, Error> {
    let mut headers = match matches.get_many::<String>("header") {
        Some(headers) => {
            let h: Vec<&String> = headers.collect();
            single_headers(h)?
        }
        None => Headers::new(),
    };

    if let Some(h) = matches.get_one::<String>("headers") {
        let h = json_headers(h)?;
        headers.append(h)
    };

    Ok(headers)
}

fn single_headers(headers: Vec<&String>) -> Result<Headers, Error> {
    let mut new_headers = Headers::new();
    for header in headers {
        let (key, val) = header_key_val(header)?;
        new_headers.add(key, val);
    }
    Ok(new_headers)
}

fn header_key_val(header: &String) -> Result<(&str, &str), Error> {
    let splits: Vec<&str> = header.splitn(2, ':').collect();
    if splits.len() < 2 {
        error!(&format!("invalid header \"{}\"", header))
    }
    Ok((splits[0].trim(), splits[1].trim()))
}

fn json_headers(headers_json: &str) -> Result<Headers, Error> {
    let json_string = match headers_json.ends_with(".json") {
        true => read_file(headers_json)?,
        false => headers_json.to_string(),
    };
    let map: std::collections::HashMap<String, String> = serde_json::from_str(&json_string)?;
    Ok(Headers::from(map))
}

fn read_file(path: &str) -> Result<String, Error> {
    let path = std::path::PathBuf::from(path);
    let mut file = std::fs::File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

fn enable_logging() -> Result<(), log::SetLoggerError> {
    static LOGGER: logs::Logger = logs::Logger;
    log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info))
}

fn get_method(method: &str) -> Method {
    match method.to_lowercase().as_str() {
        "get" => Method::Get,
        "post" => Method::Post,
        "put" => Method::Put,
        "delete" => Method::Delete,
        "patch" => Method::Patch,
        "connect" => Method::Connect,
        "options" => Method::Options,
        "trace" => Method::Trace,
        "head" => Method::Head,
        _ => Method::Get,
    }
}

fn use_clap() -> clap::Command {
    clap::command!(crate_name!())
        .disable_help_flag(true) // Help how???
        .arg_required_else_help(true)
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .arg(
            Arg::new("url")
                .help("The URL for the request")
                .required(true),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .help("Full request and response output in JSON")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("info")
                .long("info")
                .help("Show info logging")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("method")
                .help("The HTTP method to use for the request")
                .short('m')
                .long("method")
                .default_value("get")
                .hide_possible_values(true)
                .value_parser([
                    "get", "GET", "post", "POST", "put", "PUT", "trace", "TRACE", "patch", "PATCH",
                    "delete", "DELETE", "head", "HEAD", "options", "OPTIONS", "connect", "CONNECT",
                ]),
        )
        .arg(
            Arg::new("header")
                .help("Header for request")
                .short('h')
                .long("header")
                .action(clap::ArgAction::Append),
        )
        .arg(
            Arg::new("headers")
                .help("Headers as a JSON string or JSON file")
                .long("headers"),
        )
        .arg(
            Arg::new("body")
                .help("Body for request")
                .long("body")
                .conflicts_with("json"),
        )
        .arg(
            Arg::new("json")
                .help("Send body with Content-Type:application/json")
                .long("json")
                .conflicts_with("body"),
        )
        .arg(
            Arg::new("body-file")
                .help("Supply path to file to use as body")
                .long("body-file")
                .conflicts_with("body")
                .conflicts_with("json"),
        )
        .arg(
            Arg::new("no-body")
                .help("Don't print response body")
                .long("no-body")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-proxy")
                .help("Do not proxy request")
                .long("no-proxy")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("timeout")
                .help("The read timeout in seconds for the request")
                .long("timeout")
                .value_parser(clap::value_parser!(u64)),
        )
}

#[test]
fn test_valid_single_headers() {
    let headers = vec![
        "key:value".to_string(),
        "key2: value".to_string(),
        "key3 :value".to_string(),
        "key4 : value".to_string(),
    ];
    let headers = single_headers(headers.iter().collect());
    assert!(headers.is_ok());
    let headers = headers.unwrap();
    assert!(headers.get("key").is_some());
    assert!(headers.get("key2").is_some());
    assert!(headers.get("key3").is_some());
    assert!(headers.get("key4").is_some());
}

#[test]
fn test_invalid_headers() {
    let headers = vec![
        "key".to_string(),
        "key=value".to_string(),
        "key value".to_string(),
    ];
    for header in headers {
        let r = header_key_val(&header);
        assert!(r.is_err());
    }
}

#[test]
fn test_collect_body() {
    let input = vec!["hur", "http://localhost", "--body", "form:value"];
    let command = use_clap();
    let matches = command.get_matches_from(input);

    let mut headers = Headers::new();
    let body = parse_body(&matches, &mut headers).unwrap();

    assert!(body.is_some());
    assert_eq!(body.unwrap(), "form:value");
}

#[test]
fn test_collect_json_body() {
    let input = vec!["hur", "http://localhost", "--json", r#"{"key":"value"}"#];
    let command = use_clap();
    let matches = command.get_matches_from(input);

    let mut headers = Headers::new();
    let body = parse_body(&matches, &mut headers).unwrap();

    assert!(body.is_some());
    assert_eq!(body.unwrap(), r#"{"key":"value"}"#);
    assert!(headers.get("Content-Type").is_some());
}

#[test]
fn test_data_body() {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("data.json");
    let input = vec![
        "hur",
        "http://localhost",
        "--body-file",
        temp_file.to_str().unwrap(),
    ];
    let command = use_clap();
    let matches = command.get_matches_from(input);
    let mut file = std::fs::File::create(temp_file).unwrap();
    use std::io::Write;
    file.write(br#"{"key":"value"}"#).unwrap();

    let mut headers = Headers::new();
    let body = parse_body(&matches, &mut headers).unwrap();

    assert!(body.is_some());
    assert_eq!(body.unwrap(), r#"{"key":"value"}"#);
    assert!(headers.get("Content-Type").is_some());
}
