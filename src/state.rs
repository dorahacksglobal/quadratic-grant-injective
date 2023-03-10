use cosmwasm_std::Uint128;
use serde::{Deserialize, Serialize};
use sylvia::schemars;

#[derive(Serialize, Deserialize, Clone, PartialEq, schemars::JsonSchema, Debug, Default)]
pub enum RoundStatus {
    #[default] Voting,
    Finished,
    Withdrawn,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, schemars::JsonSchema, Debug, Default)]
pub enum ProjectStatus {
    #[default] OK,
    Banned,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, schemars::JsonSchema, Debug, Default)]
pub struct Round {
    pub id: u64,
    pub tax_adjustment_multiplier: u64,
    pub donation_denom: String,
    pub voting_unit: Uint128,
    pub status: RoundStatus,
    pub fund: Uint128,
    pub project_number: u64,
    pub total_area: u128,
    pub total_amounts: u128,
    pub pubkey: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, schemars::JsonSchema, Debug, Default)]
pub struct Project {
    pub id: u64,
    pub owner: String,
    pub area: u128,
    pub status: ProjectStatus,
    pub votes: u128,
    pub contribution: u128,
}
