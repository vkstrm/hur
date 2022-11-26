# HUR

Command-line utility for making HTTP requests.

Note: This program is not better than Curl. It will contain bugs and is unlikely to fully conform to the HTTP specification. I am however having fun writing it and find it to be useful in some situations.


## Usage

GET is the default method.
```sh
hur https://petstore.com/animals -h "header:value"
```
To use another method add `--method`
```sh
hur https://petstore.com/animals --method POST --body '{"name":"Luffy"}'
```
Using `--verbose` mode will print, in JSON, the request and response objects.
```json
{
    "request": {
        "method": "GET",
        "headers": {},
        "path": "/animals",
        "etc" "..."
    },
    "response": {
        "statusCode": 200,
        "body": "{}",
        "etc": "..."
    }
}
```
Use `--help` for more information.

## Proxy

Proxy support with HTTP_PROXY, HTTPS_PROXY and NO_PROXY environment variables.
Disable proxy for a request using `--no-proxy`

## Dependencies

On Ubuntu, in addition to Rust, you will need `sudo apt install build-essential libssl-dev pkg-config`.
Or Cargo will tell you what you need most likely.