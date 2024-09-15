use clap::Command;

pub fn get_cli_matches() -> clap::ArgMatches {
    Command::new("send")
        .version("0.1.0")
        .author("John Doe")
        .about("Send HTTP requests based on a request chain defined in TOML")
        .allow_external_subcommands(true)
        .get_matches()
}
