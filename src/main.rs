mod executor;
mod request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let requests = request::load_requests_from_toml("requests.toml")?;

    executor::execute_request_chain(requests).await?;

    Ok(())
}
