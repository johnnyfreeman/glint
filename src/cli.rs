use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// does testing things
    Collection {
        #[command(subcommand)]
        command: CollectionCommands,
    },
    /// does testing things
    Request {
        #[command(subcommand)]
        command: RequestCommands,
    },
}

#[derive(Subcommand)]
pub enum CollectionCommands {
    /// Does testing things
    Run {
        /// The collection file to use
        collection: String,

        /// Additional output options
        #[command(flatten)]
        output_options: OutputOptions,
    },
}

#[derive(Args, Clone)]
pub struct OutputOptions {
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

    /// Disables pre-output masking (enabled by default)
    #[arg(short = 'm', long, default_value_t = false)]
    pub disable_masking: bool,
}

#[derive(Subcommand)]
pub enum RequestCommands {
    /// does testing things
    Run {
        /// The collection file to use
        collection: String,

        /// The specific request to execute within the collection
        request: String,

        /// Additional output options
        #[command(flatten)]
        output_options: OutputOptions,
    },
}
