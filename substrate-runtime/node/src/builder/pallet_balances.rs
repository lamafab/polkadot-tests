use crate::Result;
use crate::chain_spec::get_account_id_from_seed;
use sp_core::sr25519;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct PalletBalancesCmd {
    #[structopt(subcommand)]
    call: Call,
}

#[derive(Debug, StructOpt)]
pub enum Call {
    Transfer {
        from: String,
        to: String,
        balance: u128,
    },
}

impl PalletBalancesCmd {
    pub fn run(&self) -> Result<()> {
        match &self.call {
            Call::Transfer { from, to, balance } => {

            }
        }

        Ok(())
    }
}
