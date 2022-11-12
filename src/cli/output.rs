use crate::error::Error;
use crate::http::response::Response;

pub struct Output {
    pub verbose: bool,
    pub no_body: bool,
}

pub fn handle_output(
    mut response: Response,
    request: serde_json::Value,
    output: Output,
) -> Result<(), Error> {
    if output.verbose {
        if output.no_body {
            response.body = None;
        }
        let json_output = serde_json::json!({"request": request, "response":response});
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        match &response.body {
            Some(body) => {
                if !output.no_body {
                    println!("{}", body)
                }
            }
            None => {}
        }
    }
    Ok(())
}
