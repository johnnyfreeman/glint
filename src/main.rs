mod executor;
mod options;
mod request;
mod resolvers;
use clap::Parser;
use executor::Executor;
use options::Options;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::parse();

    let requests = request::load_requests_from_toml(options.collection.as_str())?;

    Executor::new(requests, options).execute().await?;

    Ok(())
}
