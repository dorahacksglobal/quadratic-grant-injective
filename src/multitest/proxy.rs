use cosmwasm_std::{Addr, StdResult, Coin};
use cw_multi_test::{App, AppResponse, Executor};

use crate::{
    contract::{QGContract, ExecMsg, InstantiateMsg, QueryMsg},
    error::ContractError,
    responses::AdminListResp,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QGContractCodeId(u64);

impl QGContractCodeId {
    pub fn store_code(app: &mut App) -> Self {
        let code_id = app.store_code(Box::new(QGContract::new()));
        Self(code_id)
    }

    #[track_caller]
    pub fn instantiate(
        self,
        app: &mut App,
        sender: &Addr,
        admins: Vec<String>,
        donation_denom: String,
        label: &str,
        admin: Option<String>,
    ) -> StdResult<QGContractProxy> {
        let msg = InstantiateMsg { admins, donation_denom};

        app.instantiate_contract(self.0, sender.clone(), &msg, &[], label, admin)
            .map_err(|err| err.downcast().unwrap())
            .map(QGContractProxy)
    }
}

#[derive(Debug)]
pub struct QGContractProxy(Addr);

impl QGContractProxy {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    #[track_caller]
    pub fn admin_list(&self, app: &App) -> StdResult<AdminListResp> {
        let msg = QueryMsg::AdminList {};

        app.wrap().query_wasm_smart(self.0.clone(), &msg)
    }

    #[track_caller]
    pub fn add_member(
        &self,
        app: &mut App,
        sender: &Addr,
        admin: String,
    ) -> Result<AppResponse, ContractError> {
        let msg = ExecMsg::AddMember { admin };

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn donate(
        &self,
        app: &mut App,
        sender: &Addr,
        funds: &[Coin],
    ) -> Result<AppResponse, ContractError> {
        let msg = ExecMsg::Donate {};

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &funds)
            .map_err(|err| err.downcast().unwrap())
    }
}
