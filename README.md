# HUR

CLI tool for making HTTP requests. 
Written to learn more about Rust and HTTP.

## Usage

- `-m` to select method. GET is default.
- `-h, --header` and then `"header-name:header-value"` for adding headers.
- `--body <body>` for sending a request body.
- `--json` send a body and "Content-Type:application/json" header. 
- `--help` for more information

## Proxy

Proxy support with HTTP_PROXY, HTTPS_PROXY and NO_PROXY environment variables. Disable proxy for request using `--no-proxy`

## Dependencies

On Linux, in addition to Rust, you will need `sudo apt install build-essential libssl-dev pkg-config`.