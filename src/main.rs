mod executor;
mod request;
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    request: String,
    // #[arg(short, long, value_name = "FILE")]
    // env: Option<String>,

    // #[arg(short, long, action = clap::ArgAction::Count)]
    // debug: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let requests = request::load_requests_from_toml(cli.request.as_str())?;

    executor::execute_request_chain(requests).await?;

    Ok(())
}
