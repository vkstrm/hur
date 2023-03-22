use std::io::Read;
use std::path::PathBuf;

use super::http::{headers::Headers, Method};
use super::logs;

use crate::error;
use crate::error::Error;
use crate::http::headers::Header;
use crate::http::request::Request;

use clap::Parser;

pub mod output;
use url::Url;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    disable_help_flag = true,
    arg_required_else_help = true
)]
struct Cli {
    #[arg(help = "The URL for the request", required = true)]
    pub url: String,
    #[arg(short, long, help = "Full request and response output in JSON")]
    pub verbose: bool,
    #[arg(
        short,
        long,
        help = "The HTTP method to use for the request. Default Get",
        value_enum,
        default_value_t = Method::Get
    )]
    pub method: Method,
    #[arg(long)]
    pub info: bool,
    #[arg(short = 'h', long, help = "Add header as 'key:value'", value_parser = Header::try_from)]
    pub header: Option<Vec<Header>>,
    #[arg(long, help = "Add headers as a JSON string or JSON file")]
    pub headers: Option<String>,
    #[arg(short, long, help = "Add request body")]
    pub body: Option<String>,
    #[arg(
        long,
        help = "Path to file to use as request body",
        conflicts_with = "body",
        conflicts_with = "json"
    )]
    pub body_file: Option<PathBuf>,
    #[arg(
        short,
        long,
        help = "Request body to send with Content-Type:application/json",
        conflicts_with = "body",
        conflicts_with = "body_file"
    )]
    pub json: Option<String>,
    #[arg(long, action = clap::ArgAction::Help)]
    help: Option<bool>,
    #[arg(long, help = "Don't use proxy environment variables")]
    pub no_proxy: bool,
    #[arg(short, long, help = "The read timeout in seconds for the request")]
    pub timeout: Option<u64>,
}

pub fn create_request(args: Vec<String>) -> Result<(Request, bool), Error> {
    let parser = Cli::parse_from(args);

    let parsed_url = parse_url(&parser.url)?;
    if parser.info {
        enable_logging()?;
    }

    let mut headers = headers(&parser)?;
    let mut request = match parse_body(&parser, &mut headers)? {
        Some(body) => Request::with_body(parsed_url, parser.method, headers, &body),
        None => Request::new(parsed_url, parser.method, headers),
    }?;

    if parser.no_proxy {
        request.disable_proxy();
    }

    if let Some(timeout) = parser.timeout {
        request.set_timeout(timeout);
    }

    Ok((request, parser.verbose))
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

fn parse_body(matches: &Cli, headers: &mut Headers) -> Result<Option<String>, Error> {
    if let Some(body) = &matches.body {
        return Ok(Some(body.to_string()));
    }
    if let Some(body) = &matches.json {
        match serde_json::from_str::<serde_json::Value>(body) {
            Ok(_) => {
                headers.add("Content-Type", "application/json");
                return Ok(Some(body.to_string()));
            }
            Err(why) => error!(&why.to_string()),
        };
    }
    if let Some(path) = &matches.body_file {
        let file_str = read_file(path)?;
        match path.extension() {
            Some(extension) if extension == "json" => {
                headers.add("Content-Type", "application/json")
            }
            _ => (),
        };
        return Ok(Some(file_str));
    }

    Ok(None)
}

fn headers(cli: &Cli) -> Result<Headers, Error> {
    let mut headers = match &cli.header {
        Some(headers) => Headers::from(headers),
        None => Headers::new(),
    };

    if let Some(h) = &cli.headers {
        let h = json_headers(h)?;
        headers.append(h)
    };

    Ok(headers)
}

fn json_headers(headers_json: &str) -> Result<Headers, Error> {
    let json_path = std::path::PathBuf::from(headers_json);
    let json_string = match json_path.extension() {
        Some(extension) if extension == "json" => read_file(&json_path)?,
        _ => headers_json.to_string(),
    };
    let map: std::collections::HashMap<String, String> = serde_json::from_str(&json_string)?;
    Ok(Headers::from(map))
}

fn read_file(path: &PathBuf) -> Result<String, Error> {
    let mut file = std::fs::File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

fn enable_logging() -> Result<(), log::SetLoggerError> {
    static LOGGER: logs::Logger = logs::Logger;
    log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info))
}

#[test]
fn test_json_headers() {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("headers.json");
    let input = vec![
        "hur",
        "http://localhost",
        "--headers",
        temp_file.to_str().unwrap(),
    ];
    let cli = Cli::parse_from(input);
    let mut file = std::fs::File::create(temp_file).unwrap();
    use std::io::Write;
    file.write(br#"{"key":"value"}"#).unwrap();

    let headers = headers(&cli).unwrap();
    assert_eq!(headers.get("Key").unwrap()[0], "value".to_string());
}

#[test]
fn test_collect_body() {
    let input = vec!["hur", "http://localhost", "--body", "form:value"];
    let cli = Cli::parse_from(input);

    let mut headers = Headers::new();
    let body = parse_body(&cli, &mut headers).unwrap();

    assert!(body.is_some());
    assert_eq!(body.unwrap(), "form:value");
}

#[test]
fn test_collect_json_body() {
    let input = vec!["hur", "http://localhost", "--json", r#"{"key":"value"}"#];
    let cli = Cli::parse_from(input);

    let mut headers = Headers::new();
    let body = parse_body(&cli, &mut headers).unwrap();

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
    let cli = Cli::parse_from(input);
    let mut file = std::fs::File::create(temp_file).unwrap();
    use std::io::Write;
    file.write(br#"{"key":"value"}"#).unwrap();

    let mut headers = Headers::new();
    let body = parse_body(&cli, &mut headers).unwrap();

    assert!(body.is_some());
    assert_eq!(body.unwrap(), r#"{"key":"value"}"#);
    assert!(headers.get("Content-Type").is_some());
}
