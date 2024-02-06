//! Quadratic Grant Contract
//! Author: noodles@dorahacks.com
//! Version: 0.1.0
//! License: Apache-2.0

use cosmwasm_std::{
    coins, Addr, BankMsg, DenomUnit, Deps, DepsMut, Empty, Env, Event, MessageInfo, Order,
    Response, StdError, StdResult, Uint128,
};
use cw_storage_plus::{Item, Map};
use schemars;
use sylvia::contract;

use crate::{
    error::ContractError,
    helper::{math, signature},
    responses::AdminListResp,
    state::{Project, ProjectStatus, Round, RoundStatus},
};

pub struct QGContract<'a> {
    pub(crate) owner: Item<'a, Addr>,
    pub(crate) admins: Map<'a, &'a Addr, Empty>,
    pub(crate) rounds: Map<'a, &'a str, Round>,
    pub(crate) current_round: Item<'a, u64>,
    pub(crate) projects: Map<'a, (&'a str, &'a str), Project>, // (round_id, project_id)
    pub(crate) votes: Map<'a, (&'a str, &'a str, &'a Addr), u128>, // (round_id, project_id, voter)
}

#[contract]
#[error(ContractError)]
impl QGContract<'_> {
    pub const fn new() -> Self {
        Self {
            owner: Item::new("owner"),
            admins: Map::new("admins"),
            rounds: Map::new("rounds"),
            current_round: Item::new("current_round"),
            projects: Map::new("projects"),
            votes: Map::new("votes"),
        }
    }

    fn check_owner_permission(
        &self,
        deps: &DepsMut,
        info: &MessageInfo,
    ) -> Result<(), ContractError> {
        if self.owner.load(deps.storage)? != info.sender {
            return Err(ContractError::Unauthorized {
                sender: info.sender.clone(),
            });
        }
        Ok(())
    }

    fn check_admin_permission(
        &self,
        deps: &DepsMut,
        info: &MessageInfo,
    ) -> Result<(), ContractError> {
        if !self.admins.has(deps.storage, &info.sender) {
            return Err(ContractError::Unauthorized {
                sender: info.sender.clone(),
            });
        }
        Ok(())
    }

    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        admins: Vec<String>,
    ) -> Result<Response, ContractError> {
        let (deps, _, msg) = ctx;

        if admins.is_empty() {
            return Err(ContractError::NoAdmins {});
        }

        self.owner.save(deps.storage, &msg.sender)?;
        for admin in admins {
            let admin = deps.api.addr_validate(&admin)?;
            self.admins.save(deps.storage, &admin, &Empty {})?;
        }
        self.current_round.save(deps.storage, &0)?;
        Ok(Response::new())
    }

    // ============= Query ============= //
    #[msg(query)]
    pub fn admin_list(&self, ctx: (Deps, Env)) -> StdResult<AdminListResp> {
        let (deps, _) = ctx;

        let admins: Result<_, _> = self
            .admins
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|addr| addr.map(String::from))
            .collect();

        Ok(AdminListResp { admins: admins? })
    }

    #[msg(query)]
    pub fn round(&self, ctx: (Deps, Env), round_id: u64) -> StdResult<Round> {
        let (deps, _) = ctx;

        let round = self.rounds.may_load(deps.storage, &round_id.to_string())?;

        match round {
            Some(round) => Ok(round),
            None => Err(StdError::generic_err("Round not found")),
        }
    }

    #[msg(query)]
    pub fn project(&self, ctx: (Deps, Env), round_id: u64, project_id: u64) -> StdResult<Project> {
        let (deps, _) = ctx;

        let project = self.projects.may_load(
            deps.storage,
            (&round_id.to_string(), &project_id.to_string()),
        )?;

        match project {
            Some(project) => Ok(project),
            None => Err(StdError::generic_err("Project not found")),
        }
    }

    // ============= Execute ============= //
    #[msg(exec)]
    pub fn add_admin(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        admin: String,
    ) -> Result<Response, ContractError> {
        let (deps, _, info) = ctx;
        self.check_owner_permission(&deps, &info)?;

        let admin = deps.api.addr_validate(&admin)?;
        if self.admins.has(deps.storage, &admin) {
            return Err(ContractError::NoDupAddress { address: admin });
        }

        self.admins.save(deps.storage, &admin, &Empty {})?;

        let resp = Response::new()
            .add_attribute("action", "add_member")
            .add_event(Event::new("admin_added").add_attribute("addr", admin));
        Ok(resp)
    }

    #[msg(exec)]
    pub fn del_admin(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        admin: String,
    ) -> Result<Response, ContractError> {
        let (deps, _, info) = ctx;
        self.check_owner_permission(&deps, &info)?;

        let admin = deps.api.addr_validate(&admin)?;
        if !self.admins.has(deps.storage, &admin) {
            return Err(ContractError::NoAdmins);
        }
        
        self.admins.remove(deps.storage, &admin);

        let resp = Response::new()
            .add_attribute("action", "del_admin")
            .add_event(Event::new("admin_removed").add_attribute("addr", admin));
        Ok(resp)
    }

    #[msg(exec)]
    pub fn start_round(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        tax_adjustment_multiplier: u64,
        donation_denom: String,
        voting_unit: Uint128,
        fund: Uint128,
        pubkey: Vec<u8>,
    ) -> Result<Response, ContractError> {
        let (deps, _, info) = ctx;
        self.check_admin_permission(&deps, &info)?;

        let supply = deps
            .querier
            .query_supply(&donation_denom)
            .unwrap_or_default();
        if supply.amount.is_zero() {
            return Err(ContractError::InvalidDenom {
                denom: donation_denom,
            });
        }

        if voting_unit.u128() == 0 {
            return Err(ContractError::VotingUnitZero {});
        }

        if pubkey.len() != 65 && pubkey.len() != 0 {
            return Err(ContractError::InvalidPubkeyLength {});
        }

        let current_round = self.current_round.load(deps.storage)?;
        let round_id = current_round + 1;
        self.current_round.save(deps.storage, &round_id)?;

        let round = Round {
            id: round_id,
            tax_adjustment_multiplier,
            donation_denom,
            voting_unit,
            status: RoundStatus::Voting,
            fund,
            project_number: 0,
            total_area: 0,
            total_amounts: 0,
            pubkey,
        };

        self.rounds
            .save(deps.storage, &round_id.to_string(), &round)?;

        let resp = Response::new()
            .add_attribute("action", "start_round")
            .add_event(Event::new("start_round").add_attribute("id", round_id.to_string()));
        Ok(resp)
    }

    #[msg(exec)]
    pub fn batch_upload_project(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        round_id: u64,
        owner_addresses: Vec<String>,
    ) -> Result<Response, ContractError> {
        let (deps, _, info) = ctx;
        self.check_admin_permission(&deps, &info)?;

        let mut round = self.rounds.load(deps.storage, &round_id.to_string())?;

        if round.status != RoundStatus::Voting {
            return Err(ContractError::RoundNotInVoting { round_id: round.id });
        }

        owner_addresses.iter().for_each(|addr| {
            let id = round.project_number + 1;
            round.project_number = id;

            let project = Project {
                id,
                owner: addr.to_string(),
                area: 0,
                votes: 0,
                contribution: 0,
                status: ProjectStatus::OK,
            };
            self.projects
                .save(
                    deps.storage,
                    (&round_id.to_string(), &id.to_string()),
                    &project,
                )
                .unwrap();
        });

        self.rounds
            .save(deps.storage, &round_id.to_string(), &round)
            .unwrap();

        let resp = Response::new()
            .add_attribute("action", "batch_upload_project")
            .add_event(
                Event::new("batch_upload_project")
                    .add_attribute("round_id", round_id.to_string())
                    .add_attribute("projects", owner_addresses.join(", ").to_string()), // TODO...
            );
        Ok(resp)
    }

    #[msg(exec)]
    pub fn weighted_batch_vote(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        round_id: u64,
        project_ids: Vec<u64>,
        amounts: Vec<Uint128>,
        vcdora: u64,
        timestamp: u64,
        recid: u8,
        sig: Vec<u8>,
        sig_chain_id: String,
        sig_contract_addr: String,
    ) -> Result<Response, ContractError> {
        let (deps, env, info) = ctx;

        let mut round = self.rounds.load(deps.storage, &round_id.to_string())?;

        if round.status != RoundStatus::Voting {
            return Err(ContractError::RoundNotInVoting { round_id });
        }

        if project_ids.len() != amounts.len() {
            return Err(ContractError::LengthNotMatch {
                expected: project_ids.len() as u128,
                actual: amounts.len() as u128,
            });
        }

        let weight: u64;
        if sig.is_empty() {
            // If there is no signature, the weight is 1.0, which means vcDORA is not included in the calculation.
            weight = 10;
        } else if sig.len() != 64 {
            return Err(ContractError::InvalidSignatureLength {});
        } else {
            if round.pubkey.is_empty() {
                return Err(ContractError::PubkeyNotSet {});
            }
            // buidl msg
            let addr = deps.api.addr_canonicalize(info.sender.as_str()).unwrap();
            let addr_bytes = addr.as_slice();
            let msg = signature::build_msg(
                addr_bytes,
                round_id,
                &project_ids,
                &amounts,
                vcdora,
                timestamp,
                &sig_chain_id,
                &sig_contract_addr,
            );

            // verify signature
            if env.block.time.seconds() > timestamp + 60 * 60 {
                // 1 hour
                return Err(ContractError::InvalidSignatureTimestamp {});
            }
            let pubkey = signature::recover_pubkey(deps.as_ref(), msg, sig, recid);
            if pubkey != round.pubkey {
                return Err(ContractError::InvalidSignature {});
            }

            // calculate weight, 10 means 1.0
            weight = math::log2_u64_with_decimal(vcdora + 2)?; // plus 2 to avoid 0
        }

        let mut total_amounts = 0;
        let mut total_area = 0;

        let denom = round.donation_denom.clone();
        let denom_metadata = deps.querier.query_denom_metadata(&denom).unwrap();

        let mut denom_unit: Option<DenomUnit> = None;
        for unit in denom_metadata.denom_units {
            if unit.denom == denom_metadata.display {
                denom_unit = Some(unit);
                break;
            }
        }

        let decimals = denom_unit.unwrap().exponent;

        for (project_id, vote) in project_ids.iter().zip(amounts.iter()) {
            let amount = vote.u128();
            total_amounts = total_amounts + amount;
            let mut project = self.projects.load(
                deps.storage,
                (&round_id.to_string(), &project_id.to_string()),
            )?;

            let pow_10_decimals = 10u128.pow(decimals);
            let votes = amount * round.voting_unit.u128() / pow_10_decimals;
            if votes == 0 {
                return Err(ContractError::TooSmallAmount {
                    amount,
                });
            }

            project.votes = project.votes + votes;
            project.contribution = project.contribution + amount;

            // Compute area difference and update project/round area
            let mut old_votes: u128 = 0;
            let mut new_votes: u128 = votes;

            let votes = self.votes.may_load(
                deps.storage,
                (&round_id.to_string(), &project_id.to_string(), &info.sender),
            )?;
            match votes {
                Some(votes) => {
                    old_votes = votes;
                    new_votes = new_votes + old_votes;
                }
                None => {}
            }
            println!("old_votes: {} new_votes: {}", old_votes, new_votes);
            self.votes.save(
                deps.storage,
                (&round_id.to_string(), &project_id.to_string(), &info.sender),
                &new_votes,
            )?;

            let old_area = math::sqrt(old_votes * 100); // times 100 to avoid float, scale area by 10
            let new_area = math::sqrt(new_votes * 100);
            println!("old_area: {} new_area: {}", old_area, new_area);

            let area_diff = (new_area * weight / 10 - old_area) as u128; // adjust by weight, 10 means 1.0, div 10 to get the real weight

            project.area = project.area + area_diff;
            total_area = total_area + area_diff;
            println!("total_area inner: {} {}", total_area, area_diff);

            self.projects.save(
                deps.storage,
                (&round_id.to_string(), &project_id.to_string()),
                &project,
            )?;
        }
        let denom = round.donation_denom.clone();
        let transfer = cw_utils::must_pay(&info, &denom)
            .map_err(|err| StdError::generic_err(err.to_string()))?
            .u128();
        if transfer != total_amounts {
            return Err(ContractError::InvalidAmount {
                expected: total_amounts,
                actual: transfer,
            });
        }

        round.total_area += total_area;
        round.total_amounts += total_amounts;
        self.rounds
            .save(deps.storage, &round_id.to_string(), &round)?;

        let resp = Response::new()
            .add_attribute("action", "weighted_batch_vote")
            .add_event(
                Event::new("weighted_batch_vote")
                    .add_attribute("round_id", round_id.to_string())
                    .add_attribute(
                        "projects",
                        format!(
                            "{:?}",
                            project_ids
                                .iter()
                                .map(|x| x.to_string())
                                .collect::<String>()
                        ),
                    )
                    .add_attribute(
                        "amounts",
                        format!(
                            "{:?}",
                            amounts.iter().map(|x| x.to_string()).collect::<String>()
                        ),
                    )
                    .add_attribute("total_area", total_area.to_string()),
            );
        Ok(resp)
    }

    #[msg(exec)]
    pub fn end_round(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        round_id: u64,
    ) -> Result<Response, ContractError> {
        let (deps, _, info) = ctx;
        self.check_admin_permission(&deps, &info)?;

        let mut round = self.rounds.load(deps.storage, &round_id.to_string())?;

        if round.status != RoundStatus::Voting {
            return Err(ContractError::RoundNotInVoting { round_id });
        }

        round.status = RoundStatus::Finished;
        self.rounds
            .save(deps.storage, &round_id.to_string(), &round)?;

        let resp = Response::new()
            .add_attribute("action", "end_round")
            .add_event(Event::new("end_round").add_attribute("round_id", round_id.to_string()));
        Ok(resp)
    }

    #[msg(exec)]
    pub fn set_pubkey(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        round_id: u64,
        pubkey: Vec<u8>,
    ) -> Result<Response, ContractError> {
        let (deps, _, info) = ctx;
        self.check_admin_permission(&deps, &info)?;

        let mut round = self.rounds.load(deps.storage, &round_id.to_string())?;

        if round.status != RoundStatus::Voting {
            return Err(ContractError::RoundNotInVoting { round_id });
        }

        round.pubkey = pubkey.clone();
        self.rounds
            .save(deps.storage, &round_id.to_string(), &round)?;

        let resp = Response::new()
            .add_attribute("action", "set_pubkey")
            .add_event(
                Event::new("set_pubkey").add_attribute("pubkey", hex::encode(&pubkey.as_slice())),
            );
        Ok(resp)
    }

    #[msg(exec)]
    pub fn withdraw(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        round_id: u64,
    ) -> Result<Response, ContractError> {
        let (deps, _, info) = ctx;
        self.check_admin_permission(&deps, &info)?;

        let mut round = self.rounds.load(deps.storage, &round_id.to_string())?;
        let denom = round.donation_denom.clone();

        if round.status != RoundStatus::Finished {
            return Err(ContractError::RoundNotEnded { round_id });
        }

        round.status = RoundStatus::Withdrawn;
        self.rounds
            .save(deps.storage, &round_id.to_string(), &round)?;

        let amounts = round.total_amounts;
        let resp = if amounts > 0 {
            let message = BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: coins(amounts, &denom),
            };

            Response::new().add_message(message)
        } else {
            Response::new()
        };

        resp.clone().add_attribute("action", "withdraw").add_event(
            Event::new("withdraw")
                .add_attribute("round_id", round_id.to_string())
                .add_attribute("amounts", amounts.to_string()),
        );
        Ok(resp)
    }
}
