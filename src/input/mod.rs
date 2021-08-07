use super::error::Error;
use super::http;

use url::Url;
use clap::{ArgMatches, App, Arg};

pub struct Input {
    pub url: url::Url,
    pub method: http::Method,
    pub headers: Option<http::headers::Headers>,
    pub body: Option<String>,
    pub json: bool,
    pub verbose: bool,
    pub raw: bool,
}

pub fn parse_args(args: &Vec<String>) -> Result<Input, Error> {
    let matches = use_clap(&args);

    // Validate URL
    let parsed_url = match Url::parse(matches.value_of("url").unwrap()) {
        Ok(url) => url,
        Err(why) => return Err(Error::new(&why.to_string()))
    };

    if !parsed_url.has_host() {
        return Err(Error::new("no host in input"))
    }

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

    Ok(
        Input{
            url: parsed_url,
            method: get_method(matches.value_of("method").unwrap())?,
            headers,
            body,
            json,
            verbose: matches.is_present("verbose"),
            raw: matches.is_present("raw")
        }
    )
}

fn get_method(method: &str) -> Result<http::Method, Error> {
    match method.to_lowercase().as_str() {
        "get" => Ok(http::Method::GET),
        "post" => Ok(http::Method::POST),
        "put" => Ok(http::Method::PUT),
        _ => Err(Error::new("unsupported method")),
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
                .takes_value(true)
                .short('h')
                .long("header")
                .multiple_occurrences(true)
        )
        .arg(
            Arg::new("body")
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