use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Command {
    run { cfg_path: Option<String> },
}
#[derive(Debug, StructOpt)]
#[structopt(
    name = "luminar",
    about = "Manage computing resources for multiple users on single machine."
)]
pub struct LuminarArgs {
    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    pub cmd: Command,
}
