use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Options {
    pub collection: String,

    pub request: Option<String>,

    #[arg(long, default_value_t = false)]
    pub show_headers: bool,
    // #[arg(short, long, value_name = "FILE")]
    // pub env: Option<String>,

    // #[arg(short, long, action = clap::ArgAction::Count)]
    // pub debug: u8,
}
