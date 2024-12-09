use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Options {
    /// The collection file to use
    pub collection: String,

    /// The specific request to execute within the collection (optional)
    pub request: Option<String>,

    /// Displays the HTTP headers in the output (disabled by default)
    #[arg(short = 'h', long, default_value_t = false)]
    pub show_headers: bool,

    /// Suppresses the HTTP response status (enabled by default)
    #[arg(short = 's', long, default_value_t = false)]
    pub hide_status: bool,

    /// Suppresses the HTTP response body (enabled by default)
    #[arg(short = 'b', long, default_value_t = false)]
    pub hide_body: bool,

    /// Disables pretty-printing for the HTTP response (enabled by default)
    #[arg(short = 'r', long, default_value_t = false)]
    pub raw_output: bool,
}
