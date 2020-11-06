use super::{AccountId, Address, Balance, Call, SignedExtra, UncheckedExtrinsic};
use crate::chain_spec::{get_account_id_from_seed, CryptoPair};
use crate::executor::ClientTemp;
use crate::Result;
use codec::{Decode, Encode};
use pallet_balances::Call as BalancesCall;
use sp_core::crypto::Pair;
use sp_core::sr25519;
use sp_runtime::generic::{Era, SignedPayload};
use sp_runtime::traits::SignedExtension;
use std::fmt;
use std::str::FromStr;
use structopt::StructOpt;

fn sign_tx(signer: CryptoPair, function: Call, nonce: u32) -> Result<UncheckedExtrinsic> {
    fn extra_err() -> failure::Error {
        failure::err_msg("Failed to retrieve additionally signed extra")
    }

    let check_spec_version = frame_system::CheckSpecVersion::new();
    let check_tx_version = frame_system::CheckTxVersion::new();
    let check_genesis = frame_system::CheckGenesis::new();
    let check_era = frame_system::CheckEra::from(Era::Immortal);
    let check_nonce = frame_system::CheckNonce::from(nonce);
    let check_weight = frame_system::CheckWeight::new();
    let payment = pallet_transaction_payment::ChargeTransactionPayment::from(0);

    #[rustfmt::skip]
    let additional_extra = (
        check_spec_version.additional_signed().map_err(|_| extra_err())?,
        check_tx_version.additional_signed().map_err(|_| extra_err())?,
        check_genesis.additional_signed().map_err(|_| extra_err())?,
        check_era.additional_signed().map_err(|_| extra_err())?,
        check_nonce.additional_signed().map_err(|_| extra_err())?,
        check_weight.additional_signed().map_err(|_| extra_err())?,
        payment.additional_signed().map_err(|_| extra_err())?,
    );

    let extra: SignedExtra = (
        check_spec_version,
        check_tx_version,
        check_genesis,
        check_era,
        check_nonce,
        check_weight,
        payment,
    );

    let payload = SignedPayload::from_raw(function, extra, additional_extra);

    let signature = payload.using_encoded(|payload| signer.sign(payload));

    let (function, extra, _) = payload.deconstruct();

    Ok(UncheckedExtrinsic::new_signed(
        function,
        signer.public().into(),
        signature.into(),
        extra,
    ))
}

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
                    sign_tx(
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
