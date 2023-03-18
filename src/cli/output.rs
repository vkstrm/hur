use crate::error::Error;
use crate::http::response::Response;

pub fn handle_output(
    response: Response,
    request: serde_json::Value,
    verbose: bool,
) -> Result<(), Error> {
    if verbose {
        let json_output = serde_json::json!({"request": request, "response":response});
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        match &response.body {
            Some(body) => {
                println!("{}", body)
            }
            None => {}
        }
    }
    Ok(())
}
