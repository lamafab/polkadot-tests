use super::Result;
use crate::chain_spec::CryptoPair;
use crate::primitives::runtime::{Call, SignedExtra, UncheckedExtrinsic};
use codec::Encode;
use sp_runtime::generic::{Era, SignedPayload};
use sp_runtime::traits::SignedExtension;

pub mod balances;
pub mod blocks;

pub use balances::PalletBalancesCmd;
pub use blocks::BlockCmd;

fn create_tx(signer: CryptoPair, function: Call, nonce: u32) -> Result<UncheckedExtrinsic> {
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

/*
Summary:

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;
/// The address format for describing accounts.
pub type Address = AccountId;
/// An index to a block.
pub type BlockNumber = u32;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>
);
*/
