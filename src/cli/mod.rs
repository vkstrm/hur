use std::io::Read;

use super::http::{headers::Headers, Method};
use super::logs;

use crate::error;
use crate::error::Error;

use clap::{crate_authors, crate_name, crate_version, Arg, ArgMatches};

pub mod output;
use output::Output;

pub struct Input {
    pub url: String,
    pub method: Method,
    pub headers: Headers,
    pub body: Option<String>,
    pub allow_proxy: bool,
    pub timeout: u64,
}

pub fn parse_args(args: Vec<String>) -> Result<(Input, Output), Error> {
    let matches = use_clap(&args);
    let input = parse_input(&matches)?;
    let output = parse_output(&matches);
    if matches.get_flag("info") {
        enable_logging()?;
    }

    Ok((input, output))
}

fn parse_input(matches: &ArgMatches) -> Result<Input, Error> {
    let mut headers = headers(matches)?;
    let body = collect_body(
        matches.get_one::<String>("body"),
        matches.get_one::<String>("json"),
        &mut headers,
    )?;

    let timeout = match matches.get_one::<u64>("timeout") {
        Some(timeout) => timeout.to_owned(), // TODO Solve better
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
            single_headers(h)
        }
        None => Headers::new(),
    };

    if let Some(h) = matches.get_one::<String>("headers") {
        let h = json_headers(h)?;
        headers.append(h)
    };

    Ok(headers)
}

fn single_headers(headers: Vec<&String>) -> Headers {
    let mut new_headers = Headers::new();
    for val in headers {
        let splits: Vec<&str> = val.splitn(2, ':').collect();
        new_headers.add(splits[0], splits[1]);
    }
    new_headers
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
    let mut file = std::fs::File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

fn collect_body(
    body_option: Option<&String>,
    json_option: Option<&String>,
    headers: &mut Headers,
) -> Result<Option<String>, Error> {
    let mut body: Option<String> = None;
    if let Some(body_str) = body_option {
        body = Some(body_str.to_string());
    }
    if let Some(body_str) = json_option {
        match serde_json::from_str::<serde_json::Value>(body_str) {
            Ok(_) => {
                body = Some(body_str.to_string());
                headers.add("Content-Type", "application/json");
            }
            Err(why) => error!(&why.to_string()),
        };
    }

    Ok(body)
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

fn use_clap(args: &[String]) -> ArgMatches {
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
                .long("timeout"),
        )
        .get_matches_from(args)
}

#[test]
fn test_collect_body() {
    let mut headers = Headers::new();
    let input = String::from("form1:value2");
    let body_opt = Some(&input);
    let json_opt = None;

    let body = collect_body(body_opt, json_opt, &mut headers).unwrap();
    assert!(body.is_some());
    assert_eq!(body.unwrap(), "form1:value2");
}

#[test]
fn test_collect_json_body() {
    let mut headers = Headers::new();
    let input = String::from(r#"{"key":"value"}"#);
    let body_opt = None;
    let json_opt = Some(&input);

    let body = collect_body(body_opt, json_opt, &mut headers).unwrap();
    assert!(body.is_some());
    assert_eq!(body.unwrap(), r#"{"key":"value"}"#);
    assert!(headers.get("Content-Type").is_some());
}
