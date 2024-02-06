#[cfg(test)]
mod tests {
    use crate::contract::sv::{
        ContractExecMsg, ContractQueryMsg, ExecMsg, InstantiateMsg, QueryMsg,
    };
    use crate::entry_point::{execute, instantiate, query};
    use crate::responses::AdminListResp;
    use crate::state::{Project, ProjectStatus, Round, RoundStatus};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_json, Coin, DenomMetadata, DenomUnit, Timestamp, Uint128};

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
        let resp: AdminListResp = from_json(&resp).unwrap();
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
        let resp: AdminListResp = from_json(&resp).unwrap();
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
        let mut env = mock_env();

        let denom_meta_data = DenomMetadata {
            base: "inj".to_string(),
            display: "inj".to_string(),
            name: "inj".to_string(),
            description: "inj".to_string(),
            symbol: "inj".to_string(),
            uri: "inj".to_string(),
            uri_hash: "inj".to_string(),
            denom_units: vec![DenomUnit {
                denom: "inj".to_string(),
                exponent: 18,
                aliases: vec!["inj".to_string()],
            }],
        };
        deps.querier.set_denom_metadata(&vec![denom_meta_data]);
        deps.querier.update_balance(
            "test",
            vec![Coin {
                denom: "inj".to_string(),
                amount: Uint128::from(1000000000000000000u128),
            }],
        );

        env.block.time = Timestamp::from_seconds(1682415684);

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
        let resp: Round = from_json(&resp).unwrap();
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
        let resp: Round = from_json(&resp).unwrap();
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
        let resp: Project = from_json(&resp).unwrap();
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
        let resp: Round = from_json(&resp).unwrap();
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
        let resp: Project = from_json(&resp).unwrap();
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
        let resp: Round = from_json(&resp).unwrap();
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
