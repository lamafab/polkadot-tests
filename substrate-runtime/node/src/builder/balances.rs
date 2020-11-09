use super::{create_tx, Address, Balance, Call, UncheckedExtrinsic};
use crate::chain_spec::CryptoPair;
use crate::executor::ClientTemp;
use crate::Result;
use codec::Encode;
use pallet_balances::Call as BalancesCall;
use sp_core::crypto::Pair;

use std::fmt;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug)]
struct RawPrivateKey(Vec<u8>);

impl FromStr for RawPrivateKey {
    type Err = failure::Error;

    fn from_str(val: &str) -> Result<Self> {
        Ok(RawPrivateKey(
            hex::decode(val.replace("0x", ""))
                .map_err(|err| err.into())
                .and_then(|b| {
                    if b.len() == 32 {
                        Ok(b)
                    } else {
                        Err(failure::err_msg("Private key seed must be 32 bytes"))
                    }
                })?,
        ))
    }
}

impl From<RawPrivateKey> for CryptoPair {
    fn from(val: RawPrivateKey) -> Self {
        let mut seed = [0; 32];
        seed.copy_from_slice(&val.0);
        seed.into()
    }
}

#[derive(Debug)]
pub struct RawExtrinsic(Vec<u8>);

impl fmt::Display for RawExtrinsic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl From<Vec<u8>> for RawExtrinsic {
    fn from(val: Vec<u8>) -> Self {
        RawExtrinsic(val)
    }
}

impl From<UncheckedExtrinsic> for RawExtrinsic {
    fn from(val: UncheckedExtrinsic) -> Self {
        RawExtrinsic::from(val.encode())
    }
}

#[derive(Debug, StructOpt)]
pub struct PalletBalancesCmd {
    #[structopt(subcommand)]
    call: CallCmd,
}

#[derive(Debug, StructOpt)]
enum CallCmd {
    Transfer {
        #[structopt(short, long)]
        from: RawPrivateKey,
        #[structopt(short, long)]
        to: Address,
        #[structopt(short, long)]
        balance: Balance,
    },
}

impl PalletBalancesCmd {
    pub fn run(self) -> Result<RawExtrinsic> {
        match self.call {
            CallCmd::Transfer { from, to, balance } => ClientTemp::new()?
                .exec_context(|| {
                    create_tx(
                        from.into(),
                        Call::Balances(BalancesCall::transfer(to.into(), balance)),
                        0,
                    )
                    .map(|t| RawExtrinsic::from(t))
                    .map(Some)
                })
                .map(|extr| extr.unwrap()),
        }
    }
}
