# HUR

Command-line utility for making HTTP requests.

Note: This program is not better than Curl.
It will contain bugs and is unlikely to fully conform to the HTTP specification.
I am however having fun writing it and find it to be useful in some situations.

## Installation 
```nu
cargo install hur
```

## Usage

GET is the default method.
```nu
hur req https://petstore.com/animals -h "header:value"
```
To use another method add `--method`
```nu
hur req https://petstore.com/animals --method POST --body '{"name":"Luffy"}'
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
Use `help` for more information.

## Config

Hur has a configuration file. Currently it only support setting redirect mode.

It can be created by running `hur config create`.

## Proxy

Proxy support with HTTP_PROXY, HTTPS_PROXY and NO_PROXY environment variables.
Disable proxy for a request using `--no-proxy`
