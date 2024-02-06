use cosmwasm_std::{Addr, };
use cw_multi_test::App;

use crate::{
    error::ContractError, multitest::proxy::QGContractCodeId, responses::AdminListResp,
};

#[test]
fn basic() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let admins = vec![
        "admin1".to_owned(),
        "admin2".to_owned(),
        "admin3".to_owned(),
    ];

    let code_id = QGContractCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app, 
            &owner, 
            admins.clone(), 
            "Quadratic grant contract", 
            None)
        .unwrap();

    let resp = contract.admin_list(&app).unwrap();

    assert_eq!(resp, AdminListResp { admins });
}

#[test]
fn unathorized() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let admin1 = Addr::unchecked("admin1");
    let admin2 = Addr::unchecked("admin2");
    let admin3 = Addr::unchecked("admin3");

    let code_id = QGContractCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            vec![admin1.to_string(), admin2.to_string()],
            "Quadratic grant contract", 
            None,
        )
        .unwrap();

    let resp = contract.admin_list(&app).unwrap();

    assert_eq!(
        resp,
        AdminListResp {
            admins: vec![admin1.to_string(), admin2.to_string()]
        }
    );

    let err = contract
        .add_member(&mut app, &admin3, admin3.to_string())
        .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized { sender: admin3 });

    let resp = contract.admin_list(&app).unwrap();

    assert_eq!(
        resp,
        AdminListResp {
            admins: vec![admin1.to_string(), admin2.to_string()]
        }
    );
}

#[test]
fn no_dup() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let admin1 = Addr::unchecked("admin1");
    let admin2 = Addr::unchecked("admin2");

    let code_id = QGContractCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            vec![admin1.to_string(), admin2.to_string()],
            "Quadratic grant contract", 
            None,
        )
        .unwrap();

    let resp = contract.admin_list(&app).unwrap();

    assert_eq!(
        resp,
        AdminListResp {
            admins: vec![admin1.to_string(), admin2.to_string()]
        }
    );

    let err = contract
        .add_member(&mut app, &admin1, admin1.to_string())
        .unwrap_err();

    assert_eq!(err, ContractError::NoDupAddress { address: admin1.to_owned() });

    let resp = contract.admin_list(&app).unwrap();

    assert_eq!(
        resp,
        AdminListResp {
            admins: vec![admin1.to_string(), admin2.to_string()]
        }
    );
}


#[cfg(test)]
mod tests {
    use crate::contract::{InstantiateMsg, QueryMsg, ContractQueryMsg, ExecMsg, ContractExecMsg};
    use crate::entry_point::{execute, instantiate, query};
    use crate::state::{Round, RoundStatus, Project, ProjectStatus};
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
            sig_chain_id: "".to_string(),
            sig_contract_addr: "".to_string(),
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
            sig_chain_id: "".to_string(),
            sig_contract_addr: "".to_string(),
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
            sig_chain_id: "".to_string(),
            sig_contract_addr: "".to_string(),
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
                total_area: 500 * 10 + 400 * 10,
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
                area: 4000,
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

        // Set pubkey
        let info = mock_info("admin1", &[]);
        let pubkey = hex::decode("0479be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8").unwrap();
        let msg = ExecMsg::SetPubkey {
            round_id: 1,
            pubkey: pubkey.clone(),
        };
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
                total_area: 500 * 10 + 400 * 10,
                total_amounts: 410000,
                pubkey,
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
