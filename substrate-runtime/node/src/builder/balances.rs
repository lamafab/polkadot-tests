use super::{create_tx, Builder, ModuleName};
use crate::builder::genesis::get_account_id_from_seed;
use crate::executor::ClientInMem;
use crate::primitives::runtime::{Balance, BlockId, RuntimeCall};
use crate::primitives::{ExtrinsicSigner, RawExtrinsic, TxtAccountSeed, TxtChainSpec};
use crate::Result;
use pallet_balances::Call as BalancesCall;
use sp_core::crypto::Pair;
use std::convert::TryInto;
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

module!(
    #[serde(rename = "pallet_balances")]
    struct PalletBalancesCmd;

    enum CallCmd {
        #[serde(rename = "transfer")]
        Transfer {
            #[structopt(short, long)]
            genesis: Option<TxtChainSpec>,
            #[structopt(short, long)]
            from: TxtAccountSeed,
            #[structopt(short, long)]
            to: TxtAccountSeed,
            #[structopt(short, long)]
            balance: u64,
        },
    }

    impl PalletBalancesCmd {
        fn run(self) -> Result<RawExtrinsic> {
            match self.call {
                CallCmd::Transfer {
                    genesis: _,
                    from,
                    to,
                    balance,
                } => ClientInMem::new()?
                    .exec_context(&BlockId::Number(0), || {
                        create_tx::<ExtrinsicSigner>(
                            from.try_into()?,
                            RuntimeCall::Balances(BalancesCall::transfer(
                                get_account_id_from_seed::<<ExtrinsicSigner as Pair>::Public>(
                                    to.as_str(),
                                )
                                .into(),
                                balance as Balance,
                            )),
                            0,
                        )
                        .map(|t| RawExtrinsic::from(t))
                        .map(Some)
                    })
                    .map(|extr| extr.unwrap()),
            }
        }
    }
);
