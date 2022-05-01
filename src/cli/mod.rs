use super::http::{Method, headers::Headers};
use super::logs;

use crate::error::Error;
use crate::error;

use clap::{ArgMatches, Arg, crate_version, crate_authors, crate_name};

pub mod output;
use output::Output;

pub struct Input {
    pub url: String,
    pub method: Method,
    pub headers: Headers,
    pub body: Option<String>,
    pub allow_proxy: bool
}

pub fn parse_args(args: Vec<String>) -> Result<(Input, Output), Error> {
    let matches = use_clap(&args);
    let input = parse_input(&matches)?;
    let output = parse_output(&matches);
    if matches.is_present("info") {
        enable_logging()?;
    }

    Ok((input, output))
}

fn parse_input(matches: &ArgMatches) -> Result<Input, Error> {
    let mut headers = collect_headers(matches.values_of("header"));
    let body = collect_body(
        matches.value_of("body"), 
        matches.value_of("json"), 
        &mut headers)?;

    Ok(Input{
        url: matches.value_of("url").unwrap().to_string(),
        method: get_method(matches.value_of("method").unwrap()),
        headers,
        body,
        allow_proxy: !(matches.is_present("no-proxy")),
    })
}

fn parse_output(matches: &ArgMatches) -> Output {
    Output {
        verbose: matches.is_present("verbose"),
        query_header: matches.value_of("query-header").map(|q| String::from(q.trim())),
        no_body: matches.is_present("no-body"),
    }
}

fn collect_headers(headers_option: Option<clap::Values>) -> Headers {
    match headers_option {
        Some(values) => {
            let mut headers = Headers::new();
            for val in values {
                let splits: Vec<&str> = val.splitn(2, ':').collect();
                headers.add(splits[0], splits[1]);
            }
            headers
        },
        None => Headers::new()
    }
}

fn collect_body(body_option: Option<&str>, json_option: Option<&str>, headers: &mut Headers) -> Result<Option<String>, Error> {
    let mut body: Option<String> = None;
    if let Some(body_str) = body_option {
        body = Some(body_str.to_string());
    }
    if let Some(body_str) = json_option {
        match serde_json::from_str::<serde_json::Value>(body_str) {
            Ok(_) => {
                body = Some(body_str.to_string());
                headers.add("Content-Type", "application/json");
            },
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
        "get" => Method::GET,
        "post" => Method::POST,
        "put" => Method::PUT,
        "delete" => Method::DELETE,
        "patch" => Method::PATCH,
        "connect" => Method::CONNECT,
        "options" => Method::OPTIONS,
        "trace" => Method::TRACE,
        "head" => Method::HEAD, 
        _ => Method::GET,
    }
}

fn use_clap(args: &[String]) -> ArgMatches {
    return
    clap::app_from_crate!(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .help("Full request and response output in JSON")
        )
        .arg(
            Arg::new("info")
                .long("info")
                .help("Show info logging")
        )
        .arg(
            Arg::new("url")
                .required(true)
                .takes_value(true)
        )
        .arg(
            Arg::new("method")
                .takes_value(true)
                .short('m')
                .default_value("get")
                .possible_values(&[
                    "get",
                    "post",
                    "put",
                    "trace",
                    "patch",
                    "delete",
                    "head",
                    "options",
                    "connect"
                    ])
        )
        .arg(
            Arg::new("header")
                .help("Header for request")
                .takes_value(true)
                .short('h')
                .long("header")
                .multiple_occurrences(true)
        )
        .arg(
            Arg::new("body")
                .help("Body for request")
                .long("body")
                .conflicts_with("json")
                .takes_value(true)
        )
        .arg(
            Arg::new("json")
                .help("Send body with Content-Type:application/json")
                .long("json")
                .conflicts_with("body")
                .takes_value(true)
        )
        .arg(
            Arg::new("query-header")
                .help("Query for a specific header from the response")
                .long("query-header")
                .takes_value(true)
        )
        .arg(
            Arg::new("no-body")
                .help("Don't print response body")
                .long("no-body")
        )
        .arg(
            Arg::new("no-proxy")
                .help("Do not proxy request")
                .long("no-proxy")
        )
        .get_matches_from(args);
}

#[test]
fn test_collect_body() {
    let mut headers = Headers::new();
    let body_opt = Some("form1:value2");
    let json_opt = None;

    let body = collect_body(body_opt, json_opt, &mut headers).unwrap();
    assert!(body.is_some());
    assert_eq!(body.unwrap(), "form1:value2");
    assert!(headers.headers_map.is_empty());
}

#[test]
fn test_collect_json_body() {
    let mut headers = Headers::new();
    let body_opt = None;
    let json_opt = Some(r#"{"key":"value"}"#);

    let body = collect_body(body_opt, json_opt, &mut headers).unwrap();
    assert!(body.is_some());
    assert_eq!(body.unwrap(), r#"{"key":"value"}"#);
    assert!(headers.get("Content-Type").is_some());
}