use crate::error::Error;
use crate::http::request::Request;
use crate::logs::enable_debug;
use crate::modes::RedirectMode;
use crate::requester::Requester;
use crate::requester::connector::{Connector, ProxyConnector, RegularConnector};

use clap::Parser;
use command::{Cli, Commands, ConfigCommands, ReqArgs};
use config::load_config;
use output::handle_output;
use parsing::{parse_body, parse_headers, parse_url};

mod command;
mod config;
mod output;
mod parsing;

struct DefaultModes {
    redirect_mode: RedirectMode,
}

const DEFAULT_MODES: DefaultModes = DefaultModes {
    redirect_mode: RedirectMode::NoFollow,
};

pub fn handle_args(args: Vec<String>) -> Result<(), Error> {
    let parser = Cli::parse_from(args);
    if parser.debug {
        enable_debug()?;
    }

    match parser.command {
        Some(Commands::Req(req_args)) => handle_req(req_args),
        Some(Commands::Config { command }) => handle_config(command),
        None => unreachable!(),
    }
}

fn handle_req(req: ReqArgs) -> Result<(), Error> {
    let config = load_config()?;
    let parsed_url = parse_url(&req.url)?;
    let body = parse_body(req.body, req.body_json, req.body_file)?;

    // gör snyggare?
    let redirect_mode = if let Some(mode) = req.redirect_mode {
        mode
    } else if let Some(conf) = config {
        conf.redirect_mode.unwrap_or(DEFAULT_MODES.redirect_mode)
    } else {
        DEFAULT_MODES.redirect_mode
    };

    let mut headers = parse_headers(req.header, req.headers_json)?;
    if let Some(input_body) = &body
        && let Some(content_type) = &input_body.content_type
    {
        headers.add("Content-Type", content_type);
    }

    let request = match body {
        Some(body) => Request::with_body(
            parsed_url,
            req.method,
            headers,
            body.content.as_str(),
            req.timeout,
        )?,
        None => Request::new(parsed_url, req.method, headers, req.timeout)?,
    };

    // "no_proxy" is actually proxy??
    let connector: Box<dyn Connector> = if req.no_proxy {
        Box::new(ProxyConnector::new(request.timeout))
    } else {
        Box::new(RegularConnector::new(request.timeout))
    };
    let requester = Requester::new(connector, redirect_mode);
    let request_output = serde_json::to_value(&request)?;
    let response = requester.do_request(request)?;
    handle_output(response, request_output, req.verbose)
}

fn handle_config(command: Option<ConfigCommands>) -> Result<(), Error> {
    match command {
        Some(ConfigCommands::Create) => config::create_default_config(),
        None => unreachable!(),
    }
}

#[test]
fn test_parse_body() {
    let input = vec!["hur", "req", "http://localhost", "--body", "form:value"];
    let cli = Cli::parse_from(input);

    let args = if let Commands::Req(req_args) = cli.command.unwrap() {
        req_args
    } else {
        panic!()
    };
    let body = parse_body(args.body, args.body_json, args.body_file).unwrap();

    assert!(body.is_some());
    assert_eq!(body.unwrap().content, "form:value");
}

#[test]
fn test_parse_json_body() {
    let input = vec![
        "hur",
        "req",
        "http://localhost",
        "--body-json",
        r#"{"key":"value"}"#,
    ];
    let cli = Cli::parse_from(input);
    let args = if let Commands::Req(req_args) = cli.command.unwrap() {
        req_args
    } else {
        panic!()
    };

    let body = parse_body(args.body, args.body_json, args.body_file)
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
        "req",
        "http://localhost",
        "--body-file",
        temp_file.to_str().unwrap(),
    ];
    let cli = Cli::parse_from(input);
    let args = if let Commands::Req(req_args) = cli.command.unwrap() {
        req_args
    } else {
        panic!()
    };
    let mut file = std::fs::File::create(temp_file).unwrap();
    use std::io::Write;
    file.write_all(br#"{"key":"value"}"#).unwrap();

    let body = parse_body(args.body, args.body_json, args.body_file)
        .unwrap()
        .unwrap();

    assert_eq!(body.content, r#"{"key":"value"}"#);
    assert_eq!(body.content_type.unwrap(), "application/json");
}
