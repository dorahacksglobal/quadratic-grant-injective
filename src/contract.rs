use crate::error::ContractError;
use crate::responses::AdminListResp;
use cosmwasm_std::{
    coins, Addr, BankMsg, Deps, DepsMut, Empty, Env, Event, MessageInfo, Order, Response, StdError,
    StdResult,
};
use cw_storage_plus::{Item, Map};
use schemars;
use sylvia::contract;

pub struct AdminContract<'a> {
    pub(crate) admins: Map<'a, &'a Addr, Empty>,
    pub(crate) donation_denom: Item<'a, String>,
}

#[contract]
impl AdminContract<'_> {
    pub const fn new() -> Self {
        Self {
            admins: Map::new("admins"),
            donation_denom: Item::new("donation_denom"),
        }
    }

    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        admins: Vec<String>,
        donation_denom: String,
    ) -> Result<Response, ContractError> {
        let (deps, _, _) = ctx;

        for admin in admins {
            let admin = deps.api.addr_validate(&admin)?;
            self.admins.save(deps.storage, &admin, &Empty {})?;
        }
        self.donation_denom.save(deps.storage, &donation_denom)?;
        Ok(Response::new())
    }

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
    pub fn donate(&self, ctx: (DepsMut, Env, MessageInfo)) -> Result<Response, ContractError> {
        let (deps, _, info) = ctx;

        let denom = self.donation_denom.load(deps.storage)?;
        let admins_len = self
            .admins
            .keys(deps.storage, None, None, Order::Ascending)
            .filter_map(|admin| admin.ok())
            .count();

        let donation = cw_utils::must_pay(&info, &denom)
            .map_err(|err| StdError::generic_err(err.to_string()))?
            .u128();

        let donation_per_admin = donation / (admins_len as u128);

        let admins = self
            .admins
            .keys(deps.storage, None, None, Order::Ascending)
            .filter_map(|admin| admin.ok());

        let messages = admins.into_iter().map(|admin| BankMsg::Send {
            to_address: admin.to_string(),
            amount: coins(donation_per_admin, &denom),
        });

        let resp = Response::new()
            .add_messages(messages)
            .add_attribute("action", "donate")
            .add_attribute("amount", donation.to_string())
            .add_attribute("per_admin", donation_per_admin.to_string());

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use crate::entry_point::{execute, instantiate, query};
    use cosmwasm_std::from_binary;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

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
                donation_denom: "uusd".to_owned(),
            },
        )
        .unwrap();

        let msg = QueryMsg::AdminList {};
        let resp = query(deps.as_ref(), env, ContractQueryMsg::AdminContract(msg)).unwrap();
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
                donation_denom: "uusd".to_owned(),
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
            ContractExecMsg::AdminContract(msg),
        )
        .unwrap();

        let msg = QueryMsg::AdminList {};
        let resp = query(deps.as_ref(), env, ContractQueryMsg::AdminContract(msg)).unwrap();
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
}
