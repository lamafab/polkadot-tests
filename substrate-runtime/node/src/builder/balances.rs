use super::{AccountId, Address, Call, SignedExtra, UncheckedExtrinsic};
use crate::chain_spec::get_account_id_from_seed;
use crate::Result;
use codec::Encode;
use sp_core::crypto::Pair;
use sp_core::sr25519;
use sp_runtime::generic::{Era, SignedPayload};
use sp_runtime::traits::SignedExtension;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct PalletBalancesCmd {
    #[structopt(subcommand)]
    call: CallCmd,
}

fn sign_tx(signer: sr25519::Pair, function: Call, nonce: u32) -> Result<UncheckedExtrinsic> {
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
        Ok(RawPrivateKey(hex::decode(val)?))
    }
}

#[derive(Debug, StructOpt)]
enum CallCmd {
    Transfer {
        from: RawPrivateKey,
        to: Address,
        balance: u128,
    },
}

impl PalletBalancesCmd {
    pub fn run(&self) -> Result<()> {
        match &self.call {
            CallCmd::Transfer { from, to, balance } => {
                //let _ = UncheckedExtrinsics::new_signed(BalancesCall::transer(), )
            }
        }

        Ok(())
    }
}
