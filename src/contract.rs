//! Quadratic Grant Contract
//! Author: noodles@dorahacks.com
//! Version: 0.1.0
//! License: Apache-2.0

use cosmwasm_std::{
    coins, Addr, BankMsg, Deps, DepsMut, Empty, Env, Event, MessageInfo, Order, Response, StdError,
    StdResult, Uint128,
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
    pub(crate) admins: Map<'a, &'a Addr, Empty>,
    pub(crate) rounds: Map<'a, &'a str, Round>,
    pub(crate) current_round: Item<'a, u64>,
    pub(crate) projects: Map<'a, (&'a str, &'a str), Project>, // (round_id, project_id)
    pub(crate) votes: Map<'a, (&'a str, &'a str, &'a Addr), u128>, // (round_id, project_id, voter)
}

#[contract]
impl QGContract<'_> {
    pub const fn new() -> Self {
        Self {
            admins: Map::new("admins"),
            rounds: Map::new("rounds"),
            current_round: Item::new("current_round"),
            projects: Map::new("projects"),
            votes: Map::new("votes"),
        }
    }

    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        admins: Vec<String>,
    ) -> Result<Response, ContractError> {
        let (deps, _, _) = ctx;

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
    pub fn add_member(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        admin: String,
    ) -> Result<Response, ContractError> {
        let (deps, _, info) = ctx;
        if !self.admins.has(deps.storage, &info.sender) {
            return Err(ContractError::Unauthorized {
                sender: info.sender,
            });
        }

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
        if !self.admins.has(deps.storage, &info.sender) {
            return Err(ContractError::Unauthorized {
                sender: info.sender,
            });
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
        if !self.admins.has(deps.storage, &info.sender) {
            return Err(ContractError::Unauthorized {
                sender: info.sender,
            });
        }

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
    ) -> Result<Response, ContractError> {
        let (deps, env, info) = ctx;

        let mut round = self.rounds.load(deps.storage, &round_id.to_string())?;

        if round.status != RoundStatus::Voting {
            return Err(ContractError::RoundNotInVoting { round_id });
        }

        if round.pubkey.len() != 0 {
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
            );

            // verify signature
            if env.block.time.seconds() > timestamp + 60 * 60 {
                return Err(ContractError::InvalidSignatureTimestamp {});
            }
            let pubkey = signature::verify(deps.as_ref(), msg, sig, recid);
            if pubkey != round.pubkey {
                return Err(ContractError::InvalidSignature {});
            }
        }

        let weight = math::log2_u64_with_decimal(vcdora + 2);
        let mut total_amounts = 0;
        let mut total_area = 0;

        for (project_id, vote) in project_ids.iter().zip(amounts.iter()) {
            let amount = vote.u128();
            total_amounts = total_amounts + amount;
            let mut project = self.projects.load(
                deps.storage,
                (&round_id.to_string(), &project_id.to_string()),
            )?;

            const DECIMALS: u32 = 18; // TODO...

            let pow_10_decimals = 10u128.pow(DECIMALS);
            let votes = amount * round.voting_unit.u128() / pow_10_decimals;
            println!("votes: {} ", votes);
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

            let old_area = math::sqrt(old_votes * 100);
            let new_area = math::sqrt(new_votes * 100);
            println!("old_area: {} new_area: {}", old_area, new_area);

            let area_diff = (new_area * weight / 10 - old_area) as u128; // adjust by weight

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
        println!("total_area: {}", total_area);
        println!("round.total_area: {}", round.total_area);
        println!("round.total_amounts: {}", round.total_amounts);
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
        if !self.admins.has(deps.storage, &info.sender) {
            return Err(ContractError::Unauthorized {
                sender: info.sender,
            });
        }

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
    pub fn withdraw(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        round_id: u64,
    ) -> Result<Response, ContractError> {
        let (deps, _, info) = ctx;
        if !self.admins.has(deps.storage, &info.sender) {
            return Err(ContractError::Unauthorized {
                sender: info.sender,
            });
        }

        let mut round = self.rounds.load(deps.storage, &round_id.to_string())?;
        let denom = round.donation_denom.clone();

        if round.status != RoundStatus::Finished {
            return Err(ContractError::RoundNotEnded { round_id });
        }

        let amounts = round.total_amounts;
        let message = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: coins(amounts, &denom),
        };
        round.status = RoundStatus::Withdrawn;
        self.rounds
            .save(deps.storage, &round_id.to_string(), &round)?;

        let resp = Response::new()
            .add_message(message)
            .add_attribute("action", "withdraw")
            .add_event(
                Event::new("withdraw")
                    .add_attribute("round_id", round_id.to_string())
                    .add_attribute("amounts", amounts.to_string()),
            );
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use crate::entry_point::{execute, instantiate, query};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Coin, Uint128};

    use super::*;

    #[test]
    fn admin_list_query() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("sender", &[]),
            InstantiateMsg {
                admins: vec!["admin1".to_owned(), "admin2".to_owned()],
            },
        )
        .unwrap();

        let msg = QueryMsg::AdminList {};
        let resp = query(deps.as_ref(), env, ContractQueryMsg::QGContract(msg)).unwrap();
        let resp: AdminListResp = from_binary(&resp).unwrap();
        assert_eq!(
            resp,
            AdminListResp {
                admins: vec!["admin1".to_owned(), "admin2".to_owned()],
            }
        );
    }

    #[test]
    fn add_member() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("sender", &[]),
            InstantiateMsg {
                admins: vec!["admin1".to_owned(), "admin2".to_owned()],
            },
        )
        .unwrap();

        let info = mock_info("admin1", &[]);
        let msg = ExecMsg::AddMember {
            admin: "admin3".to_owned(),
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ContractExecMsg::QGContract(msg),
        )
        .unwrap();

        let msg = QueryMsg::AdminList {};
        let resp = query(deps.as_ref(), env, ContractQueryMsg::QGContract(msg)).unwrap();
        let resp: AdminListResp = from_binary(&resp).unwrap();
        assert_eq!(
            resp,
            AdminListResp {
                admins: vec![
                    "admin1".to_owned(),
                    "admin2".to_owned(),
                    "admin3".to_owned()
                ],
            }
        );
    }

    #[test]
    fn test_all() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        // Instantiate
        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("sender", &[]),
            InstantiateMsg {
                admins: vec!["admin1".to_owned(), "admin2".to_owned()],
            },
        )
        .unwrap();

        // Start round
        let info = mock_info("admin1", &[]);
        let msg = ExecMsg::StartRound {
            tax_adjustment_multiplier: 5,
            donation_denom: "inj".to_string(),
            voting_unit: Uint128::from(1_000_000_000_000_000_000u128),
            fund: Uint128::from(4000u128),
            pubkey: vec![],
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ContractExecMsg::QGContract(msg),
        )
        .unwrap();

        // Query round
        let msg = QueryMsg::Round { round_id: 1 };
        let resp = query(
            deps.as_ref(),
            env.clone(),
            ContractQueryMsg::QGContract(msg),
        )
        .unwrap();
        let resp: Round = from_binary(&resp).unwrap();
        assert_eq!(
            resp,
            Round {
                id: 1,
                tax_adjustment_multiplier: 5,
                donation_denom: "inj".to_string(),
                voting_unit: Uint128::from(1_000_000_000_000_000_000u128),
                status: RoundStatus::Voting,
                fund: Uint128::from(4000u128),
                project_number: 0,
                total_area: 0,
                total_amounts: 0,
                pubkey: vec![],
            }
        );

        // Upload project
        let info = mock_info("admin1", &[]);
        let msg = ExecMsg::BatchUploadProject {
            round_id: 1,
            owner_addresses: vec!["1".to_string(), "2".to_string()],
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ContractExecMsg::QGContract(msg),
        )
        .unwrap();

        let resp = query(
            deps.as_ref(),
            env.clone(),
            ContractQueryMsg::QGContract(QueryMsg::Round { round_id: 1 }),
        )
        .unwrap();
        let resp: Round = from_binary(&resp).unwrap();
        assert_eq!(
            resp,
            Round {
                id: 1,
                tax_adjustment_multiplier: 5,
                donation_denom: "inj".to_string(),
                voting_unit: Uint128::from(1_000_000_000_000_000_000u128),
                status: RoundStatus::Voting,
                fund: Uint128::from(4000u128),
                project_number: 2,
                total_area: 0,
                total_amounts: 0,
                pubkey: vec![],
            }
        );
        let resp = query(
            deps.as_ref(),
            env.clone(),
            ContractQueryMsg::QGContract(QueryMsg::Project {
                round_id: 1,
                project_id: 2,
            }),
        )
        .unwrap();
        let resp: Project = from_binary(&resp).unwrap();
        assert_eq!(
            resp,
            Project {
                id: 2,
                owner: "2".to_string(),
                area: 0,
                status: ProjectStatus::OK,
                votes: 0,
                contribution: 0,
            }
        );

        // Vote
        let msg = ExecMsg::WeightedBatchVote {
            round_id: 1,
            project_ids: vec![1],
            amounts: vec![Uint128::from(160000u128)],
            vcdora: 0,
            recid: 0,
            sig: vec![],
            timestamp: 0,
        };
        let info = mock_info(
            "user1",
            &[Coin {
                denom: "inj".to_string(),
                amount: Uint128::from(160000u128),
            }],
        );
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ContractExecMsg::QGContract(msg),
        )
        .unwrap();

        let msg = ExecMsg::WeightedBatchVote {
            round_id: 1,
            project_ids: vec![1],
            amounts: vec![Uint128::from(90000u128)],
            vcdora: 1,
            recid: 0,
            sig: vec![],
            timestamp: 0,
        };
        let info = mock_info(
            "user1",
            &[Coin {
                denom: "inj".to_string(),
                amount: Uint128::from(90000u128),
            }],
        );
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ContractExecMsg::QGContract(msg),
        )
        .unwrap();

        let msg = ExecMsg::WeightedBatchVote {
            round_id: 1,
            project_ids: vec![2],
            amounts: vec![Uint128::from(160000u128)],
            vcdora: 123143400,
            recid: 0,
            sig: vec![],
            timestamp: 0,
        };
        let info = mock_info(
            "user1",
            &[Coin {
                denom: "inj".to_string(),
                amount: Uint128::from(160000u128),
            }],
        );
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ContractExecMsg::QGContract(msg),
        )
        .unwrap();

        let resp = query(
            deps.as_ref(),
            env.clone(),
            ContractQueryMsg::QGContract(QueryMsg::Round { round_id: 1 }),
        )
        .unwrap();
        let resp: Round = from_binary(&resp).unwrap();
        assert_eq!(
            resp,
            Round {
                id: 1,
                tax_adjustment_multiplier: 5,
                donation_denom: "inj".to_string(),
                voting_unit: Uint128::from(1_000_000_000_000_000_000u128),
                status: RoundStatus::Voting,
                fund: Uint128::from(4000u128),
                project_number: 2,
                total_area: 500 * 15 + 400 * 268,
                total_amounts: 410000,
                pubkey: vec![],
            }
        );
        let resp = query(
            deps.as_ref(),
            env.clone(),
            ContractQueryMsg::QGContract(QueryMsg::Project {
                round_id: 1,
                project_id: 2,
            }),
        )
        .unwrap();
        let resp: Project = from_binary(&resp).unwrap();
        assert_eq!(
            resp,
            Project {
                id: 2,
                owner: "2".to_string(),
                area: 107200,
                status: ProjectStatus::OK,
                votes: 160000,
                contribution: 160000,
            }
        );

        // End round
        let info = mock_info("admin1", &[]);
        let msg = ExecMsg::EndRound { round_id: 1 };
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ContractExecMsg::QGContract(msg),
        )
        .unwrap();

        // Check round status
        let resp = query(
            deps.as_ref(),
            env.clone(),
            ContractQueryMsg::QGContract(QueryMsg::Round { round_id: 1 }),
        )
        .unwrap();
        let resp: Round = from_binary(&resp).unwrap();
        assert_eq!(
            resp,
            Round {
                id: 1,
                tax_adjustment_multiplier: 5,
                donation_denom: "inj".to_string(),
                voting_unit: Uint128::from(1_000_000_000_000_000_000u128),
                status: RoundStatus::Finished,
                fund: Uint128::from(4000u128),
                project_number: 2,
                total_area: 500 * 15 + 400 * 268,
                total_amounts: 410000,
                pubkey: vec![],
            }
        );

        // Withdraw
        let info = mock_info("admin1", &[]);
        let msg = ExecMsg::Withdraw { round_id: 1 };
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ContractExecMsg::QGContract(msg),
        )
        .unwrap();
    }
}
