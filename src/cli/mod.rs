use std::path::PathBuf;

use super::http::Method;
use super::logs;

use crate::error;
use crate::error::Error;
use crate::http::headers::Header;
use crate::io::read_file;

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
    pub headers_json: Option<String>,
    #[arg(short, long, help = "Add request body")]
    pub body: Option<String>,
    #[arg(
        long,
        help = "Path to file to use as request body",
        conflicts_with = "body",
        conflicts_with = "body_json"
    )]
    pub body_file: Option<PathBuf>,
    #[arg(
        long,
        help = "Request body to send with Content-Type:application/json",
        conflicts_with = "body",
        conflicts_with = "body_file"
    )]
    pub body_json: Option<String>,
    #[arg(long, action = clap::ArgAction::Help)]
    help: Option<bool>,
    #[arg(long, help = "Don't use proxy environment variables")]
    pub no_proxy: bool,
    #[arg(short, long, help = "The read timeout in seconds for the request")]
    pub timeout: Option<u64>,
}

pub struct Inputs {
    pub url: Url,
    pub method: Method,
    pub header: Option<Vec<Header>>,
    pub headers_json: Option<String>,
    pub body: Option<InputBody>,
    pub timeout: Option<u64>,
    pub verbose: bool,
    pub proxy: bool,
}

pub fn parse_input(args: Vec<String>) -> Result<Inputs, Error> {
    let parser = Cli::parse_from(args);
    if parser.info {
        enable_logging()?;
    }

    let parsed_url = parse_url(&parser.url)?;
    let body = parse_body(parser.body, parser.body_json, parser.body_file)?;
    let inputs = Inputs {
        url: parsed_url,
        method: parser.method,
        header: parser.header,
        headers_json: parser.headers_json,
        body,
        timeout: parser.timeout,
        verbose: parser.verbose,
        proxy: parser.no_proxy,
    };

    Ok(inputs)
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

pub struct InputBody {
    pub content: String,
    pub content_type: Option<String>,
}

fn parse_body(
    input_body: Option<String>,
    input_json: Option<String>,
    input_body_file: Option<PathBuf>,
) -> Result<Option<InputBody>, Error> {
    if let Some(body) = input_body {
        return Ok(Some(InputBody {
            content: body.to_string(),
            content_type: None,
        }));
    }
    if let Some(body) = input_json {
        match serde_json::from_str::<serde_json::Value>(&body) {
            Ok(_) => {
                return Ok(Some(InputBody {
                    content: body.to_string(),
                    content_type: Some("application/json".to_string()),
                }));
            }
            Err(why) => error!(&why.to_string()),
        };
    }
    if let Some(path) = input_body_file {
        let file_str = read_file(&path)?;
        let content_type = match path.extension() {
            Some(extension) if extension == "json" => Some("application/json".to_string()),
            _ => None,
        };
        return Ok(Some(InputBody {
            content: file_str,
            content_type,
        }));
    }

    Ok(None)
}

fn enable_logging() -> Result<(), log::SetLoggerError> {
    static LOGGER: logs::Logger = logs::Logger;
    log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info))
}

#[test]
fn test_parse_body() {
    let input = vec!["hur", "http://localhost", "--body", "form:value"];
    let cli = Cli::parse_from(input);

    let body = parse_body(cli.body, cli.body_json, cli.body_file).unwrap();

    assert!(body.is_some());
    assert_eq!(body.unwrap().content, "form:value");
}

#[test]
fn test_parse_json_body() {
    let input = vec!["hur", "http://localhost", "--json", r#"{"key":"value"}"#];
    let cli = Cli::parse_from(input);

    let body = parse_body(cli.body, cli.body_json, cli.body_file)
        .unwrap()
        .unwrap();

    assert_eq!(body.content, r#"{"key":"value"}"#);
    assert_eq!(body.content_type.unwrap(), "application/json");
}

#[test]
fn test_parse_body_file() {
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

    let body = parse_body(cli.body, cli.body_json, cli.body_file)
        .unwrap()
        .unwrap();

    assert_eq!(body.content, r#"{"key":"value"}"#);
    assert_eq!(body.content_type.unwrap(), "application/json");
}
