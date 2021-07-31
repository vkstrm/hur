use super::error::Error;
use super::request;

use url::Url;
use clap::{ArgMatches, App, Arg};

pub struct Input {
    pub url: url::Url,
    pub method: request::Method,
}

pub fn parse_args(args: &Vec<String>) -> Result<Input, Error> {
    let matches = use_clap(&args);
    let parsed_url = match Url::parse(matches.value_of("url").unwrap()) {
        Ok(url) => url,
        Err(why) => return Err(Error::new(&why.to_string()))
    };

    if !parsed_url.has_host() {
        return Err(Error::new("no host in input"))
    }

    Ok(
        Input{
            url: parsed_url,
            method: match matches.value_of("method").unwrap() {
                "get" | "GET" => request::Method::GET,
                _ => return Err(Error::new("unsupported method")),
            }
        }
    )
}

fn use_clap(args: &Vec<String>) -> ArgMatches {
    return
    App::new("rttp")
        .version("v0.1.0")
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
                .possible_values(&["get"])
        )
        .get_matches_from(args);
}