//! Export the API spec without starting the server or connecting to PostgreSQL.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spec = api_example::openapi().to_pretty_json()?;
    std::fs::write("openapi.json", format!("{spec}\n"))?;
    Ok(())
}
