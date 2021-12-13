use super::http;
use super::error::Error;

use clap::{ArgMatches, App, Arg};

pub struct Input {
    pub url: String,
    pub method: http::Method,
    pub headers: Option<http::headers::Headers>,
    pub body: Option<String>,
    pub json: bool
}

pub struct Output {
    pub verbose: bool,
    pub query_header: Option<String>
}

pub struct InOut {
    pub input: Input,
    pub output: Output,
}

pub fn parse_args(args: &Vec<String>) -> Result<InOut, Error> {
    let matches = use_clap(&args);

    let mut body: Option<String> = None;
    let mut json: bool = false;
    if let Some(body_str) = matches.value_of("body") {
        body = Some(body_str.to_string());
    }
    if let Some(body_str) = matches.value_of("json") {
        match serde_json::from_str::<serde_json::Value>(body_str) {
            Ok(_) => {
                body = Some(body_str.to_string());
            },
            Err(why) => return Err(Error::from(why)),
        };
        json = true;
    }

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

    let input = Input{
        url: matches.value_of("url").unwrap().to_string(),
        method: get_method(matches.value_of("method").unwrap()),
        headers,
        body,
        json
    };

    let output = Output {
        verbose: matches.is_present("verbose"),
        query_header: match matches.value_of("query-header") {
            Some(q) => Some(String::from(q.trim())),
            None => None,
        }
    };

    Ok(InOut{
        input,
        output
    })
}

fn get_method(method: &str) -> http::Method {
    match method.to_lowercase().as_str() {
        "get" => http::Method::GET,
        "post" => http::Method::POST,
        "put" => http::Method::PUT,
        "delete" => http::Method::DELETE,
        "patch" => http::Method::PATCH,
        "connect" => http::Method::CONNECT,
        "options" => http::Method::OPTIONS,
        "trace" => http::Method::TRACE,
        "head" => http::Method::HEAD, 
        _ => http::Method::GET,
    }
}

fn use_clap(args: &Vec<String>) -> ArgMatches {
    return
    App::new("hur")
        .version("v0.1.0")
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .about("Print request and response in full")
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
                .conflicts_with("json")
                .takes_value(true)
        )
        .arg(
            Arg::new("json")
                .about("Send body with Content-Type:application/json")
                .long("json")
                .conflicts_with("body")
                .takes_value(true)
        )
        .arg(
            Arg::new("query-header")
                .about("Query for a specific header from the response")
                .long("query-header")
                .takes_value(true)
        )
        .get_matches_from(args);
}