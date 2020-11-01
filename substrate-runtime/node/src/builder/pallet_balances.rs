use crate::Result;
use crate::chain_spec::get_account_id_from_seed;
use sp_core::sr25519;
use structopt::StructOpt;
use std::str::FromStr;

#[derive(Debug, StructOpt)]
pub struct PalletBalancesCmd {
    call: Call,
}

#[derive(Debug, StructOpt)]
pub enum Call {
    Transfer,
}

impl FromStr for Call {
    type Err = failure::Error;

    fn from_str(val: &str) -> Result<Self> {
        match val {
            "transfer" => Ok(Call::Transfer),
            _ => Err(failure::err_msg(format!("Call to function '{}' is not supported", val))),
        }
    }
}

impl PalletBalancesCmd {
    pub fn run(&self) -> Result<()> {
        match self {
            Transfer => {

            }
        }

        Ok(())
    }
}
