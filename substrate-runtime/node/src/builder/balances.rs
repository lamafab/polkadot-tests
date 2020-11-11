use super::create_tx;
use crate::chain_spec::CryptoPair;
use crate::executor::ClientTemp;
use crate::primitives::runtime::{Address, Balance, Call, UncheckedExtrinsic};
use crate::primitives::RawExtrinsic;
use crate::Result;
use codec::Encode;
use pallet_balances::Call as BalancesCall;
use sp_core::crypto::Pair;

use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, StructOpt)]
pub struct PalletBalancesCmd {
    #[structopt(subcommand)]
    call: CallCmd,
}

impl PalletBalancesCmd {
    pub fn transer(details: TransferDetails) -> Self {
        PalletBalancesCmd {
            call: CallCmd::Transfer { details },
        }
    }
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct TransferDetails {
    #[structopt(short, long)]
    from: RawPrivateKey,
    #[structopt(short, long)]
    to: Address,
    #[structopt(short, long)]
    balance: Balance,
}

#[derive(Debug, StructOpt)]
enum CallCmd {
    Transfer {
        #[structopt(flatten)]
        details: TransferDetails,
    },
}

impl PalletBalancesCmd {
    pub fn run(self) -> Result<RawExtrinsic> {
        match self.call {
            CallCmd::Transfer { details } => ClientTemp::new()?
                .exec_context(|| {
                    create_tx(
                        details.from.into(),
                        Call::Balances(BalancesCall::transfer(details.to.into(), details.balance)),
                        0,
                    )
                    .map(|t| RawExtrinsic::from(t))
                    .map(Some)
                })
                .map(|extr| extr.unwrap()),
        }
    }
}
