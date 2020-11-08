use super::builder;
use sc_cli::RunCmd;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(subcommand)]
    pub subcommand: Subcommand,

    #[structopt(flatten)]
    pub run: RunCmd,
}

#[derive(Debug, StructOpt)]
pub enum Subcommand {
    PalletBalances(builder::PalletBalancesCmd),
}
