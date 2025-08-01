use std::path::PathBuf;

use crate::http::headers::Header;
use crate::{http::Method, modes::RedirectMode};
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    disable_help_flag = true,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[arg(long, help = "Enable debug logging", global = true)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Manage the hur configuration")]
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },
    #[command(about = "Make HTTP requests")]
    Req(ReqArgs),
}

#[derive(Args)]
#[command()]
pub struct ReqArgs {
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
    #[arg(short, long, value_enum, help = "To follow or not follow redirects")]
    pub redirect_mode: Option<RedirectMode>,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    #[command(about = "Create a default config file at $HOME/.config/hur/config.toml")]
    Create,
}
