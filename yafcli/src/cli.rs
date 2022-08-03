use clap::Parser;

//

#[derive(Debug, Parser)]
#[clap(name = "YAFCLI")]
#[clap(about = "A command line interface for Yet Another F***ing Calculator (YAFC)")]
#[clap(author, version)]
pub struct CliArgs {
    /// Calculate directly instead of
    /// entering the yafcli
    #[clap(long, short, value_parser)]
    pub direct: Option<String>,

    /// Debug logging
    #[clap(long, value_parser)]
    pub debug: bool,

    /// Verbose output
    #[clap(long, short, value_parser)]
    pub verbose: bool,
}
