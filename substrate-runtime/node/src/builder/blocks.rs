use structopt::StructOpt;
use std::str::FromStr;

#[derive(Debug, StructOpt)]
pub struct BlockCmd {
    #[structopt(subcommand)]
    call: CallCmd,
}

#[derive(Debug, StructOpt)]
enum CallCmd {
    BuildBlock,
}
