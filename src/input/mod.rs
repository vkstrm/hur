use super::http;

use clap::{ArgMatches, App, Arg};

pub struct Input {
    pub url: String,
    pub method: http::Method,
    pub headers: Option<http::headers::Headers>,
    pub body: Option<String>,
    pub json: bool,
    pub verbose: bool,
    pub raw: bool,
}

pub fn parse_args(args: &Vec<String>) -> Input {
    let matches = use_clap(&args);

    // Collect headers
    let headers = match matches.values_of("header") {
        Some(values) => {
            let mut headers = http::headers::Headers::new();
            for val in values {
                let splits: Vec<&str> = val.splitn(2, ':').collect();
                headers.add(splits[0], splits[1]);
            }
            Some(headers)
        },
        None => None
    };

    // Collect body and possible content type
    let mut body: Option<String> = None;
    let mut json: bool = false;
    if let Some(body_str) = matches.value_of("body") {
        body = Some(body_str.to_string());
    }
    if let Some(body_str) = matches.value_of("json") {
        body = Some(body_str.to_string());
        json = true;
    }

    Input{
        url: matches.value_of("url").unwrap().to_string(),
        method: get_method(matches.value_of("method").unwrap()),
        headers,
        body,
        json,
        verbose: matches.is_present("verbose"),
        raw: matches.is_present("raw")
    }
}

fn get_method(method: &str) -> http::Method {
    match method.to_lowercase().as_str() {
        "get" => http::Method::GET,
        "post" => http::Method::POST,
        "put" => http::Method::PUT,
        _ => http::Method::GET,
    }
}

fn use_clap(args: &Vec<String>) -> ArgMatches {
    return
    App::new("rttp")
        .version("v0.1.0")
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .about("Print request and response in full")
        )
        .arg(
            Arg::new("raw")
                .long("raw")
                .about("Print full response and request HTTP")
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
                .possible_values(&["get","post"])
        )
        .arg(
            Arg::new("header")
                .about("Header for request")
                .takes_value(true)
                .short('h')
                .long("header")
                .multiple_occurrences(true)
        )
        .arg(
            Arg::new("body")
                .about("Body for request")
                .long("body")
                .takes_value(true)
        )
        .arg(
            Arg::new("json")
                .about("Send body with application/json Content-Type")
                .long("json")
                .conflicts_with("body")
                .takes_value(true)
        )
        .get_matches_from(args);
}