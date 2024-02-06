use cosmwasm_std::{Addr, StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{sender} is not a contract admin")]
    Unauthorized { sender: Addr },

    #[error("Payment error: {0}")]
    Payment(#[from] PaymentError),

    #[error("{address} is already an admin")]
    NoDupAddress { address: Addr },

    #[error("{round_id} is not in voting status")]
    RoundNotInVoting { round_id: u64 },

    #[error("{round_id} is not ended")]
    RoundNotEnded { round_id: u64 },

    #[error("{round_id} does not exist")]
    RoundNotExist { round_id: u64 },

    #[error("expected {expected} but got {actual}")]
    InvalidAmount { expected: u128, actual: u128 },

    #[error("expected {expected} but got {actual}")]
    LengthNotMatch { expected: u128, actual: u128 },

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid signature length")]
    InvalidSignatureLength,

    #[error("Invalid signature timestamp")]
    InvalidSignatureTimestamp,

    #[error("Pubkey not set")]
    PubkeyNotSet,

    #[error("Invalid pubkey length")]
    InvalidPubkeyLength,

    #[error("No admins set")]
    NoAdmins,

    #[error("Invalid denom: {denom}")]
    InvalidDenom { denom: String },

    #[error("Invalid voting unit")]
    VotingUnitZero
}
